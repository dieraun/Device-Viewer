//! CPU topology and live per-core load.

use egui::RichText;

use crate::device_query::format_frequency;
use crate::export::{ExportFormat, markdown_table};
use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    widgets::page_header(ui, i18n, "nav.cpu", "page.cpu.sub");
    let Some(cpu) = state.snapshot.cpu.clone() else {
        widgets::empty(ui, i18n.t("overview.no_data"));
        return;
    };

    ui.horizontal(|ui| {
        widgets::metric(
            ui,
            i18n.t("common.cores"),
            format!("{} / {}", cpu.physical_cores, cpu.logical_cores),
            Some(format!(
                "{} physical · {} logical",
                cpu.physical_cores, cpu.logical_cores
            )),
        );
        widgets::metric(
            ui,
            i18n.t("common.usage"),
            format!("{:.1}%", cpu.global_usage_percent),
            None,
        );
        if let Some(load) = cpu.load_average {
            widgets::metric(
                ui,
                i18n.t("common.load_average"),
                format!("{:.2}", load[0]),
                Some(format!("{:.2} / {:.2}", load[1], load[2])),
            );
        }
        widgets::metric(
            ui,
            i18n.t("common.frequency"),
            cpu.cores
                .iter()
                .map(|core| core.frequency_mhz)
                .max()
                .map(format_frequency)
                .unwrap_or_else(|| i18n.t("app.n_a").to_string()),
            None,
        );
    });

    ui.add_space(10.0);
    widgets::card(ui, |ui| {
        widgets::kv_grid(
            ui,
            "cpu-identity",
            &[
                (i18n.t("common.vendor"), cpu.vendor.clone()),
                (i18n.t("common.model"), cpu.brand.clone()),
                (i18n.t("common.architecture"), cpu.architecture.clone()),
            ],
        );
    });

    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new(i18n.t("common.threads")).strong());
        let rows: Vec<Vec<String>> = cpu
            .cores
            .iter()
            .map(|core| {
                vec![
                    core.id.to_string(),
                    format!("{:.1}%", core.usage_percent),
                    format_frequency(core.frequency_mhz),
                ]
            })
            .collect();
        let markdown = markdown_table(
            &["#", i18n.t("common.usage"), i18n.t("common.frequency")],
            &rows,
        );
        widgets::copy_table_button(ui, i18n, markdown);
        if ui.button(format!("⧉ {}", i18n.t("app.copy"))).clicked() {
            ui.ctx().copy_text(crate::export::render(
                &state.snapshot,
                ExportFormat::Markdown,
                i18n,
                state.settings.units,
            ));
        }
    });

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(ui.available_height() - 120.0)
        .show(ui, |ui| {
            egui::Grid::new("cpu-cores")
                .num_columns(3)
                .striped(true)
                .spacing([24.0, 5.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("#").weak());
                    ui.label(RichText::new(i18n.t("common.usage")).weak());
                    ui.label(RichText::new(i18n.t("common.frequency")).weak());
                    ui.end_row();
                    for core in &cpu.cores {
                        ui.label(core.id.to_string());
                        ui.add(
                            egui::ProgressBar::new((core.usage_percent / 100.0).clamp(0.0, 1.0))
                                .desired_width(220.0)
                                .text(format!("{:.1}%", core.usage_percent)),
                        );
                        ui.label(format_frequency(core.frequency_mhz));
                        ui.end_row();
                    }
                });
        });

    widgets::warnings(ui, i18n, "cpu-warnings", &cpu.warnings);
}
