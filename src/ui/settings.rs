//! Appearance, language, units, portable mode and storage locations.

use egui::RichText;

use crate::device_query::UnitSystem;
use crate::i18n::{I18n, Language};
use crate::state::{AppState, Settings};
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    widgets::page_header(ui, i18n, "nav.settings", "page.settings.sub");

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.columns(2, |columns| {
                widgets::card(&mut columns[0], |ui| {
                    ui.label(RichText::new(i18n.t("settings.appearance")).strong());
                    let mut dark = state.settings.dark;
                    if ui.checkbox(&mut dark, i18n.t("app.dark")).changed() {
                        state.settings.dark = dark;
                        state.mark_dirty();
                        apply_theme(ui.ctx(), dark);
                    }
                    ui.label(
                        RichText::new(i18n.t("settings.theme_hint"))
                            .weak()
                            .size(11.0),
                    );

                    ui.add_space(10.0);
                    ui.label(RichText::new(i18n.t("settings.language")).strong());
                    egui::ComboBox::from_id_salt("settings-language")
                        .selected_text(state.settings.language.label())
                        .show_ui(ui, |ui| {
                            for language in Language::ALL {
                                if ui
                                    .selectable_value(
                                        &mut state.settings.language,
                                        language,
                                        language.label(),
                                    )
                                    .changed()
                                {
                                    state.mark_dirty();
                                }
                            }
                        });

                    ui.add_space(10.0);
                    ui.label(RichText::new(i18n.t("settings.units")).strong());
                    let mut units = state.settings.units;
                    ui.radio_value(&mut units, UnitSystem::Binary, i18n.t("settings.binary"));
                    ui.radio_value(&mut units, UnitSystem::Decimal, i18n.t("settings.decimal"));
                    if units != state.settings.units {
                        state.settings.units = units;
                        state.mark_dirty();
                    }
                });

                widgets::card(&mut columns[0], |ui| {
                    ui.label(RichText::new(i18n.t("settings.portable")).strong());
                    let mut portable = state.settings.portable;
                    if ui
                        .checkbox(&mut portable, i18n.t("settings.portable"))
                        .changed()
                    {
                        state.set_portable(portable);
                        state.ui.toast = Some(if portable {
                            i18n.t("toast.portable_on").to_string()
                        } else {
                            i18n.t("toast.portable_off").to_string()
                        });
                    }
                    ui.label(
                        RichText::new(i18n.t("settings.portable_hint"))
                            .weak()
                            .size(11.0),
                    );
                    ui.label(
                        RichText::new(i18n.t("settings.path_hint"))
                            .weak()
                            .size(11.0),
                    );
                });

                widgets::card(&mut columns[1], |ui| {
                    ui.label(RichText::new(i18n.t("settings.storage")).strong());
                    widgets::kv_grid(
                        ui,
                        "settings-paths",
                        &[
                            (
                                i18n.t("settings.config"),
                                state.config_path.display().to_string(),
                            ),
                            (
                                i18n.t("settings.snapshots"),
                                state.snapshots_dir.display().to_string(),
                            ),
                            (
                                i18n.t("snapshot.storage"),
                                state.snapshots_dir.display().to_string(),
                            ),
                        ],
                    );
                });

                widgets::card(&mut columns[1], |ui| {
                    ui.label(RichText::new(i18n.t("settings.query")).strong());
                    ui.label(format!(
                        "{}: {} s",
                        i18n.t("settings.timeout"),
                        crate::device_query::DEFAULT_MODULE_TIMEOUT.as_secs()
                    ));
                    ui.label(
                        RichText::new(format!(
                            "{}: {}",
                            i18n.t("settings.filters"),
                            if state.settings.filters.pcie_text.is_empty() {
                                i18n.t("app.none")
                            } else {
                                "saved"
                            }
                        ))
                        .weak()
                        .size(11.0),
                    );
                    ui.label(format!(
                        "{}: {}",
                        i18n.t("settings.favorites"),
                        state.settings.favorite_extensions.len()
                    ));
                });
            });

            ui.add_space(10.0);
            widgets::card(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .button(format!("↺ {}", i18n.t("settings.reset")))
                        .clicked()
                    {
                        state.settings = Settings {
                            language: state.settings.language,
                            dark: state.settings.dark,
                            units: state.settings.units,
                            portable: state.settings.portable,
                            ..Settings::default()
                        };
                        state.mark_dirty();
                        state.ui.toast = Some(i18n.t("settings.reset_done").to_string());
                    }
                    if ui.button(i18n.t("app.save")).clicked() {
                        state.save_config();
                        state.ui.toast = Some(i18n.t("toast.config_saved").to_string());
                    }
                });
            });
        });
}

pub fn apply_theme(ctx: &egui::Context, dark: bool) {
    if dark {
        ctx.set_visuals(egui::Visuals::dark());
    } else {
        ctx.set_visuals(egui::Visuals::light());
    }
}
