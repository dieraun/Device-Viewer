//! Temperatures, fans and voltages.

use egui::RichText;

use crate::export::markdown_table;
use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    widgets::page_header(ui, i18n, "nav.sensors", "page.sensors.sub");
    let Some(info) = state.snapshot.sensors.clone() else {
        widgets::empty(ui, i18n.t("overview.no_data"));
        return;
    };

    ui.horizontal_wrapped(|ui| {
        widgets::metric(
            ui,
            i18n.t("common.sensors"),
            format!(
                "{} / {} / {}",
                info.temperatures.len(),
                info.fans.len(),
                info.voltages.len()
            ),
            None,
        );
    });
    ui.add_space(8.0);

    widgets::card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "{} ({})",
                    i18n.t("sensors.temperatures"),
                    info.temperatures.len()
                ))
                .strong(),
            );
            let rows: Vec<Vec<String>> = info
                .temperatures
                .iter()
                .map(|sensor| {
                    vec![
                        sensor.kind.clone(),
                        sensor.label.clone(),
                        format!("{:.1} °C", sensor.celsius),
                        sensor
                            .max_celsius
                            .map(|value| format!("{value:.1} °C"))
                            .unwrap_or_else(|| "-".to_string()),
                    ]
                })
                .collect();
            widgets::copy_table_button(
                ui,
                i18n,
                markdown_table(
                    &[
                        i18n.t("common.kind"),
                        i18n.t("common.label"),
                        i18n.t("sensors.temperature"),
                        i18n.t("common.max"),
                    ],
                    &rows,
                ),
            );
        });
        egui::Grid::new("sensor-temps")
            .num_columns(4)
            .striped(true)
            .show(ui, |ui| {
                for header in [
                    i18n.t("common.kind"),
                    i18n.t("common.label"),
                    i18n.t("sensors.temperature"),
                    i18n.t("common.max"),
                ] {
                    ui.label(RichText::new(header).weak());
                }
                ui.end_row();
                for sensor in &info.temperatures {
                    ui.label(&sensor.kind);
                    ui.label(&sensor.label);
                    ui.label(format!("{:.1} °C", sensor.celsius));
                    ui.label(
                        sensor
                            .max_celsius
                            .map(|value| format!("{value:.1} °C"))
                            .unwrap_or_else(|| "-".to_string()),
                    );
                    ui.end_row();
                }
            });
    });

    ui.add_space(8.0);
    ui.columns(2, |columns| {
        widgets::card(&mut columns[0], |ui| {
            ui.label(
                RichText::new(format!("{} ({})", i18n.t("sensors.fans"), info.fans.len())).strong(),
            );
            for fan in &info.fans {
                ui.horizontal(|ui| {
                    ui.label(&fan.label);
                    ui.label(RichText::new(
                        fan.rpm
                            .map(|rpm| format!("{rpm} RPM"))
                            .unwrap_or_else(|| "-".to_string()),
                    ));
                });
            }
            if info.fans.is_empty() {
                ui.label(RichText::new(i18n.t("app.none")).weak());
            }
        });
        widgets::card(&mut columns[1], |ui| {
            ui.label(
                RichText::new(format!(
                    "{} ({})",
                    i18n.t("sensors.voltages"),
                    info.voltages.len()
                ))
                .strong(),
            );
            for voltage in &info.voltages {
                ui.horizontal(|ui| {
                    ui.label(&voltage.label);
                    ui.label(RichText::new(
                        voltage
                            .volts
                            .map(|volts| format!("{volts:.3} V"))
                            .unwrap_or_else(|| "-".to_string()),
                    ));
                });
            }
            if info.voltages.is_empty() {
                ui.label(RichText::new(i18n.t("app.none")).weak());
            }
        });
    });

    widgets::warnings(ui, i18n, "sensors-warnings", &info.warnings);
}
