//! Temperatures, fans and voltages.

use egui::RichText;

use crate::export::markdown_table;
use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::theme;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    let Some(info) = state.snapshot.sensors.clone() else {
        widgets::empty(ui, i18n.t("overview.no_data"));
        return;
    };

    // 每个计数都附带来源说明，避免用户把 0 误读成“硬件不存在”。
    widgets::metric_grid(
        ui,
        &[
            (
                i18n.t("sensors.temperatures"),
                info.temperatures.len().to_string(),
                Some(i18n.t("sensors.temperatures_detail").to_string()),
            ),
            (
                i18n.t("sensors.fans"),
                info.fans.len().to_string(),
                Some(i18n.t("sensors.fans_detail").to_string()),
            ),
            (
                i18n.t("sensors.voltages"),
                info.voltages.len().to_string(),
                Some(i18n.t("sensors.voltages_detail").to_string()),
            ),
        ],
    );

    ui.add_space(theme::SPACE_4);
    widgets::card(ui, |ui| {
        ui.label(RichText::new(i18n.t("sensors.limits_title")).strong());
        ui.add_space(theme::SPACE_1);
        widgets::hint(ui, i18n.t("sensors.windows_limited"));
        if info.temperatures.is_empty() {
            ui.add_space(theme::SPACE_1);
            widgets::hint(ui, i18n.t("sensors.temp_none"));
        }
    });

    ui.add_space(theme::SPACE_4);
    widgets::card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "{} ({})",
                    i18n.t("sensors.all_temperatures"),
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
                            .unwrap_or_else(|| i18n.t("value.not_reported").to_string()),
                        sensor
                            .critical_celsius
                            .map(|value| format!("{value:.1} °C"))
                            .unwrap_or_else(|| i18n.t("value.not_reported").to_string()),
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
                        i18n.t("sensors.max"),
                        i18n.t("sensors.critical"),
                    ],
                    &rows,
                ),
            );
        });
        if info.temperatures.is_empty() {
            return;
        }
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
                            .unwrap_or_else(|| i18n.t("value.not_reported").to_string()),
                    ),
                    widgets::Cell::Muted(
                        sensor
                            .critical_celsius
                            .map(|value| format!("{value:.1} °C"))
                            .unwrap_or_else(|| i18n.t("value.not_reported").to_string()),
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
                i18n.t("sensors.reading"),
                i18n.t("sensors.max"),
                i18n.t("sensors.critical"),
            ],
            &[1.1, 2.2, 1.0, 1.0, 1.0],
            &rows,
            false,
        );
        if table.clicked_row.is_some() {
            state.ui.notify(i18n.t("toast.copied"));
        }
    });

    ui.add_space(theme::SPACE_4);
    widgets::two_pane(
        ui,
        |ui| {
            widgets::card(ui, |ui| {
                ui.label(
                    RichText::new(format!(
                        "{} ({})",
                        i18n.t("sensors.fans_title"),
                        info.fans.len()
                    ))
                    .strong(),
                );
                if info.fans.is_empty() {
                    widgets::hint(ui, i18n.t("sensors.fan_empty"));
                    return;
                }
                for fan in &info.fans {
                    ui.horizontal(|ui| {
                        ui.label(&fan.label);
                        ui.label(RichText::new(
                            fan.rpm
                                .map(|rpm| format!("{rpm} RPM"))
                                .unwrap_or_else(|| i18n.t("value.not_reported").to_string()),
                        ));
                    });
                }
            });
        },
        |ui| {
            widgets::card(ui, |ui| {
                ui.label(
                    RichText::new(format!(
                        "{} ({})",
                        i18n.t("sensors.voltages_title"),
                        info.voltages.len()
                    ))
                    .strong(),
                );
                if info.voltages.is_empty() {
                    widgets::hint(ui, i18n.t("sensors.voltage_empty"));
                    return;
                }
                for voltage in &info.voltages {
                    ui.horizontal(|ui| {
                        ui.label(&voltage.label);
                        ui.label(RichText::new(
                            voltage
                                .volts
                                .map(|volts| format!("{volts:.3} V"))
                                .unwrap_or_else(|| i18n.t("value.not_reported").to_string()),
                        ));
                    });
                }
            });
        },
    );

    if !info.notes.is_empty() {
        ui.add_space(theme::SPACE_2);
        for note in &info.notes {
            widgets::hint(ui, note.as_str());
        }
    }
    widgets::warnings(ui, i18n, "sensors-warnings", &info.warnings);
}
