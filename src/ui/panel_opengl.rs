//! OpenGL renderer identity, limits and extensions.

use egui::RichText;

use crate::export::markdown_table;
use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::theme;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    let Some(info) = state.snapshot.opengl.clone() else {
        widgets::empty(ui, i18n.t("overview.no_data"));
        return;
    };

    if !info.available {
        widgets::badge(ui, i18n.t("app.not_available"), widgets::BadgeKind::Warning);
        ui.add_space(theme::SPACE_2);
        widgets::hint(ui, i18n.t("opengl.unavailable"));
    }

    let not_provided = || i18n.t("value.not_provided").to_string();
    widgets::card(ui, |ui| {
        widgets::kv_grid(
            ui,
            "opengl-identity",
            &[
                (
                    i18n.t("common.vendor"),
                    info.vendor.clone().unwrap_or_else(not_provided),
                ),
                (
                    i18n.t("common.renderer"),
                    info.renderer.clone().unwrap_or_else(not_provided),
                ),
                (
                    i18n.t("common.version"),
                    info.version.clone().unwrap_or_else(not_provided),
                ),
                (
                    "GLSL",
                    info.shading_language_version
                        .clone()
                        .unwrap_or_else(not_provided),
                ),
                (
                    i18n.t("common.max"),
                    info.max_texture_size
                        .map(|value| format!("{value} px texture"))
                        .unwrap_or_else(not_provided),
                ),
            ],
        );
        ui.add_space(theme::SPACE_1);
        widgets::hint(ui, i18n.t("opengl.context_explain"));
    });

    ui.add_space(theme::SPACE_3);
    widgets::card(ui, |ui| {
        ui.label(RichText::new(i18n.t("common.limits")).strong());
        ui.add_space(theme::SPACE_1);
        widgets::hint(ui, i18n.t("opengl.limits_explain"));
        if info.limits.is_empty() {
            widgets::hint(ui, i18n.t("opengl.no_limits"));
            return;
        }
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

    ui.add_space(theme::SPACE_3);
    widgets::card(ui, |ui| {
        if widgets::filter_input(ui, i18n, &mut state.settings.filters.opengl_extension_text)
            .changed()
        {
            state.mark_dirty();
        }
        let filter = state.settings.filters.opengl_extension_text.to_lowercase();
        let visible: Vec<&String> = info
            .extensions
            .iter()
            .filter(|extension| filter.is_empty() || extension.to_lowercase().contains(&filter))
            .collect();
        ui.horizontal(|ui| {
            ui.label(RichText::new(i18n.t("common.extensions")).strong());
            widgets::badge(
                ui,
                i18n.t("opengl.extension_count")
                    .replacen("{}", &visible.len().to_string(), 1)
                    .replacen("{}", &info.extensions.len().to_string(), 1),
                widgets::BadgeKind::Info,
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
        widgets::hint(ui, i18n.t("opengl.extensions_hint"));
        if visible.is_empty() {
            widgets::hint(ui, i18n.t("opengl.no_extensions"));
            return;
        }
        egui::ScrollArea::vertical()
            .id_salt("opengl-ext-scroll")
            .max_height(320.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                for extension in visible {
                    ui.monospace(extension);
                }
            });
    });

    widgets::warnings(ui, i18n, "opengl-warnings", &info.warnings);
}
