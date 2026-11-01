//! eframe application shell: startup collection, navigation, toolbar and export.

use std::cmp::Ordering;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use eframe::egui;
use serde::Deserialize;
use tokio::runtime::Runtime;
use tokio_util::sync::CancellationToken;

use crate::device_query::opengl;
use crate::device_query::vulkan::{self, GpuRuntimeStats};
use crate::device_query::{DEFAULT_MODULE_TIMEOUT, HardwareSnapshot, QueryOptions, collect_all};
use crate::export::{self, ExportFormat};
use crate::i18n::{self, I18n};
use crate::log;
use crate::state::{AppState, ThemeMode, UpdateReport, UpdateState};
use crate::ui;

const UPDATE_ENDPOINT: &str = "https://api.github.com/repos/dieraun/Device-Viewer/releases";
const UPDATE_USER_AGENT: &str = "device-viewer";

enum QueryEvent {
    Snapshot {
        generation: u64,
        snapshot: HardwareSnapshot,
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
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([980.0, 640.0]),
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
    export_format: ExportFormat,
    export_preview: String,
    startup_error: Option<String>,
    applied_theme: Option<egui::ThemePreference>,
    last_window_rect: Option<[f32; 4]>,
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
            export_format: ExportFormat::Markdown,
            export_preview: String::new(),
            startup_error: None,
            applied_theme: None,
            last_window_rect: None,
        };
        crate::ui::widgets::install_style(&creation.egui_ctx);
        app.apply_theme(&creation.egui_ctx);
        app.start_query();
        if app.state.settings.auto_check_updates {
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
        runtime.spawn(async move {
            let snapshot = collect_all(&QueryOptions::default(), &token).await;
            let _ = sender.send(QueryEvent::Snapshot {
                generation,
                snapshot,
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
                    self.state.snapshot = snapshot;
                    self.state.record_gpu_sample();
                    self.last_gpu_sample = Instant::now();
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
        if self
            .state
            .snapshot
            .opengl
            .as_ref()
            .is_some_and(|info| info.available)
        {
            return;
        }
        let Some(gl) = frame.gl() else {
            return;
        };
        self.state.snapshot.opengl = Some(opengl::query_from_context(gl));
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
        let path = self.state.snapshots_dir.join(format!(
            "device-viewer-export-{timestamp}.{}",
            self.export_format.extension()
        ));
        if let Some(parent) = path.parent() {
            if let Err(error) = std::fs::create_dir_all(parent) {
                log::error(format!("cannot create export directory: {error}"));
                self.state.ui.toast = Some(format!("export failed: {error}"));
                return;
            }
        }
        match std::fs::write(&path, &self.export_preview) {
            Ok(()) => {
                self.state.ui.toast = Some(format!("Saved {}", path.display()));
                log::info(format!("exported report to {}", path.display()));
            }
            Err(error) => {
                log::error(format!("cannot write export {}: {error}", path.display()));
                self.state.ui.toast = Some(format!("export failed: {error}"));
            }
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Device Viewer").strong().size(15.0));
            ui.separator();
            if ui
                .button(format!("⟳ {}", self.i18n.t("common.refresh")))
                .clicked()
            {
                self.state.ui.request_refresh = true;
            }
            if ui
                .button(format!("⇩ {}", self.i18n.t("nav.export")))
                .clicked()
            {
                self.open_export();
            }
            let language_label = self.i18n.t(self.state.settings.language.label_key());
            if ui
                .button(language_label)
                .on_hover_text(self.i18n.t("settings.language_hint"))
                .clicked()
            {
                self.state.settings.language = self.state.settings.language.cycle();
                self.state.mark_dirty();
            }
            let theme = self.state.settings.theme;
            if ui
                .button(theme.icon())
                .on_hover_text(self.i18n.t(theme.label_key()))
                .clicked()
            {
                self.state.settings.theme = theme.cycle();
                self.state.settings.dark = None;
                self.state.mark_dirty();
            }
            if self.busy {
                ui.spinner();
                ui.label(
                    egui::RichText::new(format!(
                        "{} {:.1}s",
                        self.i18n.t("app.collecting"),
                        self.started.elapsed().as_secs_f32()
                    ))
                    .weak(),
                );
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let warnings = self.state.total_warnings();
                if warnings > 0 {
                    ui::widgets::badge(
                        ui,
                        format!("! {warnings}"),
                        ui::widgets::BadgeKind::Warning,
                    );
                }
                ui.label(
                    egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                        .weak()
                        .size(11.0),
                );
            });
        });
    }

    fn loading_page(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space((ui.available_height() * 0.18).max(40.0));
                ui.label(egui::RichText::new("◈").size(54.0));
                ui.label(egui::RichText::new("Device Viewer").size(30.0).strong());
                ui.label(
                    egui::RichText::new(self.i18n.t("loading.subtitle"))
                        .weak()
                        .size(13.0),
                );
                ui.add_space(28.0);
                let progress = (self.started.elapsed().as_secs_f32()
                    / DEFAULT_MODULE_TIMEOUT.as_secs_f32())
                .mul_add(0.95, 0.0)
                .clamp(0.0, 0.95);
                ui.add_sized(
                    [430.0, 18.0],
                    egui::ProgressBar::new(progress)
                        .text(format!("{}%", (progress * 100.0) as u32)),
                );
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new(self.i18n.t("loading.hint"))
                        .weak()
                        .size(11.0),
                );
                if let Some(error) = &self.startup_error {
                    ui.add_space(12.0);
                    ui.colored_label(egui::Color32::from_rgb(230, 90, 90), error);
                }
            });
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
                    if ui.button(self.i18n.t("app.copy")).clicked() {
                        ui.ctx().copy_text(self.export_preview.clone());
                    }
                    if ui.button(self.i18n.t("common.save")).clicked() {
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
        egui::Panel::top("permission-banner").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new(format!("! {}", self.i18n.t("permission.standard_hint")))
                        .color(egui::Color32::from_rgb(230, 185, 80)),
                );
                if !self.state.permission.items.is_empty() {
                    ui.label(
                        egui::RichText::new(self.state.permission.items.join(" · "))
                            .weak()
                            .size(11.0),
                    );
                }
                if ui.button("✕").clicked() {
                    self.state.ui.permission_notice_open = false;
                }
            });
        });
    }

    fn root_ui(&mut self, ui: &mut egui::Ui) {
        let page_before = self.state.settings.page;
        let dark = ui.visuals().dark_mode;
        let sidebar_bg = if dark {
            egui::Color32::from_rgb(13, 19, 26)
        } else {
            egui::Color32::from_rgb(238, 243, 249)
        };
        let toolbar_bg = if dark {
            egui::Color32::from_rgb(16, 22, 30)
        } else {
            egui::Color32::WHITE
        };
        let border = if dark {
            egui::Color32::from_rgb(31, 42, 55)
        } else {
            egui::Color32::from_rgb(219, 226, 235)
        };
        egui::Panel::left("device-viewer-sidebar")
            .resizable(false)
            .exact_size(224.0)
            .frame(
                egui::Frame::new()
                    .fill(sidebar_bg)
                    .inner_margin(egui::Margin::symmetric(10, 8)),
            )
            .show(ui, |ui| {
                ui::sidebar::show(ui, &mut self.state, self.i18n);
            });
        egui::Panel::top("device-viewer-toolbar")
            .frame(
                egui::Frame::new()
                    .fill(toolbar_bg)
                    .inner_margin(egui::Margin::symmetric(12, 6))
                    .stroke(egui::Stroke::new(1.0, border)),
            )
            .show(ui, |ui| {
                self.toolbar(ui);
            });
        self.permission_banner(ui);
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui::show_page(ui, &mut self.state, self.i18n);
                });
        });

        if page_before != self.state.settings.page && self.busy {
            log::info("page changed while collecting; restarting the query pass");
            self.start_query();
        }
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

        self.export_window(&ctx);
        if let Some(toast) = &self.state.ui.toast {
            egui::Area::new(egui::Id::unique("toast"))
                .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -18.0])
                .show(&ctx, |ui| {
                    egui::Frame::new()
                        .fill(egui::Color32::from_black_alpha(220))
                        .corner_radius(8)
                        .inner_margin(10)
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new(toast).color(egui::Color32::WHITE));
                        });
                });
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
