//! Storage devices and SMART health.

use egui::RichText;

use crate::device_query::format_bytes;
use crate::export::markdown_table;
use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::theme;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    let Some(info) = state.snapshot.storage.clone() else {
        widgets::empty(ui, i18n.t("storage.no_devices"));
        return;
    };
    let units = state.settings.units;

    if info.devices.is_empty() {
        widgets::empty(ui, i18n.t("storage.no_devices"));
    }

    ui.horizontal(|ui| {
        widgets::badge(
            ui,
            format!("{}: {}", i18n.t("common.devices"), info.devices.len()),
            widgets::BadgeKind::Info,
        );
        let rows: Vec<Vec<String>> = info
            .devices
            .iter()
            .map(|device| {
                vec![
                    device
                        .mount_point
                        .clone()
                        .or_else(|| device.volume_label.clone())
                        .or_else(|| device.model.clone())
                        .unwrap_or_else(|| device.name.clone()),
                    device
                        .capacity_bytes
                        .map(|value| format_bytes(value, units))
                        .unwrap_or_else(|| "-".to_string()),
                    device.protocol.clone().unwrap_or_else(|| "-".to_string()),
                    device
                        .health
                        .as_ref()
                        .and_then(|health| health.passed)
                        .map(|passed| {
                            if passed {
                                i18n.t("storage.healthy").to_string()
                            } else {
                                i18n.t("storage.failing").to_string()
                            }
                        })
                        .unwrap_or_else(|| i18n.t("app.unknown").to_string()),
                ]
            })
            .collect();
        widgets::copy_table_button(
            ui,
            i18n,
            markdown_table(
                &[
                    i18n.t("storage.mount"),
                    i18n.t("storage.capacity"),
                    i18n.t("common.protocol"),
                    i18n.t("storage.health"),
                ],
                &rows,
            ),
        );
    });

    ui.add_space(8.0);
    widgets::page_body(ui, |ui| {
        for (index, device) in info.devices.iter().enumerate() {
            widgets::card(ui, |ui| {
                // 标题用盘符 / 挂载点，卷标（“系统”“win10”）作为副标题；
                // 之前把卷标当标题，才出现“C 盘显示 win10”的问题。
                let title = device
                    .mount_point
                    .clone()
                    .or_else(|| device.volume_label.clone())
                    .or_else(|| device.model.clone())
                    .unwrap_or_else(|| device.name.clone());
                ui.horizontal(|ui| {
                    ui.label(RichText::new(title.clone()).size(theme::FONT_H2).strong());
                    // 型号与标题相同（sysinfo 回填时就是盘符）时不再重复显示。
                    if let Some(model) = &device.model
                        && *model != title
                    {
                        ui.label(RichText::new(model).size(theme::FONT_CAPTION).weak());
                    }
                    if device.smart_available {
                        widgets::badge(ui, i18n.t("storage.smart"), widgets::BadgeKind::Info);
                    }
                    if let Some(health) = &device.health {
                        match health.passed {
                            Some(true) => widgets::badge(
                                ui,
                                i18n.t("storage.healthy"),
                                widgets::BadgeKind::Ok,
                            ),
                            Some(false) => widgets::badge(
                                ui,
                                i18n.t("storage.failing"),
                                widgets::BadgeKind::Error,
                            ),
                            None => {}
                        }
                    }
                });
                widgets::kv_grid(
                    ui,
                    &format!("storage-grid-{index}"),
                    &[
                        (
                            i18n.t("storage.mount"),
                            device
                                .mount_point
                                .clone()
                                .unwrap_or_else(|| "-".to_string()),
                        ),
                        (
                            i18n.t("storage.volume_label"),
                            device
                                .volume_label
                                .clone()
                                .unwrap_or_else(|| "-".to_string()),
                        ),
                        (
                            i18n.t("storage.model"),
                            // 没有 SMART 时采集器会把盘符当型号，这里不要把它当成型号重复展示。
                            device
                                .model
                                .clone()
                                .filter(|model| {
                                    let model = model.trim();
                                    !model.is_empty()
                                        && Some(model) != device.mount_point.as_deref()
                                        && model != device.name
                                })
                                .unwrap_or_else(|| "-".to_string()),
                        ),
                        (
                            i18n.t("common.firmware"),
                            device.firmware.clone().unwrap_or_else(|| "-".to_string()),
                        ),
                        (
                            i18n.t("common.serial"),
                            device.serial.clone().unwrap_or_else(|| "-".to_string()),
                        ),
                        (
                            i18n.t("storage.capacity"),
                            device
                                .capacity_bytes
                                .map(|value| format_bytes(value, units))
                                .unwrap_or_else(|| "-".to_string()),
                        ),
                        (
                            i18n.t("storage.available_space"),
                            device
                                .available_bytes
                                .map(|value| format_bytes(value, units))
                                .unwrap_or_else(|| "-".to_string()),
                        ),
                        (
                            i18n.t("storage.file_system"),
                            device
                                .file_system
                                .clone()
                                .unwrap_or_else(|| "-".to_string()),
                        ),
                        (
                            i18n.t("common.protocol"),
                            device.protocol.clone().unwrap_or_else(|| "-".to_string()),
                        ),
                        (
                            i18n.t("storage.type"),
                            device
                                .device_type
                                .clone()
                                .unwrap_or_else(|| "-".to_string()),
                        ),
                    ],
                );

                if let (Some(total), Some(free)) = (device.capacity_bytes, device.available_bytes)
                    && total > 0
                {
                    let used = total.saturating_sub(free) as f64 / total as f64 * 100.0;
                    ui.add_space(theme::SPACE_2);
                    widgets::percent_bar(ui, i18n.t("storage.used"), used as f32);
                }

                if let Some(health) = &device.health {
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        widgets::metric(
                            ui,
                            i18n.t("sensors.temperature"),
                            health
                                .temperature_c
                                .map(|value| format!("{value:.0} °C"))
                                .unwrap_or_else(|| "-".to_string()),
                            None,
                        );
                        widgets::metric(
                            ui,
                            i18n.t("storage.power_on"),
                            health
                                .power_on_hours
                                .map(|value| format!("{value} h"))
                                .unwrap_or_else(|| "-".to_string()),
                            None,
                        );
                        widgets::metric(
                            ui,
                            i18n.t("storage.written"),
                            health
                                .total_written_bytes
                                .map(|value| {
                                    let text = format_bytes(value, units);
                                    if health.written_bytes_estimated {
                                        format!("≈ {text}")
                                    } else {
                                        text
                                    }
                                })
                                .unwrap_or_else(|| "-".to_string()),
                            None,
                        );
                        widgets::metric(
                            ui,
                            i18n.t("storage.bad_blocks"),
                            health
                                .bad_blocks
                                .map(|value| value.to_string())
                                .unwrap_or_else(|| "-".to_string()),
                            None,
                        );
                    });
                    if let Some(used) = health.percentage_used {
                        widgets::percent_bar(ui, i18n.t("storage.used"), used as f32);
                    }
                }
                widgets::warnings(ui, i18n, "storage-device-warnings", &device.warnings);
            });
            ui.add_space(8.0);
        }
    });

    widgets::warnings(ui, i18n, "storage-warnings", &info.warnings);
}
