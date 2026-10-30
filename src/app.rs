//! eframe application shell: startup collection, navigation, toolbar and export.

use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use eframe::egui;
use tokio::runtime::Runtime;
use tokio_util::sync::CancellationToken;

use crate::device_query::vulkan::{self, GpuRuntimeStats};
use crate::device_query::{DEFAULT_MODULE_TIMEOUT, HardwareSnapshot, QueryOptions, collect_all};
use crate::export::{self, ExportFormat};
use crate::i18n::{self, I18n};
use crate::log;
use crate::state::AppState;
use crate::ui;

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
    busy: bool,
    started: Instant,
    gpu_inflight: bool,
    last_gpu_poll: Instant,
    last_gpu_sample: Instant,
    export_open: bool,
    export_format: ExportFormat,
    export_preview: String,
    startup_error: Option<String>,
    applied_theme: Option<bool>,
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
        app.apply_theme(&creation.egui_ctx);
        app.start_query();
        app
    }

    fn apply_theme(&mut self, ctx: &egui::Context) {
        let dark = self.state.settings.dark;
        if self.applied_theme != Some(dark) {
            ctx.set_visuals(if dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
            self.applied_theme = Some(dark);
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
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
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
            if ui.button(self.i18n.t("app.language")).clicked() {
                self.state.settings.language = self.state.settings.language.toggle();
                self.state.mark_dirty();
            }
            if ui
                .button(if self.state.settings.dark {
                    "☾"
                } else {
                    "☀"
                })
                .clicked()
            {
                self.state.settings.dark = !self.state.settings.dark;
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
        egui::Panel::left("device-viewer-sidebar")
            .resizable(false)
            .exact_size(224.0)
            .show(ui, |ui| {
                ui::sidebar::show(ui, &mut self.state, self.i18n);
            });
        egui::Panel::top("device-viewer-toolbar").show(ui, |ui| {
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
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.i18n.set_language(self.state.settings.language);
        self.apply_theme(&ctx);
        self.poll_events();
        self.poll_gpu();

        if self.state.ui.request_refresh {
            self.state.ui.request_refresh = false;
            self.start_query();
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
