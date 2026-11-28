//! Vulkan adapters, memory heaps and live GPU telemetry.

use egui::RichText;

use crate::device_query::format_bytes;
use crate::export::markdown_table;
use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::theme;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    let Some(info) = state.snapshot.vulkan.clone() else {
        let response = widgets::badge_response(
            ui,
            i18n.t("gpu.no_vulkan_support"),
            widgets::BadgeKind::Warning,
        );
        let _ = response.on_hover_text(i18n.t("gpu.no_vulkan_hint"));
        ui.add_space(8.0);
        widgets::hint(ui, i18n.t("gpu.no_vulkan_hint"));
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
        widgets::hint(ui, i18n.t("gpu.no_vulkan_hint"));
    }

    if info.devices.is_empty() {
        widgets::empty(ui, i18n.t("gpu.no_device"));
        return;
    }

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
            widgets::badge(
                ui,
                format!("{} {version}", i18n.t("vulkan.loader")),
                widgets::BadgeKind::Info,
            );
        }
    });
    ui.add_space(theme::SPACE_3);

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

    // 使用率在 Windows 上来自操作系统计数器而不是 Vulkan 本身；
    // 读不到时给出原因，而不是一个孤零零的“不适用”。
    let utilization_detail = if utilization.is_some() {
        i18n.t("gpu.runtime_source").to_string()
    } else {
        i18n.t("overview.gpu_usage_missing_detail").to_string()
    };
    widgets::metric_grid(
        ui,
        &[
            (
                i18n.t("gpu.utilization"),
                utilization
                    .map(|value| format!("{value:.1}%"))
                    .unwrap_or_else(|| i18n.t("value.not_reported").to_string()),
                Some(utilization_detail),
            ),
            (
                i18n.t("gpu.vram"),
                used.map(|value| format_bytes(value, units))
                    .unwrap_or_else(|| i18n.t("value.not_reported").to_string()),
                Some(format!(
                    "{}: {}",
                    i18n.t("common.total"),
                    format_bytes(device.total_memory_bytes, units)
                )),
            ),
            (
                i18n.t("vulkan.api_version"),
                device.api_version.clone(),
                Some(i18n.t("gpu.runtime_source").to_string()),
            ),
            (
                i18n.t("vulkan.vid_pid"),
                format!("{:04X}:{:04X}", device.vendor_id, device.device_id),
                Some(device.device_type.clone()),
            ),
        ],
    );

    ui.add_space(theme::SPACE_4);
    widgets::card(ui, |ui| {
        ui.label(RichText::new(i18n.t("vulkan.adapter_detail")).strong());
        ui.add_space(theme::SPACE_1);
        widgets::kv_grid(
            ui,
            "vulkan-adapter-details",
            &[
                (i18n.t("common.device"), device.name.clone()),
                (i18n.t("common.type"), device.device_type.clone()),
                (i18n.t("vulkan.api_version"), device.api_version.clone()),
                (
                    i18n.t("vulkan.driver_version"),
                    device.driver_version.clone(),
                ),
                (
                    i18n.t("vulkan.vid_pid"),
                    format!("{:04X}:{:04X}", device.vendor_id, device.device_id),
                ),
                (
                    i18n.t("common.total"),
                    format_bytes(device.total_memory_bytes, units),
                ),
            ],
        );
        if state.gpu_runtime.is_empty() {
            ui.add_space(theme::SPACE_1);
            widgets::hint(ui, i18n.t("gpu.no_runtime"));
        }
    });

    ui.add_space(theme::SPACE_4);
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
    widgets::two_pane(
        ui,
        |ui| {
            widgets::card(ui, |ui| {
                ui.label(RichText::new(i18n.t("gpu.utilization")).strong());
                widgets::sparkline(
                    ui,
                    "gpu-utilization-spark",
                    &utilization_history,
                    100.0,
                    widgets::SparkKind::Utilization,
                    120.0,
                );
                widgets::hint(ui, i18n.t("gpu.utilization_explain"));
            });
        },
        |ui| {
            widgets::card(ui, |ui| {
                ui.label(RichText::new(i18n.t("gpu.vram")).strong());
                widgets::sparkline(
                    ui,
                    "gpu-vram-spark",
                    &vram_history,
                    100.0,
                    widgets::SparkKind::Memory,
                    120.0,
                );
            });
        },
    );

    ui.add_space(theme::SPACE_4);
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
        widgets::hint(ui, i18n.t("vulkan.heap_explain"));
        let rows: Vec<widgets::Row> = device
            .heaps
            .iter()
            .map(|heap| {
                let fraction = if device.total_memory_bytes == 0 {
                    0.0
                } else {
                    heap.size_bytes as f32 / device.total_memory_bytes as f32
                };
                widgets::Row::new(vec![
                    widgets::Cell::Muted(heap.index.to_string()),
                    widgets::Cell::Mono(format_bytes(heap.size_bytes, units)),
                    widgets::Cell::Text(
                        i18n.t(if heap.device_local {
                            "app.yes"
                        } else {
                            "app.no"
                        })
                        .to_string(),
                    ),
                    widgets::Cell::Bar(fraction),
                ])
            })
            .collect();
        let table = widgets::data_table(
            ui,
            i18n,
            "vulkan-heaps",
            &[
                "#",
                i18n.t("common.size"),
                i18n.t("common.device_local"),
                i18n.t("common.value"),
            ],
            &[0.5, 1.2, 1.2, 2.4],
            &rows,
            false,
        );
        if table.clicked_row.is_some() {
            state.ui.notify(i18n.t("toast.copied"));
        }
    });

    ui.add_space(theme::SPACE_4);
    widgets::card(ui, |ui| {
        if widgets::filter_input(ui, i18n, &mut state.settings.filters.extension_text).changed() {
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
        let visible: Vec<&String> = extensions
            .iter()
            .filter(|extension| filter.is_empty() || extension.to_lowercase().contains(&filter))
            .collect();

        ui.horizontal(|ui| {
            ui.label(RichText::new(i18n.t("common.extensions")).strong());
            widgets::badge(
                ui,
                i18n.t("vulkan.extension_count")
                    .replacen("{}", &visible.len().to_string(), 1)
                    .replacen("{}", &device.extensions.len().to_string(), 1),
                widgets::BadgeKind::Info,
            );
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
        });
        widgets::hint(ui, i18n.t("vulkan.extensions_hint"));
        if visible.is_empty() {
            widgets::hint(ui, i18n.t("vulkan.no_extensions"));
            return;
        }
        egui::ScrollArea::vertical()
            .id_salt("vulkan-ext-scroll")
            .max_height(360.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                for extension in visible {
                    let favorite = state
                        .settings
                        .favorite_extensions
                        .iter()
                        .any(|item| item == extension);
                    ui.horizontal(|ui| {
                        let icon = if favorite {
                            theme::Icon::Check
                        } else {
                            theme::Icon::Plus
                        };
                        if widgets::icon_button(ui, icon, i18n.t("gpu.favorite")).clicked() {
                            if favorite {
                                state
                                    .settings
                                    .favorite_extensions
                                    .retain(|item| item != extension);
                                state.ui.notify(i18n.t("gpu.favorite_removed"));
                            } else {
                                state.settings.favorite_extensions.push(extension.clone());
                                state.ui.notify(i18n.t("gpu.favorite_added"));
                            }
                            state.mark_dirty();
                        }
                        ui.monospace(extension);
                    });
                }
            });
    });

    widgets::warnings(ui, i18n, "vulkan-warnings", &info.warnings);
}
