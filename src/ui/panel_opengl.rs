//! OpenGL renderer identity, limits and extensions.

use egui::RichText;

use crate::export::markdown_table;
use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    widgets::page_header(ui, i18n, "nav.opengl", "page.opengl.sub");
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
        egui::Grid::new("opengl-limits")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                for (name, value) in &info.limits {
                    ui.monospace(name);
                    ui.label(value.to_string());
                    ui.end_row();
                }
            });
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
        egui::ScrollArea::vertical()
            .max_height(260.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for extension in &info.extensions {
                    ui.monospace(extension);
                }
            });
    });

    widgets::warnings(ui, i18n, "opengl-warnings", &info.warnings);
}
