//! Temperatures, fans and voltages.

use egui::RichText;

use crate::export::markdown_table;
use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
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
        let rows: Vec<widgets::Row> = info
            .temperatures
            .iter()
            .map(|sensor| {
                widgets::Row::new(vec![
                    widgets::Cell::Text(sensor.kind.clone()),
                    widgets::Cell::Text(sensor.label.clone()),
                    widgets::Cell::Mono(format!("{:.1} °C", sensor.celsius)),
                    widgets::Cell::Muted(
                        sensor
                            .max_celsius
                            .map(|value| format!("{value:.1} °C"))
                            .unwrap_or_else(|| "-".to_string()),
                    ),
                ])
            })
            .collect();
        let table = widgets::data_table(
            ui,
            i18n,
            "sensor-temps",
            &[
                i18n.t("common.kind"),
                i18n.t("common.label"),
                i18n.t("sensors.temperature"),
                i18n.t("common.max"),
            ],
            &[1.2, 2.4, 1.0, 1.0],
            &rows,
            false,
        );
        if table.clicked_row.is_some() {
            state.ui.notify(i18n.t("toast.copied"));
        }
    });

    ui.add_space(8.0);
    widgets::two_pane(
        ui,
        |ui| {
            widgets::card(ui, |ui| {
                ui.label(
                    RichText::new(format!("{} ({})", i18n.t("sensors.fans"), info.fans.len()))
                        .strong(),
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
        },
        |ui| {
            widgets::card(ui, |ui| {
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
        },
    );

    for note in &info.notes {
        ui.label(RichText::new(note.as_str()).weak());
    }
    widgets::warnings(ui, i18n, "sensors-warnings", &info.warnings);
}
