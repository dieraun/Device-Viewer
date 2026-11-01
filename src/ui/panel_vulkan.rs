//! Vulkan adapters, memory heaps and live GPU telemetry.

use egui::{Color32, RichText};

use crate::device_query::format_bytes;
use crate::export::markdown_table;
use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    widgets::page_header(ui, i18n, "nav.vulkan", "page.vulkan.sub");
    let Some(info) = state.snapshot.vulkan.clone() else {
        let response = widgets::badge_response(
            ui,
            i18n.t("gpu.no_vulkan_support"),
            widgets::BadgeKind::Warning,
        );
        let _ = response.on_hover_text(i18n.t("gpu.no_vulkan_hint"));
        ui.add_space(8.0);
        widgets::empty(ui, i18n.t("gpu.no_device"));
        return;
    };
    let units = state.settings.units;

    if !info.available {
        let response = widgets::badge_response(
            ui,
            i18n.t("gpu.no_vulkan_support"),
            widgets::BadgeKind::Warning,
        );
        let _ = response.on_hover_text(i18n.t("gpu.no_vulkan_hint"));
        ui.add_space(8.0);
    }

    if info.devices.is_empty() {
        widgets::empty(ui, i18n.t("gpu.no_device"));
        return;
    }

    if !info.devices.is_empty() {
        let selected = state
            .ui
            .selected_vulkan_device
            .min(info.devices.len().saturating_sub(1));
        ui.horizontal(|ui| {
            ui.label(format!("{}:", i18n.t("common.device")));
            egui::ComboBox::from_id_salt("vulkan-device")
                .selected_text(&info.devices[selected].name)
                .show_ui(ui, |ui| {
                    for (index, device) in info.devices.iter().enumerate() {
                        ui.selectable_value(
                            &mut state.ui.selected_vulkan_device,
                            index,
                            format!("{} ({})", device.name, device.device_type),
                        );
                    }
                });
            if let Some(version) = &info.loader_version {
                widgets::badge(ui, format!("Loader {version}"), widgets::BadgeKind::Info);
            }
        });
        ui.add_space(8.0);

        let device = &info.devices[selected];
        let runtime = state.gpu_runtime.get(selected);
        let utilization = runtime
            .and_then(|stats| stats.utilization_percent)
            .or_else(|| {
                state
                    .gpu_runtime
                    .first()
                    .and_then(|stats| stats.utilization_percent)
            });
        let used = runtime
            .and_then(|stats| stats.memory_used_bytes)
            .or(device.used_memory_bytes);
        ui.horizontal(|ui| {
            widgets::metric(
                ui,
                i18n.t("gpu.utilization"),
                utilization
                    .map(|value| format!("{value:.1}%"))
                    .unwrap_or_else(|| i18n.t("app.n_a").to_string()),
                None,
            );
            widgets::metric(
                ui,
                i18n.t("gpu.vram"),
                used.map(|value| format_bytes(value, units))
                    .unwrap_or_else(|| i18n.t("app.n_a").to_string()),
                Some(format!(
                    "{}: {}",
                    i18n.t("common.total"),
                    format_bytes(device.total_memory_bytes, units)
                )),
            );
            widgets::metric(
                ui,
                "API",
                device.api_version.clone(),
                Some(device.driver_version.clone()),
            );
            widgets::metric(
                ui,
                "VID:PID",
                format!("{:04X}:{:04X}", device.vendor_id, device.device_id),
                Some(device.device_type.clone()),
            );
        });

        ui.add_space(8.0);
        let utilization_history: Vec<f32> = state
            .ui
            .gpu_history
            .iter()
            .map(|sample| sample.utilization)
            .collect();
        let vram_history: Vec<f32> = state
            .ui
            .gpu_history
            .iter()
            .map(|sample| sample.vram_percent)
            .collect();
        ui.columns(2, |columns| {
            widgets::card(&mut columns[0], |ui| {
                ui.label(RichText::new(i18n.t("gpu.utilization")).strong());
                widgets::sparkline(
                    ui,
                    i18n.t("gpu.history"),
                    &utilization_history,
                    100.0,
                    Color32::from_rgb(90, 200, 255),
                    120.0,
                );
            });
            widgets::card(&mut columns[1], |ui| {
                ui.label(RichText::new(i18n.t("gpu.vram")).strong());
                widgets::sparkline(
                    ui,
                    i18n.t("gpu.history"),
                    &vram_history,
                    100.0,
                    Color32::from_rgb(255, 170, 90),
                    120.0,
                );
            });
        });
        if state.gpu_runtime.is_empty() {
            ui.label(RichText::new(i18n.t("gpu.no_runtime")).weak().size(11.0));
        }

        ui.add_space(8.0);
        widgets::card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(i18n.t("common.heaps")).strong());
                widgets::badge(
                    ui,
                    format!(
                        "{}: {}",
                        i18n.t("gpu.budget_supported"),
                        if device.memory_budget_supported {
                            i18n.t("app.yes")
                        } else {
                            i18n.t("app.no")
                        }
                    ),
                    if device.memory_budget_supported {
                        widgets::BadgeKind::Ok
                    } else {
                        widgets::BadgeKind::Warning
                    },
                );
            });
            egui::Grid::new("vulkan-heaps")
                .num_columns(4)
                .striped(true)
                .spacing([20.0, 5.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("#").weak());
                    ui.label(RichText::new(i18n.t("common.size")).weak());
                    ui.label(RichText::new(i18n.t("common.device_local")).weak());
                    ui.label(RichText::new(i18n.t("common.value")).weak());
                    ui.end_row();
                    for heap in &device.heaps {
                        ui.label(heap.index.to_string());
                        ui.label(format_bytes(heap.size_bytes, units));
                        ui.label(if heap.device_local {
                            i18n.t("app.yes")
                        } else {
                            i18n.t("app.no")
                        });
                        ui.add(
                            egui::ProgressBar::new(if device.total_memory_bytes == 0 {
                                0.0
                            } else {
                                heap.size_bytes as f32 / device.total_memory_bytes as f32
                            })
                            .desired_width(180.0),
                        );
                        ui.end_row();
                    }
                });
        });

        ui.add_space(8.0);
        widgets::card(ui, |ui| {
            ui.label(RichText::new(i18n.t("common.extensions")).strong());
            if widgets::filter_input(ui, i18n, &mut state.settings.filters.extension_text).changed()
            {
                state.mark_dirty();
            }
            let filter = state.settings.filters.extension_text.to_lowercase();
            let mut extensions = device.extensions.clone();
            extensions.sort_by_key(|extension| {
                !state
                    .settings
                    .favorite_extensions
                    .iter()
                    .any(|favorite| favorite == extension)
            });
            let rows: Vec<Vec<String>> = device
                .extensions
                .iter()
                .map(|extension| vec![extension.clone()])
                .collect();
            widgets::copy_table_button(
                ui,
                i18n,
                markdown_table(&[i18n.t("common.extensions")], &rows),
            );
            egui::ScrollArea::vertical()
                .max_height(240.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for extension in extensions {
                        if !filter.is_empty() && !extension.to_lowercase().contains(&filter) {
                            continue;
                        }
                        let favorite = state
                            .settings
                            .favorite_extensions
                            .iter()
                            .any(|item| item == &extension);
                        ui.horizontal(|ui| {
                            if ui
                                .button(if favorite { "★" } else { "☆" })
                                .on_hover_text(i18n.t("gpu.favorite"))
                                .clicked()
                            {
                                if favorite {
                                    state
                                        .settings
                                        .favorite_extensions
                                        .retain(|item| item != &extension);
                                    state.ui.toast =
                                        Some(i18n.t("gpu.favorite_removed").to_string());
                                } else {
                                    state.settings.favorite_extensions.push(extension.clone());
                                    state.ui.toast = Some(i18n.t("gpu.favorite_added").to_string());
                                }
                                state.mark_dirty();
                            }
                            ui.monospace(extension);
                        });
                    }
                });
        });
    }

    widgets::warnings(ui, i18n, "vulkan-warnings", &info.warnings);
}
