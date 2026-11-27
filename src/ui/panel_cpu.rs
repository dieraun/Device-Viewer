//! CPU topology and live per-core load.

use egui::{Align2, RichText, Sense, Vec2};

use crate::device_query::format_frequency;
use crate::export::markdown_table;
use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::theme;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    let Some(cpu) = state.snapshot.cpu.clone() else {
        widgets::empty(ui, i18n.t("overview.no_data"));
        return;
    };
    let t = theme::theme(ui);

    let frequency = cpu
        .cores
        .iter()
        .map(|core| core.frequency_mhz)
        .max()
        .map(format_frequency)
        .unwrap_or_else(|| i18n.t("app.n_a").to_string());
    let load = cpu.load_average;
    widgets::metric_grid(
        ui,
        &[
            (
                i18n.t("common.cores"),
                format!("{} / {}", cpu.physical_cores, cpu.logical_cores),
                Some(format!(
                    "{} {} · {} {}",
                    cpu.physical_cores,
                    i18n.t("cpu.physical"),
                    cpu.logical_cores,
                    i18n.t("cpu.logical")
                )),
            ),
            (
                i18n.t("common.usage"),
                format!("{:.1}%", cpu.global_usage_percent),
                None,
            ),
            (i18n.t("common.frequency"), frequency, None),
            (
                i18n.t("common.load_average"),
                load.map(|load| format!("{:.2}", load[0]))
                    .unwrap_or_else(|| i18n.t("app.n_a").to_string()),
                load.map(|load| format!("{:.2} / {:.2}", load[1], load[2])),
            ),
        ],
    );

    widgets::section(ui, i18n.t("cpu.identity"));
    widgets::kv_grid(
        ui,
        "cpu-identity",
        &[
            (i18n.t("common.vendor"), cpu.vendor.clone()),
            (i18n.t("common.model"), cpu.brand.clone()),
            (i18n.t("common.architecture"), cpu.architecture.clone()),
        ],
    );

    widgets::section(ui, i18n.t("common.threads"));
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!(
                "{} · {}",
                cpu.cores.len(),
                i18n.t("common.threads")
            ))
            .size(theme::FONT_CAPTION)
            .color(t.text_tertiary),
        );
        let rows: Vec<Vec<String>> = cpu
            .cores
            .iter()
            .map(|core| {
                vec![
                    core.id.to_string(),
                    format!("{:.0}%", core.usage_percent),
                    format_frequency(core.frequency_mhz),
                ]
            })
            .collect();
        let markdown = markdown_table(
            &["#", i18n.t("common.usage"), i18n.t("common.frequency")],
            &rows,
        );
        widgets::copy_table_button(ui, i18n, markdown);
    });
    ui.add_space(theme::SPACE_2);

    if cpu.cores.len() > 16 {
        // Two waterfalls keep the list short on many-core machines.
        let middle = cpu.cores.len().div_ceil(2);
        let gap = theme::SPACE_6;
        let column_width = ((ui.available_width() - gap) / 2.0).max(200.0);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(column_width, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| core_column(ui, &cpu.cores[..middle], i18n, &t),
            );
            ui.add_space(gap);
            ui.allocate_ui_with_layout(
                Vec2::new(column_width, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| core_column(ui, &cpu.cores[middle..], i18n, &t),
            );
        });
    } else {
        core_column(ui, &cpu.cores, i18n, &t);
    }

    widgets::warnings(ui, i18n, "cpu-warnings", &cpu.warnings);
}

fn core_column(
    ui: &mut egui::Ui,
    cores: &[crate::device_query::cpu::CpuCore],
    i18n: I18n,
    t: &theme::Theme,
) {
    let width = ui.available_width();
    let id_width = 32.0;
    let usage_width = 48.0;
    let frequency_width = 72.0;
    let bar_left = id_width + theme::SPACE_2;
    let bar_width = (width - bar_left - usage_width - frequency_width - theme::SPACE_3).max(20.0);

    let (header, _) = ui.allocate_exact_size(Vec2::new(width, 20.0), Sense::hover());
    let caption = egui::FontId::monospace(theme::FONT_CAPTION);
    ui.painter().text(
        egui::pos2(header.left(), header.center().y),
        Align2::LEFT_CENTER,
        "#",
        caption.clone(),
        t.text_tertiary,
    );
    ui.painter().text(
        egui::pos2(
            header.right() - frequency_width - theme::SPACE_3,
            header.center().y,
        ),
        Align2::RIGHT_CENTER,
        i18n.t("common.usage"),
        caption.clone(),
        t.text_tertiary,
    );
    ui.painter().text(
        egui::pos2(header.right(), header.center().y),
        Align2::RIGHT_CENTER,
        i18n.t("common.frequency"),
        caption,
        t.text_tertiary,
    );
    for core in cores {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 24.0), Sense::hover());
        let painter = ui.painter();
        painter.text(
            egui::pos2(rect.left(), rect.center().y),
            Align2::LEFT_CENTER,
            core.id.to_string(),
            egui::FontId::monospace(theme::FONT_CAPTION),
            t.text_secondary,
        );
        let track = egui::Rect::from_min_size(
            egui::pos2(rect.left() + bar_left, rect.center().y - 1.0),
            Vec2::new(bar_width, 2.0),
        );
        painter.rect_filled(track, egui::CornerRadius::same(1), t.border_subtle);
        let filled = (bar_width * (core.usage_percent / 100.0).clamp(0.0, 1.0)).max(0.5);
        painter.rect_filled(
            egui::Rect::from_min_size(track.min, Vec2::new(filled, 2.0)),
            egui::CornerRadius::same(1),
            t.accent,
        );
        // Integer percentages keep the right aligned column from jittering.
        painter.text(
            egui::pos2(
                rect.right() - frequency_width - theme::SPACE_3,
                rect.center().y,
            ),
            Align2::RIGHT_CENTER,
            format!("{:.0}%", core.usage_percent),
            egui::FontId::monospace(theme::FONT_CAPTION),
            t.text_primary,
        );
        painter.text(
            egui::pos2(rect.right(), rect.center().y),
            Align2::RIGHT_CENTER,
            format_frequency(core.frequency_mhz),
            egui::FontId::monospace(theme::FONT_CAPTION),
            t.text_secondary,
        );
    }
}
