//! OpenGL renderer identity, limits and extensions.

use egui::RichText;

use crate::export::markdown_table;
use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    let Some(info) = state.snapshot.opengl.clone() else {
        widgets::empty(ui, i18n.t("overview.no_data"));
        return;
    };

    if !info.available {
        widgets::badge(ui, i18n.t("app.not_available"), widgets::BadgeKind::Warning);
        ui.add_space(6.0);
        ui.label(
            RichText::new(
                "OpenGL is reported by the renderer after a context exists; the collector cannot create one headlessly.",
            )
            .weak(),
        );
    }

    widgets::card(ui, |ui| {
        widgets::kv_grid(
            ui,
            "opengl-identity",
            &[
                (
                    i18n.t("common.vendor"),
                    info.vendor.clone().unwrap_or_else(|| "-".to_string()),
                ),
                (
                    "Renderer",
                    info.renderer.clone().unwrap_or_else(|| "-".to_string()),
                ),
                (
                    i18n.t("common.version"),
                    info.version.clone().unwrap_or_else(|| "-".to_string()),
                ),
                (
                    "GLSL",
                    info.shading_language_version
                        .clone()
                        .unwrap_or_else(|| "-".to_string()),
                ),
                (
                    i18n.t("common.max"),
                    info.max_texture_size
                        .map(|value| format!("{value} px texture"))
                        .unwrap_or_else(|| "-".to_string()),
                ),
            ],
        );
    });

    ui.add_space(8.0);
    widgets::card(ui, |ui| {
        ui.label(RichText::new("Limits").strong());
        let rows: Vec<widgets::Row> = info
            .limits
            .iter()
            .map(|(name, value)| {
                widgets::Row::new(vec![
                    widgets::Cell::Mono(name.clone()),
                    widgets::Cell::Text(value.to_string()),
                ])
            })
            .collect();
        let table = widgets::data_table(
            ui,
            i18n,
            "opengl-limits",
            &[i18n.t("common.name"), i18n.t("common.value")],
            &[2.6, 1.0],
            &rows,
            false,
        );
        if table.clicked_row.is_some() {
            state.ui.notify(i18n.t("toast.copied"));
        }
    });

    ui.add_space(8.0);
    widgets::card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "{} ({})",
                    i18n.t("common.extensions"),
                    info.extensions.len()
                ))
                .strong(),
            );
            let rows: Vec<Vec<String>> = info
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
        for extension in &info.extensions {
            ui.monospace(extension);
        }
    });

    widgets::warnings(ui, i18n, "opengl-warnings", &info.warnings);
}
