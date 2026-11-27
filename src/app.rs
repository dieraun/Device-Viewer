//! eframe application shell: startup collection, navigation, toolbar and export.

use std::cmp::Ordering;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use eframe::egui;
use serde::Deserialize;
use tokio::runtime::Runtime;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

use crate::device_query::opengl;
use crate::device_query::vulkan::{self, GpuRuntimeStats};
use crate::device_query::{DEFAULT_MODULE_TIMEOUT, HardwareSnapshot, QueryOptions, collect_all};
use crate::export::{self, ExportFormat};
use crate::i18n::{self, I18n};
use crate::log;
use crate::state::{AppState, ThemeMode, UpdateReport, UpdateState};
use crate::ui;
use crate::ui::theme;

const UPDATE_ENDPOINT: &str = "https://api.github.com/repos/dieraun/Device-Viewer/releases";
const UPDATE_USER_AGENT: &str = "device-viewer";

enum QueryEvent {
    Snapshot {
        generation: u64,
        snapshot: Box<HardwareSnapshot>,
    },
    Gpu {
        generation: u64,
        stats: Vec<GpuRuntimeStats>,
    },
    GpuFailed {
        generation: u64,
        message: String,
    },
    Update {
        generation: u64,
        result: Result<UpdateReport, String>,
    },
}

pub fn run(portable: bool) -> eframe::Result {
    // 上次退出时的位置与尺寸；缺失或不合理时退回默认窗口。
    let viewport = match crate::state::saved_window_rect(portable) {
        Some([x, y, width, height]) => egui::ViewportBuilder::default()
            .with_inner_size([width, height])
            .with_position([x, y]),
        None => egui::ViewportBuilder::default().with_inner_size([1280.0, 820.0]),
    }
    .with_min_inner_size([980.0, 640.0]);
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "Device Viewer",
        options,
        Box::new(move |creation| Ok(Box::new(DeviceViewerApp::new(creation, portable)))),
    )
}

pub struct DeviceViewerApp {
    state: AppState,
    i18n: I18n,
    runtime: Option<Runtime>,
    sender: std::sync::mpsc::Sender<QueryEvent>,
    receiver: Receiver<QueryEvent>,
    cancel: CancellationToken,
    gpu_cancel: CancellationToken,
    generation: u64,
    update_generation: u64,
    busy: bool,
    started: Instant,
    gpu_inflight: bool,
    last_gpu_poll: Instant,
    last_gpu_sample: Instant,
    export_open: bool,
    more_open: bool,
    /// 命令面板（Ctrl+K）是否打开。
    palette_open: bool,
    palette_query: String,
    palette_selection: usize,
    /// 打开面板的那一帧要把焦点交给输入框，否则 Ctrl+K 之后还得再点一下才能打字。
    palette_focus: bool,
    export_format: ExportFormat,
    export_preview: String,
    startup_error: Option<String>,
    applied_theme: Option<egui::ThemePreference>,
    last_window_rect: Option<[f32; 4]>,
    /// OpenGL 身份只在获得渲染上下文后探测一次，失败也不逐帧重试。
    opengl_probed: bool,
    /// 采集是互斥的：取消旧一轮后，旧任务可能仍阻塞在平台调用里，
    /// 用信号量保证同一时刻只有一个采集批次真正运行。
    query_semaphore: Arc<Semaphore>,
}

/// 命令面板可执行的动作：页面跳转或全局操作。
#[derive(Clone, Copy)]
enum PaletteAction {
    Page(crate::state::Page),
    Refresh,
    Export,
    ToggleTheme,
}

impl DeviceViewerApp {
    pub fn new(creation: &eframe::CreationContext<'_>, portable: bool) -> Self {
        i18n::install_fonts(&creation.egui_ctx);
        let state = AppState::load(portable);
        log::init(Some(crate::state::log_file(state.settings.portable)));
        let i18n = I18n::new(state.settings.language);
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(|error| format!("cannot start the async runtime: {error}"))
            .ok();
        let (sender, receiver) = std::sync::mpsc::channel();

        log::info("Device Viewer starting");

        let mut app = Self {
            state,
            i18n,
            runtime,
            sender,
            receiver,
            cancel: CancellationToken::new(),
            gpu_cancel: CancellationToken::new(),
            generation: 0,
            update_generation: 0,
            busy: false,
            started: Instant::now(),
            gpu_inflight: false,
            last_gpu_poll: Instant::now(),
            last_gpu_sample: Instant::now(),
            export_open: false,
            more_open: false,
            palette_open: false,
            palette_query: String::new(),
            palette_selection: 0,
            palette_focus: false,
            export_format: ExportFormat::Markdown,
            export_preview: String::new(),
            startup_error: None,
            applied_theme: None,
            last_window_rect: None,
            opengl_probed: false,
            query_semaphore: Arc::new(Semaphore::new(1)),
        };
        crate::ui::widgets::install_style(&creation.egui_ctx);
        app.apply_theme(&creation.egui_ctx);
        app.start_query();
        // 启动时自动检查更新受 24 小时节流：GitHub 未认证 API 每小时只有
        // 60 次额度，频繁重启必须复用上一次的检查时间。
        let now = now_ms();
        if app.state.settings.auto_check_updates
            && now.saturating_sub(app.state.settings.last_update_check_ms)
                >= UPDATE_CHECK_INTERVAL_MS
        {
            app.start_update_check();
        }
        app
    }

    fn apply_theme(&mut self, ctx: &egui::Context) {
        let preference = match self.state.settings.theme {
            ThemeMode::System => egui::ThemePreference::System,
            ThemeMode::Dark => egui::ThemePreference::Dark,
            ThemeMode::Light => egui::ThemePreference::Light,
        };
        if self.applied_theme != Some(preference) {
            ctx.set_theme(preference);
            self.applied_theme = Some(preference);
        }
    }

    fn start_query(&mut self) {
        self.cancel.cancel();
        self.gpu_cancel.cancel();
        self.cancel = CancellationToken::new();
        self.gpu_cancel = CancellationToken::new();
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        self.busy = true;
        self.started = Instant::now();
        self.gpu_inflight = false;
        log::debug(format!("starting collection generation {generation}"));
        let Some(runtime) = &self.runtime else {
            self.busy = false;
            self.startup_error = Some("the async runtime is unavailable".to_string());
            log::error("async runtime is unavailable");
            return;
        };
        let token = self.cancel.clone();
        let sender = self.sender.clone();
        let semaphore = self.query_semaphore.clone();
        runtime.spawn(async move {
            // 已取消的旧批次可能还占着许可；等到它退出或本轮被取消。
            let Ok(_permit) = semaphore.acquire_owned().await else {
                return;
            };
            if token.is_cancelled() {
                return;
            }
            let snapshot = collect_all(&QueryOptions::default(), &token).await;
            // 结果只带自己的代号返回；poll_events 会丢弃非当前代号的快照，
            // 因此被取消的旧任务不可能把过期数据写回界面。
            let _ = sender.send(QueryEvent::Snapshot {
                generation,
                snapshot: Box::new(snapshot),
            });
        });
    }

    fn start_update_check(&mut self) {
        if self.state.update.is_checking() {
            return;
        }
        self.update_generation = self.update_generation.wrapping_add(1);
        let generation = self.update_generation;
        let current = env!("CARGO_PKG_VERSION").to_string();
        let include_prerelease = self.state.settings.include_prerelease;
        self.state.update = UpdateState::Checking;
        let Some(runtime) = &self.runtime else {
            self.state.update = UpdateState::Failed("the async runtime is unavailable".to_string());
            log::warn("update check skipped: async runtime is unavailable");
            return;
        };
        // 请求一旦发出就会消耗 API 额度，无论成功还是失败都要记下时间，
        // 这样重启后不会再立刻发起第二次请求。
        self.state.settings.last_update_check_ms = now_ms();
        self.state.mark_dirty();
        let sender = self.sender.clone();
        runtime.spawn_blocking(move || {
            let result = check_for_update(&current, include_prerelease);
            let _ = sender.send(QueryEvent::Update { generation, result });
        });
    }

    fn poll_gpu(&mut self) {
        if self.gpu_inflight || self.runtime.is_none() {
            return;
        }
        if self.last_gpu_poll.elapsed() < Duration::from_millis(1000) {
            return;
        }
        let Some(runtime) = &self.runtime else {
            return;
        };
        self.last_gpu_poll = Instant::now();
        self.gpu_inflight = true;
        let generation = self.generation;
        let token = self.gpu_cancel.clone();
        let sender = self.sender.clone();
        runtime.spawn(async move {
            match vulkan::query_runtime_stats(&token).await {
                Ok(stats) => {
                    let _ = sender.send(QueryEvent::Gpu { generation, stats });
                }
                Err(error) => {
                    let _ = sender.send(QueryEvent::GpuFailed {
                        generation,
                        message: error.message,
                    });
                }
            }
        });
    }

    fn poll_events(&mut self) {
        loop {
            match self.receiver.try_recv() {
                Ok(QueryEvent::Snapshot {
                    generation,
                    snapshot,
                }) => {
                    if generation != self.generation {
                        continue;
                    }
                    self.busy = false;
                    self.state.snapshot = *snapshot;
                    // 新一轮采集结果需要重新从渲染上下文取 OpenGL 身份。
                    self.opengl_probed = false;
                    self.state.record_gpu_sample();
                    self.last_gpu_sample = Instant::now();
                    // 导出窗口开着时，预览必须跟着新快照走；否则用户点刷新
                    // 看到的仍是上一轮采集结果。
                    if self.export_open {
                        self.refresh_export();
                    }
                    log::info(format!(
                        "collection finished with {} warning(s) and {} error(s)",
                        self.state.snapshot.warning_count(),
                        self.state.snapshot.error_count()
                    ));
                }
                Ok(QueryEvent::Gpu { generation, stats }) => {
                    if generation != self.generation {
                        continue;
                    }
                    self.gpu_inflight = false;
                    self.state.gpu_runtime = stats;
                    if self.last_gpu_sample.elapsed() >= Duration::from_millis(500) {
                        self.state.record_gpu_sample();
                        self.last_gpu_sample = Instant::now();
                    }
                }
                Ok(QueryEvent::GpuFailed {
                    generation,
                    message,
                }) => {
                    if generation != self.generation {
                        continue;
                    }
                    self.gpu_inflight = false;
                    log::warn(format!("live GPU sampling failed: {message}"));
                }
                Ok(QueryEvent::Update { generation, result }) => {
                    if generation != self.update_generation {
                        continue;
                    }
                    match result {
                        Ok(report) => {
                            log::info(format!(
                                "update check: current={} latest={} available={} prerelease={}",
                                report.current, report.latest, report.available, report.prerelease
                            ));
                            self.state.update = UpdateState::Ready(report);
                        }
                        Err(message) => {
                            log::warn(format!("update check failed: {message}"));
                            self.state.update = UpdateState::Failed(message);
                        }
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
    }

    /// Fills the OpenGL section from eframe's live glow context.
    ///
    /// The background collector has no rendering context, so it can only mark
    /// the section as unavailable. The GUI render thread does own one, and
    /// this is the only place where querying `GL_VENDOR`/`GL_RENDERER` is
    /// valid. It runs once per collection result and is a no-op afterwards.
    fn refresh_opengl(&mut self, frame: &eframe::Frame) {
        if self.opengl_probed {
            return;
        }
        if self
            .state
            .snapshot
            .opengl
            .as_ref()
            .is_some_and(|info| info.available)
        {
            self.opengl_probed = true;
            return;
        }
        let Some(gl) = frame.gl() else {
            // 渲染上下文尚未就绪：下一帧再试，不缓存失败状态。
            return;
        };
        self.state.snapshot.opengl = Some(opengl::query_from_context(gl));
        self.opengl_probed = true;
        log::info("OpenGL identity captured from the rendering context");
    }

    fn open_export(&mut self) {
        self.export_preview = export::render(
            &self.state.snapshot,
            self.export_format,
            self.i18n,
            self.state.settings.units,
        );
        self.export_open = true;
    }

    fn refresh_export(&mut self) {
        self.export_preview = export::render(
            &self.state.snapshot,
            self.export_format,
            self.i18n,
            self.state.settings.units,
        );
    }

    fn save_export(&mut self) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or_default();
        let path = self.state.exports_dir.join(format!(
            "device-viewer-export-{timestamp}.{}",
            self.export_format.extension()
        ));
        if let Some(parent) = path.parent()
            && let Err(error) = std::fs::create_dir_all(parent)
        {
            log::error(format!("cannot create export directory: {error}"));
            self.state
                .ui
                .notify_error(format!("export failed: {error}"));
            return;
        }
        match std::fs::write(&path, &self.export_preview) {
            Ok(()) => {
                self.state
                    .ui
                    .notify_success(format!("Saved {}", path.display()));
                log::info(format!("exported report to {}", path.display()));
            }
            Err(error) => {
                log::error(format!("cannot write export {}: {error}", path.display()));
                self.state
                    .ui
                    .notify_error(format!("export failed: {error}"));
            }
        }
    }

    /// 命令面板：输入即搜页面与全局动作，支持上下键选择。
    fn palette_window(&mut self, ctx: &egui::Context) {
        if !self.palette_open {
            return;
        }
        let t = theme::Theme::of(ctx.theme() == egui::Theme::Dark);
        let mut open = true;
        let mut chosen: Option<PaletteAction> = None;
        egui::Window::new(self.i18n.t("shell.palette"))
            .id(egui::Id::unique("command-palette"))
            .open(&mut open)
            .anchor(egui::Align2::CENTER_TOP, [0.0, 96.0])
            .min_width(520.0)
            .max_width(520.0)
            .resizable(false)
            .frame(t.overlay_frame())
            .show(ctx, |ui| {
                let query_edit = ui.add(
                    egui::TextEdit::singleline(&mut self.palette_query)
                        .hint_text(self.i18n.t("shell.palette_hint"))
                        .desired_width(f32::INFINITY),
                );
                if self.palette_focus {
                    query_edit.request_focus();
                    self.palette_focus = false;
                }
                let query = self.palette_query.to_lowercase();
                let mut items: Vec<(PaletteAction, String)> = Vec::new();
                for section in ui::sidebar::SECTIONS {
                    for page in section.pages {
                        let title = self.i18n.t(page.title_key()).to_string();
                        if query.is_empty() || title.to_lowercase().contains(&query) {
                            items.push((PaletteAction::Page(*page), title));
                        }
                    }
                }
                // 页面之外再放三个全局动作，这样 23 个入口可以真正藏进 Ctrl+K。
                for (action, label) in [
                    (PaletteAction::Refresh, self.i18n.t("common.refresh")),
                    (PaletteAction::Export, self.i18n.t("nav.export")),
                    (PaletteAction::ToggleTheme, self.i18n.t("settings.theme")),
                ] {
                    if query.is_empty() || label.to_lowercase().contains(&query) {
                        items.push((action, label.to_string()));
                    }
                }
                if items.is_empty() {
                    ui.add_space(theme::SPACE_3);
                    ui.label(
                        egui::RichText::new(self.i18n.t("shell.palette_empty"))
                            .size(theme::FONT_BODY)
                            .color(t.text_tertiary),
                    );
                    return;
                }
                let last = items.len() - 1;
                if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    self.palette_selection = (self.palette_selection + 1).min(last);
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    self.palette_selection = self.palette_selection.saturating_sub(1);
                }
                self.palette_selection = self.palette_selection.min(last);
                let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                if enter {
                    chosen = Some(items[self.palette_selection].0);
                }
                ui.add_space(theme::SPACE_2);
                egui::ScrollArea::vertical()
                    .id_salt("palette-results")
                    .max_height(320.0)
                    .show(ui, |ui| {
                        for (index, (action, title)) in items.iter().enumerate() {
                            let selected = index == self.palette_selection;
                            if ui.selectable_label(selected, title).clicked() {
                                chosen = Some(*action);
                            }
                        }
                    });
            });
        if let Some(action) = chosen {
            self.palette_open = false;
            match action {
                PaletteAction::Page(page) => {
                    self.state.settings.page = page;
                    self.state.mark_dirty();
                }
                PaletteAction::Refresh => self.start_query(),
                PaletteAction::Export => self.open_export(),
                PaletteAction::ToggleTheme => {
                    self.state.settings.theme = self.state.settings.theme.cycle();
                    self.state.mark_dirty();
                }
            }
        }
        if !open {
            self.palette_open = false;
        }
    }

    /// 吸顶标题栏：只放页面标题、刷新、导出和更多菜单。语言与主题在设置页，
    /// 版本号只在侧栏出现一次。
    fn header(&mut self, ui: &mut egui::Ui) {
        let t = ui::theme::theme(ui);
        let page = self.state.settings.page;
        ui.horizontal(|ui| {
            ui.add_space(theme::SPACE_1);
            ui.label(
                egui::RichText::new(self.i18n.t(page.title_key()))
                    .size(theme::FONT_H1)
                    .strong()
                    .color(t.text_primary),
            )
            .on_hover_text(self.i18n.t(page.subtitle_key()));
            if self.busy {
                let (rect, response) =
                    ui.allocate_exact_size(egui::Vec2::splat(10.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 3.5, t.accent);
                response.on_hover_text(format!(
                    "{} {:.1}s",
                    self.i18n.t("app.collecting"),
                    self.started.elapsed().as_secs_f32()
                ));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(self.i18n.t("shell.more"))
                                .size(theme::FONT_BODY)
                                .color(t.text_secondary),
                        )
                        .frame(false),
                    )
                    .on_hover_text(self.i18n.t("shell.more_hint"))
                    .clicked()
                {
                    self.more_open = !self.more_open;
                }
                if ui::widgets::icon_text_button(ui, theme::Icon::Export, self.i18n.t("nav.export"))
                    .clicked()
                {
                    self.open_export();
                }
                if ui::widgets::icon_text_button(
                    ui,
                    theme::Icon::Refresh,
                    self.i18n.t("common.refresh"),
                )
                .clicked()
                {
                    self.state.ui.request_refresh = true;
                }
            });
        });
    }

    /// 启动采集期间的欢迎页：品牌标识、骨架卡片和一条真实进度。
    ///
    /// 整块内容在窗口里居中；骨架块的假文字行带一点呼吸感，让"正在读取"
    /// 看起来是在工作，而不是卡住的空窗口。
    fn loading_page(&mut self, ui: &mut egui::Ui) {
        let t = ui::theme::theme(ui);
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(t.canvas))
            .show(ui, |ui| {
                let width = ui.available_width().min(560.0);
                let pad_x = ((ui.available_width() - width) / 2.0).max(0.0);
                let pad_y = ((ui.available_height() - 372.0) / 2.0).max(24.0);
                let pulse = 0.55 + 0.15 * (ui.input(|input| input.time) * 2.2).sin() as f32;
                ui.add_space(pad_y);
                ui.horizontal(|ui| {
                    ui.add_space(pad_x);
                    ui.vertical(|ui| {
                        ui.set_width(width);
                        ui.vertical_centered(|ui| {
                            let (tile, _) = ui
                                .allocate_exact_size(egui::Vec2::splat(64.0), egui::Sense::hover());
                            // 一圈柔光，让品牌块不像是贴在纯色背景上的方块。
                            theme::paint_glow(ui.painter(), tile.center(), 200.0, t.accent);
                            ui.painter().rect_filled(
                                tile,
                                egui::CornerRadius::same(18),
                                t.accent_subtle,
                            );
                            theme::paint_icon(
                                ui.painter(),
                                egui::Rect::from_center_size(
                                    tile.center(),
                                    egui::Vec2::splat(30.0),
                                ),
                                theme::Icon::Hardware,
                                t.accent,
                            );

                            ui.add_space(theme::SPACE_4);
                            ui.label(
                                egui::RichText::new("Device Viewer")
                                    .size(theme::FONT_DISPLAY)
                                    .strong()
                                    .color(t.text_primary),
                            );
                            ui.add_space(theme::SPACE_1);
                            ui.label(
                                egui::RichText::new(format!(
                                    "v{} · MIT OR Apache-2.0",
                                    env!("CARGO_PKG_VERSION")
                                ))
                                .size(theme::FONT_CAPTION)
                                .color(t.text_tertiary),
                            );

                            ui.add_space(theme::SPACE_6);
                            ui.horizontal(|ui| {
                                ui.add(egui::Spinner::new().size(14.0).color(t.accent));
                                ui.label(
                                    egui::RichText::new(self.i18n.t("loading.reading"))
                                        .size(theme::FONT_BODY)
                                        .color(t.text_secondary),
                                );
                            });

                            ui.add_space(theme::SPACE_6);
                            let gap = theme::SPACE_3;
                            let card = ((width - gap * 2.0) / 3.0).max(80.0);
                            ui.horizontal(|ui| {
                                for index in 0..3 {
                                    let (rect, _) = ui.allocate_exact_size(
                                        egui::Vec2::new(card, 84.0),
                                        egui::Sense::hover(),
                                    );
                                    let painter = ui.painter().clone();
                                    painter.rect_filled(
                                        rect,
                                        egui::CornerRadius::same(theme::RADIUS_CARD),
                                        t.raised,
                                    );
                                    painter.rect_stroke(
                                        rect,
                                        egui::CornerRadius::same(theme::RADIUS_CARD),
                                        egui::Stroke::new(theme::BORDER_WIDTH, t.border_subtle),
                                        egui::StrokeKind::Inside,
                                    );
                                    painter.rect_filled(
                                        egui::Rect::from_min_size(
                                            egui::pos2(rect.left() + 14.0, rect.top() + 18.0),
                                            egui::Vec2::new(card * 0.42, 7.0),
                                        ),
                                        egui::CornerRadius::same(3),
                                        t.border_strong,
                                    );
                                    painter.rect_filled(
                                        egui::Rect::from_min_size(
                                            egui::pos2(rect.left() + 14.0, rect.top() + 36.0),
                                            egui::Vec2::new(card * 0.72, 6.0),
                                        ),
                                        egui::CornerRadius::same(3),
                                        t.border_subtle.gamma_multiply(pulse),
                                    );
                                    painter.rect_filled(
                                        egui::Rect::from_min_size(
                                            egui::pos2(rect.left() + 14.0, rect.top() + 50.0),
                                            egui::Vec2::new(card * 0.55, 6.0),
                                        ),
                                        egui::CornerRadius::same(3),
                                        t.border_subtle.gamma_multiply(pulse),
                                    );
                                    if index < 2 {
                                        ui.add_space(gap);
                                    }
                                }
                            });

                            ui.add_space(theme::SPACE_8);
                            let progress = (self.started.elapsed().as_secs_f32()
                                / DEFAULT_MODULE_TIMEOUT.as_secs_f32())
                            .mul_add(0.95, 0.0)
                            .clamp(0.0, 0.95);
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::ProgressBar::new(progress)
                                        .desired_width(width - 52.0)
                                        .desired_height(6.0)
                                        .corner_radius(egui::CornerRadius::same(3))
                                        .fill(t.accent),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{:.0}%",
                                                progress * 100.0
                                            ))
                                            .size(theme::FONT_CAPTION)
                                            .color(t.text_secondary),
                                        );
                                    },
                                );
                            });
                            ui.add_space(theme::SPACE_1);
                            ui.label(
                                egui::RichText::new(self.i18n.t("loading.hint"))
                                    .size(theme::FONT_CAPTION)
                                    .color(t.text_tertiary),
                            );
                            if let Some(error) = &self.startup_error {
                                ui.add_space(theme::SPACE_3);
                                ui.label(
                                    egui::RichText::new(error)
                                        .size(theme::FONT_BODY)
                                        .color(t.err),
                                );
                            }
                        });
                    });
                });
                ui.ctx().request_repaint();
            });
    }

    fn export_window(&mut self, ctx: &egui::Context) {
        if !self.export_open {
            return;
        }
        let mut open = self.export_open;
        egui::Window::new(self.i18n.t("export.title"))
            .open(&mut open)
            .default_width(720.0)
            .default_height(520.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for format in ExportFormat::ALL {
                        if ui
                            .selectable_label(
                                self.export_format == format,
                                self.i18n.t(format.label_key()),
                            )
                            .clicked()
                            && self.export_format != format
                        {
                            self.export_format = format;
                            self.refresh_export();
                        }
                    }
                    ui.separator();
                    if ui
                        .add(crate::ui::widgets::ghost_button(self.i18n.t("app.copy")))
                        .clicked()
                    {
                        ui.ctx().copy_text(self.export_preview.clone());
                    }
                    if ui
                        .add(crate::ui::widgets::primary_button(
                            self.i18n.t("common.save"),
                        ))
                        .clicked()
                    {
                        self.save_export();
                    }
                });
                ui.add_space(6.0);
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let mut preview = self.export_preview.as_str();
                        ui.add(
                            egui::TextEdit::multiline(&mut preview)
                                .code_editor()
                                .desired_width(f32::INFINITY)
                                .interactive(false),
                        );
                    });
            });
        self.export_open = open;
    }

    fn permission_banner(&mut self, ui: &mut egui::Ui) {
        if self.state.permission.elevated || !self.state.ui.permission_notice_open {
            return;
        }
        let t = ui::theme::theme(ui);
        egui::Panel::top("permission-banner")
            .frame(
                egui::Frame::new()
                    .fill(t.canvas)
                    .inner_margin(egui::Margin::symmetric(
                        theme::SPACE_6 as i8,
                        theme::SPACE_2 as i8,
                    )),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // 左侧 3 px 警示条 + 图标，避免整段黄字像报错横幅。
                    let (bar, _) =
                        ui.allocate_exact_size(egui::Vec2::new(3.0, 16.0), egui::Sense::hover());
                    ui.painter()
                        .rect_filled(bar, egui::CornerRadius::same(2), t.warn);
                    ui.add_space(theme::SPACE_2);
                    ui::theme::icon_widget(ui, theme::Icon::Warning, 14.0);
                    ui.label(
                        egui::RichText::new(self.i18n.t("permission.standard_hint"))
                            .size(theme::FONT_CAPTION)
                            .strong()
                            .color(t.text_primary),
                    );
                    if !self.state.permission.items.is_empty() {
                        ui.label(
                            egui::RichText::new(self.state.permission.items.join("  ·  "))
                                .size(theme::FONT_CAPTION)
                                .color(t.text_tertiary),
                        );
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui::widgets::icon_button(
                            ui,
                            theme::Icon::Close,
                            self.i18n.t("app.close"),
                        )
                        .clicked()
                        {
                            self.state.ui.permission_notice_open = false;
                        }
                    });
                });
            });
    }

    /// The overflow menu behind the More button: a small floating surface.
    fn more_window(&mut self, ctx: &egui::Context) {
        if !self.more_open {
            return;
        }
        let t = ui::theme::Theme::of(ctx.theme() == egui::Theme::Dark);
        let mut window_open = true;
        let mut close_now = false;
        egui::Window::new(self.i18n.t("shell.more"))
            .open(&mut window_open)
            .resizable(false)
            .collapsible(false)
            .title_bar(false)
            .anchor(egui::Align2::RIGHT_TOP, [-16.0, 64.0])
            .frame(t.overlay_frame())
            .show(ctx, |ui| {
                ui.set_min_width(180.0);
                if ui.button(self.i18n.t("nav.export")).clicked() {
                    self.open_export();
                    close_now = true;
                }
                if ui.button(self.i18n.t("nav.snapshots")).clicked() {
                    self.state.settings.page = crate::state::Page::Snapshots;
                    self.state.mark_dirty();
                    close_now = true;
                }
                if ui.button(self.i18n.t("nav.settings")).clicked() {
                    self.state.settings.page = crate::state::Page::Settings;
                    self.state.mark_dirty();
                    close_now = true;
                }
                if ui.button(self.i18n.t("nav.updates")).clicked() {
                    self.state.settings.page = crate::state::Page::Updates;
                    self.state.mark_dirty();
                    close_now = true;
                }
            });
        self.more_open = window_open && !close_now;
    }

    fn root_ui(&mut self, ui: &mut egui::Ui) {
        let t = ui::theme::theme(ui);
        let sidebar_width = ui::sidebar::width(ui.ctx());
        egui::Panel::left("device-viewer-sidebar")
            .resizable(false)
            .exact_size(sidebar_width)
            .frame(
                egui::Frame::new()
                    .fill(t.canvas)
                    .inner_margin(egui::Margin::symmetric(
                        theme::SPACE_2 as i8,
                        theme::SPACE_2 as i8,
                    ))
                    .stroke(egui::Stroke::new(theme::BORDER_WIDTH, t.border_subtle)),
            )
            .show(ui, |ui| {
                ui::sidebar::show(ui, &mut self.state, self.i18n);
            });
        egui::Panel::top("device-viewer-toolbar")
            .frame(
                egui::Frame::new()
                    .fill(t.canvas)
                    .inner_margin(egui::Margin::symmetric(
                        theme::SPACE_6 as i8,
                        theme::SPACE_3 as i8,
                    ))
                    .stroke(egui::Stroke::new(theme::BORDER_WIDTH, t.border_subtle)),
            )
            .show(ui, |ui| {
                self.header(ui);
            });
        self.permission_banner(ui);
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(t.canvas)
                    .inner_margin(egui::Margin::symmetric(
                        theme::SPACE_6 as i8,
                        theme::SPACE_3 as i8,
                    )),
            )
            .show(ui, |ui| {
                // 页面级滚动由 ui::show_page 统一提供，根层不再套第二层。
                ui::show_page(ui, &mut self.state, self.i18n);
            });

        // 切换页面只影响显示，绝不再触发整轮采集：17 个模块的开销与
        // 用户想看的页面无关。
    }

    fn save_window_rect(&mut self, ctx: &egui::Context) {
        if let Some(rect) = ctx.input(|input| input.viewport().outer_rect) {
            let values = [rect.min.x, rect.min.y, rect.width(), rect.height()];
            let valid = values.iter().all(|value| value.is_finite())
                && (320.0..=16384.0).contains(&values[2])
                && (240.0..=16384.0).contains(&values[3])
                && values[0].abs() <= 100_000.0
                && values[1].abs() <= 100_000.0;
            if !valid {
                return;
            }
            let changed = self
                .last_window_rect
                .map(|previous| {
                    previous
                        .iter()
                        .zip(values.iter())
                        .any(|(left, right)| (left - right).abs() > 1.0)
                })
                .unwrap_or(true);
            if changed {
                self.state.settings.window_rect = Some(values);
                self.last_window_rect = Some(values);
                self.state.mark_dirty();
            }
        }
    }
}

impl eframe::App for DeviceViewerApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.i18n.set_preference(self.state.settings.language);
        self.apply_theme(&ctx);
        self.poll_events();
        self.poll_gpu();
        self.refresh_opengl(frame);

        if self.state.ui.request_refresh {
            self.state.ui.request_refresh = false;
            self.start_query();
        }
        if self.state.ui.request_update_check {
            self.state.ui.request_update_check = false;
            self.start_update_check();
        }

        if self.state.snapshot.captured_at_unix_ms == 0 && self.busy {
            self.loading_page(ui);
        } else {
            self.root_ui(ui);
        }

        let refresh_shortcut =
            ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::R));
        let export_shortcut = ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::E));
        let palette_shortcut =
            ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::K));
        let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        if refresh_shortcut {
            self.start_query();
        }
        if export_shortcut {
            self.open_export();
        }
        if palette_shortcut {
            self.palette_open = true;
            self.palette_selection = 0;
            self.palette_query.clear();
            self.palette_focus = true;
        }
        if escape {
            self.more_open = false;
            self.export_open = false;
            self.palette_open = false;
            self.state.ui.snapshot_menu = None;
            self.state.ui.snapshot_menu_pos = None;
        }

        self.palette_window(&ctx);
        self.more_window(&ctx);
        self.export_window(&ctx);
        self.state.ui.prune_toasts();
        if !self.state.ui.toasts.is_empty() {
            let t = ui::theme::Theme::of(ctx.theme() == egui::Theme::Dark);
            let mut jump = None;
            egui::Area::new(egui::Id::unique("toasts"))
                .anchor(
                    egui::Align2::RIGHT_BOTTOM,
                    [-theme::SPACE_6, -theme::SPACE_6],
                )
                .show(&ctx, |ui| {
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                        for toast in self.state.ui.toasts.iter().rev() {
                            let (icon, color) = match toast.kind {
                                crate::state::ToastKind::Info => (theme::Icon::Info, t.info),
                                crate::state::ToastKind::Success => (theme::Icon::Check, t.ok),
                                crate::state::ToastKind::Warning => (theme::Icon::Warning, t.warn),
                                crate::state::ToastKind::Error => (theme::Icon::Warning, t.err),
                            };
                            let response = t
                                .overlay_frame()
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let (rect, _) = ui.allocate_exact_size(
                                            egui::vec2(16.0, 16.0),
                                            egui::Sense::hover(),
                                        );
                                        ui::theme::paint_icon(ui.painter(), rect, icon, color);
                                        ui.label(
                                            egui::RichText::new(&toast.text)
                                                .size(theme::FONT_BODY)
                                                .color(t.text_primary),
                                        );
                                    });
                                })
                                .response;
                            // 带来源页的提示可点击直达，避免用户只知道“保存了”却找不到。
                            if toast.page.is_some() {
                                let response = response.interact(egui::Sense::click());
                                if response.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if response.clicked() {
                                    jump = toast.page;
                                }
                            }
                            ui.add_space(theme::SPACE_2);
                        }
                    });
                });
            if let Some(page) = jump {
                self.state.settings.page = page;
                self.state.mark_dirty();
                self.state.ui.toasts.clear();
            }
        }

        self.save_window_rect(&ctx);
        self.state.save_if_due();
        ctx.request_repaint_after(Duration::from_millis(250));
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        self.state.save_config();
    }

    fn on_exit(&mut self, _gl: Option<&glow::Context>) {
        self.cancel.cancel();
        self.gpu_cancel.cancel();
        self.state.save_config();
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_timeout(Duration::from_secs(2));
        }
    }
}

#[derive(Debug, Deserialize)]
struct ReleaseDto {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    published_at: Option<String>,
}

#[derive(Debug)]
struct Release {
    tag: String,
    version: Version,
    name: String,
    url: String,
    prerelease: bool,
    published_at: Option<String>,
}

impl Release {
    fn from_dto(dto: ReleaseDto) -> Option<Self> {
        if dto.draft || dto.tag_name.trim().is_empty() {
            return None;
        }
        let version = parse_version(&dto.tag_name)?;
        let tag = dto.tag_name.trim().to_string();
        Some(Self {
            name: dto
                .name
                .filter(|name| !name.trim().is_empty())
                .unwrap_or_else(|| tag.clone()),
            version,
            url: if dto.html_url.trim().is_empty() {
                format!("https://github.com/dieraun/Device-Viewer/releases/tag/{tag}")
            } else {
                dto.html_url
            },
            prerelease: dto.prerelease,
            published_at: dto.published_at,
            tag,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Version {
    numbers: Vec<u64>,
    pre: Option<String>,
}

fn parse_version(value: &str) -> Option<Version> {
    let value = value.trim();
    let value = value
        .strip_prefix('v')
        .or_else(|| value.strip_prefix('V'))
        .unwrap_or(value);
    let value = value.split('+').next().unwrap_or(value);
    let (core, pre) = match value.split_once('-') {
        Some((core, pre)) if !pre.is_empty() => (core, Some(pre.to_ascii_lowercase())),
        Some((core, _)) => (core, None),
        None => (value, None),
    };
    let numbers = core
        .split('.')
        .map(|part| part.parse::<u64>().ok())
        .collect::<Option<Vec<_>>>()?;
    if numbers.is_empty() {
        return None;
    }
    Some(Version { numbers, pre })
}

fn compare_versions(left: &Version, right: &Version) -> Ordering {
    let len = left.numbers.len().max(right.numbers.len());
    for index in 0..len {
        let left = left.numbers.get(index).copied().unwrap_or(0);
        let right = right.numbers.get(index).copied().unwrap_or(0);
        match left.cmp(&right) {
            Ordering::Equal => {}
            other => return other,
        }
    }
    match (&left.pre, &right.pre) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(left), Some(right)) => compare_pre(left, right),
    }
}

fn compare_pre(left: &str, right: &str) -> Ordering {
    let mut left_parts = left.split('.');
    let mut right_parts = right.split('.');
    loop {
        match (left_parts.next(), right_parts.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(left), Some(right)) => match compare_identifier(left, right) {
                Ordering::Equal => {}
                other => return other,
            },
        }
    }
}

fn compare_identifier(left: &str, right: &str) -> Ordering {
    match (left.parse::<u64>(), right.parse::<u64>()) {
        (Ok(left), Ok(right)) => left.cmp(&right),
        (Ok(_), Err(_)) => Ordering::Less,
        (Err(_), Ok(_)) => Ordering::Greater,
        (Err(_), Err(_)) => left.cmp(right),
    }
}

fn check_for_update(current: &str, include_prerelease: bool) -> Result<UpdateReport, String> {
    let current_version =
        parse_version(current).ok_or_else(|| format!("invalid current version `{current}`"))?;
    let release = if include_prerelease {
        let releases: Vec<ReleaseDto> =
            get_json(&format!("{UPDATE_ENDPOINT}?per_page=30"), UPDATE_USER_AGENT)?;
        releases
            .into_iter()
            .filter_map(Release::from_dto)
            .max_by(|left, right| compare_versions(&left.version, &right.version))
            .ok_or_else(|| "no published release was found".to_string())?
    } else {
        let dto: ReleaseDto = get_json(&format!("{UPDATE_ENDPOINT}/latest"), UPDATE_USER_AGENT)?;
        Release::from_dto(dto)
            .ok_or_else(|| "the latest release has no valid version tag".to_string())?
    };

    let available = compare_versions(&release.version, &current_version) == Ordering::Greater;
    Ok(UpdateReport {
        current: current.to_string(),
        latest: release.tag.trim_start_matches(['v', 'V']).to_string(),
        tag: release.tag,
        name: release.name,
        url: release.url,
        prerelease: release.prerelease,
        available,
        published_at: release.published_at,
        checked_at_ms: now_ms(),
    })
}

fn get_json<T: serde::de::DeserializeOwned>(url: &str, user_agent: &str) -> Result<T, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout_read(Duration::from_secs(12))
        .build();
    let response = agent
        .get(url)
        .set("Accept", "application/vnd.github+json")
        .set(
            "User-Agent",
            &format!("{user_agent}/{}", env!("CARGO_PKG_VERSION")),
        )
        .call();
    match response {
        Ok(response) => response
            .into_string()
            .map_err(|error| format!("could not read the GitHub response: {error}"))
            .and_then(|body| {
                serde_json::from_str(&body)
                    .map_err(|error| format!("could not parse the GitHub response: {error}"))
            }),
        Err(ureq::Error::Status(404, _)) => {
            Err("no published release was found for this repository".to_string())
        }
        Err(ureq::Error::Status(403 | 429, _)) => {
            Err("GitHub API rate limit reached; please try again later".to_string())
        }
        Err(ureq::Error::Status(code, _)) => Err(format!("the GitHub API returned HTTP {code}")),
        Err(ureq::Error::Transport(error)) => Err(format!("network request failed: {error}")),
    }
}

/// 自动检查更新的最小间隔：GitHub 未认证 API 的限额是每小时 60 次。
const UPDATE_CHECK_INTERVAL_MS: u64 = 24 * 60 * 60 * 1000;

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(value: &str) -> Version {
        parse_version(value).expect("valid version")
    }

    #[test]
    fn version_comparison_handles_tags_and_releases() {
        assert_eq!(
            compare_versions(&parse("v1.0.1"), &parse("1.0.1")),
            Ordering::Equal
        );
        assert_eq!(
            compare_versions(&parse("1.0.1"), &parse("1.0.2")),
            Ordering::Less
        );
        assert_eq!(
            compare_versions(&parse("1.0"), &parse("1.0.0")),
            Ordering::Equal
        );
        assert_eq!(
            compare_versions(&parse("1.0.0"), &parse("1.0.0-rc.1")),
            Ordering::Greater
        );
    }

    #[test]
    fn version_comparison_orders_prereleases_semantically() {
        assert_eq!(
            compare_versions(&parse("1.0.0-alpha"), &parse("1.0.0-beta")),
            Ordering::Less
        );
        assert_eq!(
            compare_versions(&parse("1.0.0-rc.2"), &parse("1.0.0-rc.10")),
            Ordering::Less
        );
        assert_eq!(
            compare_versions(&parse("1.0.0-1"), &parse("1.0.0-alpha")),
            Ordering::Less
        );
    }

    #[test]
    fn version_parser_rejects_non_versions() {
        assert!(parse_version("latest").is_none());
        assert!(parse_version("").is_none());
        assert!(parse_version("v1.x.0").is_none());
    }
}
