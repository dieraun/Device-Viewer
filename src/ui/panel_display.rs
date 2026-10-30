//! Monitors and display adapters.

use egui::RichText;

use crate::device_query::format_bytes;
use crate::export::markdown_table;
use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    widgets::page_header(ui, i18n, "nav.display", "page.display.sub");
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
                    }
                });
                ui.label(
                    RichText::new(format!("{} × {}", monitor.width, monitor.height))
                        .size(20.0)
                        .strong(),
                );
                ui.label(
                    RichText::new(match monitor.refresh_hz {
                        Some(hz) => format!("{hz} Hz"),
                        None => "-".to_string(),
                    })
                    .weak(),
                );
                ui.label(
                    RichText::new(format!(
                        "{}: {}, {}",
                        i18n.t("display.position"),
                        monitor.position_x,
                        monitor.position_y
                    ))
                    .weak()
                    .size(11.0),
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
        egui::Grid::new("display-adapters")
            .num_columns(3)
            .striped(true)
            .show(ui, |ui| {
                for header in [
                    i18n.t("common.name"),
                    i18n.t("common.vendor"),
                    i18n.t("common.vram"),
                ] {
                    ui.label(RichText::new(header).weak());
                }
                ui.end_row();
                for adapter in &info.adapters {
                    ui.label(&adapter.name);
                    ui.label(adapter.vendor.clone().unwrap_or_else(|| "-".to_string()));
                    ui.label(
                        adapter
                            .vram_bytes
                            .map(|value| format_bytes(value, units))
                            .unwrap_or_else(|| "-".to_string()),
                    );
                    ui.end_row();
                }
            });
    });

    widgets::warnings(ui, i18n, "display-warnings", &info.warnings);
}
