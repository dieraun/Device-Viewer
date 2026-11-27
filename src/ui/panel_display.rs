//! Monitors and display adapters.

use egui::RichText;

use crate::device_query::format_bytes;
use crate::export::markdown_table;
use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::theme;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    let Some(info) = state.snapshot.display.clone() else {
        widgets::empty(ui, i18n.t("display.no_monitors"));
        return;
    };
    let units = state.settings.units;

    if info.monitors.is_empty() {
        widgets::empty(ui, i18n.t("display.no_monitors"));
    }

    ui.horizontal_wrapped(|ui| {
        for monitor in &info.monitors {
            widgets::card(ui, |ui| {
                ui.set_width(230.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(if monitor.name.is_empty() {
                            &monitor.id
                        } else {
                            &monitor.name
                        })
                        .strong(),
                    );
                    if monitor.primary {
                        widgets::badge(ui, i18n.t("display.primary"), widgets::BadgeKind::Ok);
                    } else if !monitor.primary_known {
                        // 平台没有主屏信息时，不能靠“没有徽章”暗示它是副屏。
                        widgets::badge(
                            ui,
                            i18n.t("display.primary_unknown"),
                            widgets::BadgeKind::Info,
                        );
                    }
                });
                ui.label(
                    RichText::new(format!("{} × {}", monitor.width, monitor.height))
                        .size(theme::FONT_H1)
                        .strong(),
                );
                ui.label(
                    RichText::new(match monitor.refresh_hz {
                        Some(hz) => format!("{hz} Hz"),
                        None => "-".to_string(),
                    })
                    .weak(),
                );
                // 平台不提供桌面布局时给“未知”，不要用 0, 0 冒充真实坐标。
                ui.label(
                    RichText::new(if monitor.position_known {
                        format!(
                            "{}: {}, {}",
                            i18n.t("display.position"),
                            monitor.position_x,
                            monitor.position_y
                        )
                    } else {
                        format!("{}: —", i18n.t("display.position"))
                    })
                    .weak()
                    .size(theme::FONT_CAPTION),
                );
            });
        }
    });

    ui.add_space(8.0);
    widgets::card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "{} ({})",
                    i18n.t("display.adapters"),
                    info.adapters.len()
                ))
                .strong(),
            );
            let rows: Vec<Vec<String>> = info
                .adapters
                .iter()
                .map(|adapter| {
                    vec![
                        adapter.name.clone(),
                        adapter.vendor.clone().unwrap_or_else(|| "-".to_string()),
                        adapter
                            .vram_bytes
                            .map(|value| format_bytes(value, units))
                            .unwrap_or_else(|| "-".to_string()),
                    ]
                })
                .collect();
            widgets::copy_table_button(
                ui,
                i18n,
                markdown_table(
                    &[
                        i18n.t("common.name"),
                        i18n.t("common.vendor"),
                        i18n.t("common.vram"),
                    ],
                    &rows,
                ),
            );
        });
        let rows: Vec<widgets::Row> = info
            .adapters
            .iter()
            .map(|adapter| {
                widgets::Row::new(vec![
                    widgets::Cell::Text(adapter.name.clone()),
                    widgets::Cell::Muted(adapter.vendor.clone().unwrap_or_else(|| "-".to_string())),
                    widgets::Cell::Text(
                        adapter
                            .vram_bytes
                            .map(|value| format_bytes(value, units))
                            .unwrap_or_else(|| "-".to_string()),
                    ),
                ])
            })
            .collect();
        let table = widgets::data_table(
            ui,
            i18n,
            "display-adapters",
            &[
                i18n.t("common.name"),
                i18n.t("common.vendor"),
                i18n.t("common.vram"),
            ],
            &[2.0, 1.4, 1.0],
            &rows,
            false,
        );
        if table.clicked_row.is_some() {
            state.ui.notify(i18n.t("toast.copied"));
        }
    });

    widgets::warnings(ui, i18n, "display-warnings", &info.warnings);
}
