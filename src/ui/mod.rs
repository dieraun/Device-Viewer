//! Panel components. Every function here consumes an already collected
//! [`crate::device_query::HardwareSnapshot`]; no hardware query is issued.

pub mod panel_audio;
pub mod panel_checklist;
pub mod panel_cpu;
pub mod panel_dev;
pub mod panel_directx;
pub mod panel_display;
pub mod panel_metal;
pub mod panel_network;
pub mod panel_npu;
pub mod panel_opencl;
pub mod panel_opengl;
pub mod panel_openvx;
pub mod panel_pcie;
pub mod panel_sensors;
pub mod panel_storage;
pub mod panel_usb;
pub mod panel_vulkan;
pub mod settings;
pub mod sidebar;
pub mod snapshot_compare;
pub mod snapshot_list;
pub mod theme;
pub mod widgets;

use crate::device_query::{format_bytes, format_duration};
use crate::i18n::I18n;
use crate::state::{AppState, Page};

pub fn show_page(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    // 标题由顶栏吸顶显示；这里只画当前分区的页签，再画唯一的一层滚动。
    let section = sidebar::section_of(state.settings.page);
    let failures: Vec<(String, String)> = state
        .snapshot
        .failures
        .iter()
        .map(|failure| (failure.module.to_string(), failure.message.clone()))
        .collect();
    let total_warnings = state.total_warnings();
    let mut selected = None;
    ui.horizontal(|ui| {
        if section.pages.len() > 1 {
            selected = widgets::tabs(ui, i18n, section.pages, state.settings.page);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            widgets::warnings_chip(ui, i18n, &failures, total_warnings);
        });
    });
    if let Some(page) = selected {
        state.settings.page = page;
        state.mark_dirty();
    }
    ui.add_space(theme::SPACE_2);
    egui::ScrollArea::vertical()
        .id_salt("page-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| match state.settings.page {
            Page::Overview => overview(ui, state, i18n),
            Page::Cpu => panel_cpu::show(ui, state, i18n),
            Page::Pcie => panel_pcie::show(ui, state, i18n),
            Page::Vulkan => panel_vulkan::show(ui, state, i18n),
            Page::OpenGl => panel_opengl::show(ui, state, i18n),
            Page::OpenCl => panel_opencl::show(ui, state, i18n),
            Page::Npu => panel_npu::show(ui, state, i18n),
            Page::Usb => panel_usb::show(ui, state, i18n),
            Page::Storage => panel_storage::show(ui, state, i18n),
            Page::Display => panel_display::show(ui, state, i18n),
            Page::DirectX => panel_directx::show(ui, state, i18n),
            Page::Metal => panel_metal::show(ui, state, i18n),
            Page::OpenVx => panel_openvx::show(ui, state, i18n),
            Page::Sensors => panel_sensors::show(ui, state, i18n),
            Page::Network => panel_network::show(ui, state, i18n),
            Page::Audio => panel_audio::show(ui, state, i18n),
            Page::Checklist => panel_checklist::show(ui, state, i18n),
            Page::Dev => panel_dev::show(ui, state, i18n),
            Page::Snapshots => snapshot_list::show(ui, state, i18n),
            Page::Compare => snapshot_compare::show(ui, state, i18n),
            Page::About => settings::about(ui, state, i18n),
            Page::Updates => settings::updates(ui, state, i18n),
            Page::Settings => settings::show(ui, state, i18n),
        });
}

fn overview(ui: &mut egui::Ui, state: &AppState, i18n: I18n) {
    let units = state.settings.units;
    let snapshot = &state.snapshot;
    let t = theme::theme(ui);

    let cpu_value = snapshot
        .cpu
        .as_ref()
        .map(|cpu| format!("{:.1}%", cpu.global_usage_percent))
        .unwrap_or_else(|| i18n.t("app.n_a").to_string());
    let cpu_detail = snapshot
        .cpu
        .as_ref()
        .map(|cpu| format!("{} / {} threads", cpu.brand, cpu.logical_cores));

    let (memory_value, memory_detail) = snapshot
        .memory
        .as_ref()
        .map(|memory| {
            (
                format_bytes(memory.used_bytes, units),
                Some(format!(
                    "{} / {}",
                    format_bytes(memory.used_bytes, units),
                    format_bytes(memory.total_bytes, units)
                )),
            )
        })
        .unwrap_or_else(|| (i18n.t("app.n_a").to_string(), None));

    let no_vulkan = snapshot
        .vulkan
        .as_ref()
        .is_none_or(|info| !info.available || info.devices.is_empty());
    let gpu_value = state
        .gpu_runtime
        .first()
        .and_then(|gpu| gpu.utilization_percent)
        .map(|value| format!("{value:.1}%"))
        .unwrap_or_else(|| {
            if no_vulkan {
                i18n.t("gpu.no_vulkan_support").to_string()
            } else {
                i18n.t("app.n_a").to_string()
            }
        });
    let gpu_detail = state
        .gpu_runtime
        .first()
        .map(|gpu| gpu.name.clone())
        .or_else(|| {
            snapshot
                .vulkan
                .as_ref()
                .and_then(|info| info.devices.first())
                .map(|device| device.name.clone())
        });

    let storage_value = snapshot
        .storage
        .as_ref()
        .map(|storage| format!("{}", storage.devices.len()))
        .unwrap_or_else(|| "0".to_string());

    widgets::metric_grid(
        ui,
        &[
            (i18n.t("nav.cpu"), cpu_value, cpu_detail),
            (i18n.t("overview.memory"), memory_value, memory_detail),
            (i18n.t("overview.gpu_summary"), gpu_value, gpu_detail),
            (
                i18n.t("overview.storage_summary"),
                storage_value,
                Some(i18n.t("common.devices").to_string()),
            ),
        ],
    );

    ui.add_space(theme::SPACE_6);
    hairline(ui, &t);

    widgets::section(ui, i18n.t("overview.memory"));
    if let Some(memory) = &snapshot.memory {
        let ratio = if memory.total_bytes == 0 {
            0.0
        } else {
            memory.used_bytes as f32 / memory.total_bytes as f32
        };
        ui.add(
            egui::ProgressBar::new(ratio)
                .desired_height(8.0)
                .corner_radius(egui::CornerRadius::same(theme::RADIUS_CONTROL))
                .fill(t.accent),
        );
        ui.add_space(theme::SPACE_2);
        widgets::kv_grid(
            ui,
            "overview-memory",
            &[
                (
                    i18n.t("common.used"),
                    format_bytes(memory.used_bytes, units),
                ),
                (
                    i18n.t("common.free"),
                    format_bytes(memory.free_bytes, units),
                ),
                (
                    i18n.t("common.total"),
                    format_bytes(memory.total_bytes, units),
                ),
                (
                    i18n.t("common.swap"),
                    format_bytes(memory.swap_used_bytes, units),
                ),
            ],
        );
    } else {
        widgets::empty(ui, i18n.t("overview.no_data"));
    }

    widgets::section(ui, i18n.t("overview.motherboard"));
    if let Some(board) = &snapshot.motherboard {
        let bios = board.bios.as_ref();
        let mut rows = vec![
            (
                i18n.t("common.vendor"),
                board.vendor.clone().unwrap_or_default(),
            ),
            (
                i18n.t("common.model"),
                board.name.clone().unwrap_or_default(),
            ),
        ];
        if let Some(bios) = bios {
            rows.push((
                i18n.t("common.bios"),
                format!(
                    "{} {}",
                    bios.vendor.clone().unwrap_or_default(),
                    bios.version.clone().unwrap_or_default()
                ),
            ));
        }
        widgets::kv_grid(ui, "overview-board", &rows);
    } else {
        widgets::empty(ui, i18n.t("overview.no_data"));
    }

    widgets::section(ui, i18n.t("overview.system"));
    if let Some(system) = &snapshot.system {
        let mut rows = vec![
            (i18n.t("common.os"), system.os.long_name.clone()),
            (i18n.t("common.kernel"), system.os.kernel_version.clone()),
            (
                i18n.t("common.architecture"),
                system.os.architecture.clone(),
            ),
            (i18n.t("common.hostname"), system.os.hostname.clone()),
        ];
        if let Some(uptime) = system.os.uptime_seconds {
            rows.push((i18n.t("common.uptime"), format_duration(uptime)));
        }
        widgets::kv_grid(ui, "overview-system", &rows);
        ui.add_space(theme::SPACE_2);
        ui.label(
            egui::RichText::new(i18n.t("overview.collect_note"))
                .size(theme::FONT_CAPTION)
                .color(theme::theme(ui).text_tertiary),
        );
    } else {
        widgets::empty(ui, i18n.t("overview.no_data"));
    }

    widgets::section(ui, i18n.t("overview.bandwidth"));
    if let Some(bandwidth) = &snapshot.bandwidth {
        let value = |speed: Option<f64>| {
            speed
                .map(|value| format!("{value:.1} MB/s"))
                .unwrap_or_else(|| i18n.t("app.n_a").to_string())
        };
        widgets::kv_grid(
            ui,
            "overview-bandwidth",
            &[
                (
                    i18n.t("common.read"),
                    value(bandwidth.read_megabytes_per_second),
                ),
                (
                    i18n.t("common.write"),
                    value(bandwidth.write_megabytes_per_second),
                ),
                (
                    i18n.t("common.copy"),
                    value(bandwidth.copy_megabytes_per_second),
                ),
            ],
        );
    } else {
        widgets::empty(ui, i18n.t("overview.no_data"));
    }
}

/// A one pixel divider between two sections.
fn hairline(ui: &mut egui::Ui, t: &theme::Theme) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(width, 1.0), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::ZERO, t.border_subtle);
}
