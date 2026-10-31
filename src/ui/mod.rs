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
pub mod widgets;

use egui::RichText;

use crate::device_query::{format_bytes, format_duration};
use crate::i18n::I18n;
use crate::state::{AppState, Page};

pub fn show_page(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    match state.settings.page {
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
    }
}

fn overview(ui: &mut egui::Ui, state: &AppState, i18n: I18n) {
    widgets::page_header(ui, i18n, "nav.overview", "page.overview.sub");
    let units = state.settings.units;
    let snapshot = &state.snapshot;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.columns(4, |columns| {
                let cpu_value = snapshot
                    .cpu
                    .as_ref()
                    .map(|cpu| format!("{:.1}%", cpu.global_usage_percent))
                    .unwrap_or_else(|| i18n.t("app.n_a").to_string());
                let cpu_detail = snapshot
                    .cpu
                    .as_ref()
                    .map(|cpu| format!("{} / {} threads", cpu.brand, cpu.logical_cores));
                widgets::metric(&mut columns[0], i18n.t("nav.cpu"), cpu_value, cpu_detail);

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
                widgets::metric(
                    &mut columns[1],
                    i18n.t("overview.memory"),
                    memory_value,
                    memory_detail,
                );

                let gpu_value = state
                    .gpu_runtime
                    .first()
                    .and_then(|gpu| gpu.utilization_percent)
                    .map(|value| format!("{value:.1}%"))
                    .unwrap_or_else(|| i18n.t("app.n_a").to_string());
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
                widgets::metric(
                    &mut columns[2],
                    i18n.t("overview.gpu_summary"),
                    gpu_value,
                    gpu_detail,
                );

                let storage_value = snapshot
                    .storage
                    .as_ref()
                    .map(|storage| format!("{}", storage.devices.len()))
                    .unwrap_or_else(|| "0".to_string());
                widgets::metric(
                    &mut columns[3],
                    i18n.t("overview.storage_summary"),
                    storage_value,
                    Some(i18n.t("common.devices").to_string()),
                );
            });

            ui.add_space(10.0);
            ui.columns(2, |columns| {
                widgets::card(&mut columns[0], |ui| {
                    ui.label(RichText::new(i18n.t("overview.memory")).strong());
                    if let Some(memory) = &snapshot.memory {
                        let ratio = if memory.total_bytes == 0 {
                            0.0
                        } else {
                            memory.used_bytes as f32 / memory.total_bytes as f32
                        };
                        ui.add(
                            egui::ProgressBar::new(ratio)
                                .text(format!(
                                    "{} / {}",
                                    format_bytes(memory.used_bytes, units),
                                    format_bytes(memory.total_bytes, units)
                                ))
                                .desired_height(14.0),
                        );
                        ui.label(
                            RichText::new(format!(
                                "{}: {}  ·  {}: {}",
                                i18n.t("common.free"),
                                format_bytes(memory.free_bytes, units),
                                "Swap",
                                format_bytes(memory.swap_used_bytes, units)
                            ))
                            .weak()
                            .size(11.0),
                        );
                    } else {
                        ui.label(i18n.t("overview.no_data"));
                    }
                });

                widgets::card(&mut columns[1], |ui| {
                    ui.label(RichText::new(i18n.t("overview.motherboard")).strong());
                    if let Some(board) = &snapshot.motherboard {
                        ui.label(
                            board
                                .vendor
                                .clone()
                                .unwrap_or_else(|| i18n.t("app.unknown").to_string()),
                        );
                        ui.label(RichText::new(board.name.clone().unwrap_or_default()).strong());
                        if let Some(bios) = &board.bios {
                            ui.label(
                                RichText::new(format!(
                                    "BIOS {} {}",
                                    bios.vendor.clone().unwrap_or_default(),
                                    bios.version.clone().unwrap_or_default()
                                ))
                                .weak()
                                .size(11.0),
                            );
                        }
                    } else {
                        ui.label(i18n.t("overview.no_data"));
                    }
                });
            });

            ui.add_space(10.0);
            ui.columns(2, |columns| {
                widgets::card(&mut columns[0], |ui| {
                    ui.label(RichText::new(i18n.t("overview.system")).strong());
                    if let Some(system) = &snapshot.system {
                        ui.label(&system.os.long_name);
                        ui.label(
                            RichText::new(format!(
                                "{} · {} · {}",
                                system.os.kernel_version,
                                system.os.architecture,
                                system.os.hostname
                            ))
                            .weak()
                            .size(11.0),
                        );
                        if let Some(uptime) = system.os.uptime_seconds {
                            ui.label(
                                RichText::new(format!(
                                    "{}: {}",
                                    i18n.t("common.uptime"),
                                    format_duration(uptime)
                                ))
                                .weak()
                                .size(11.0),
                            );
                        }
                    } else {
                        ui.label(i18n.t("overview.no_data"));
                    }
                });

                widgets::card(&mut columns[1], |ui| {
                    ui.label(RichText::new(i18n.t("overview.bandwidth")).strong());
                    if let Some(bandwidth) = &snapshot.bandwidth {
                        let rows = [
                            (i18n.t("common.read"), bandwidth.read_megabytes_per_second),
                            (i18n.t("common.write"), bandwidth.write_megabytes_per_second),
                            (i18n.t("common.copy"), bandwidth.copy_megabytes_per_second),
                        ];
                        for (label, value) in rows {
                            ui.label(format!(
                                "{label}: {}",
                                value
                                    .map(|value| format!("{value:.1} MB/s"))
                                    .unwrap_or_else(|| i18n.t("app.n_a").to_string())
                            ));
                        }
                    } else {
                        ui.label(i18n.t("overview.no_data"));
                    }
                });
            });

            if state.total_warnings() > 0 {
                ui.add_space(10.0);
                widgets::card(ui, |ui| {
                    ui.label(
                        RichText::new(format!(
                            "{} ({})",
                            i18n.t("overview.warnings"),
                            state.total_warnings()
                        ))
                        .strong(),
                    );
                    for failure in &snapshot.failures {
                        ui.label(
                            RichText::new(format!(
                                "{} · {}: {}",
                                failure.module,
                                failure.level.as_str(),
                                failure.message
                            ))
                            .weak()
                            .size(11.0),
                        );
                    }
                });
            }
        });
}
