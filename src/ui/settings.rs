//! Appearance, language, updates, About and storage locations.

use egui::RichText;

use crate::device_query::UnitSystem;
use crate::i18n::{I18n, LanguagePreference};
use crate::state::{AppState, Page, Settings, ThemeMode, UpdateState, format_timestamp};
use crate::ui::widgets::{self, BadgeKind, badge};

const AUTHOR: &str = "dieraun";
const AUTHOR_URL: &str = "https://www.github.com/dieraun";
const REPOSITORY_URL: &str = "https://github.com/dieraun/Device-Viewer";

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    widgets::page_header(ui, i18n, "nav.settings", "page.settings.sub");

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.columns(2, |columns| {
                widgets::card(&mut columns[0], |ui| {
                    ui.label(RichText::new(i18n.t("settings.appearance")).strong());

                    ui.label(RichText::new(i18n.t("settings.theme")).size(11.0).weak());
                    egui::ComboBox::from_id_salt("settings-theme")
                        .selected_text(i18n.t(state.settings.theme.label_key()))
                        .show_ui(ui, |ui| {
                            for theme in ThemeMode::ALL {
                                if ui
                                    .selectable_value(
                                        &mut state.settings.theme,
                                        theme,
                                        i18n.t(theme.label_key()),
                                    )
                                    .changed()
                                {
                                    state.settings.dark = None;
                                    state.mark_dirty();
                                    apply_theme(ui.ctx(), theme);
                                }
                            }
                        });
                    ui.label(
                        RichText::new(i18n.t("settings.theme_hint"))
                            .weak()
                            .size(11.0),
                    );

                    ui.add_space(10.0);
                    ui.label(RichText::new(i18n.t("settings.language")).size(11.0).weak());
                    egui::ComboBox::from_id_salt("settings-language")
                        .selected_text(i18n.t(state.settings.language.label_key()))
                        .show_ui(ui, |ui| {
                            for language in LanguagePreference::ALL {
                                if ui
                                    .selectable_value(
                                        &mut state.settings.language,
                                        language,
                                        i18n.t(language.label_key()),
                                    )
                                    .changed()
                                {
                                    state.mark_dirty();
                                }
                            }
                        });
                    ui.label(
                        RichText::new(i18n.t("settings.language_hint"))
                            .weak()
                            .size(11.0),
                    );

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
                    ui.label(RichText::new(i18n.t("settings.updates")).strong());
                    let mut automatic = state.settings.auto_check_updates;
                    if ui.checkbox(&mut automatic, i18n.t("update.auto")).changed() {
                        state.settings.auto_check_updates = automatic;
                        state.mark_dirty();
                    }
                    let mut prerelease = state.settings.include_prerelease;
                    if ui
                        .checkbox(&mut prerelease, i18n.t("update.include_prerelease"))
                        .changed()
                    {
                        state.settings.include_prerelease = prerelease;
                        state.mark_dirty();
                    }
                    ui.label(RichText::new(i18n.t("update.hint")).weak().size(11.0));
                    if ui
                        .button(format!("⇧ {}", i18n.t("update.open_updates")))
                        .clicked()
                    {
                        state.settings.page = Page::Updates;
                        state.mark_dirty();
                    }
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
                        ],
                    );
                });
            });

            ui.add_space(10.0);
            widgets::card(ui, |ui| {
                widgets::kv_grid(
                    ui,
                    "settings-query",
                    &[
                        (
                            i18n.t("settings.timeout"),
                            format!(
                                "{} s",
                                crate::device_query::DEFAULT_MODULE_TIMEOUT.as_secs()
                            ),
                        ),
                        (
                            i18n.t("settings.filters"),
                            if state.settings.filters.pcie_text.is_empty() {
                                i18n.t("app.none").to_string()
                            } else {
                                i18n.t("app.saved").to_string()
                            },
                        ),
                        (
                            i18n.t("settings.favorites"),
                            state.settings.favorite_extensions.len().to_string(),
                        ),
                    ],
                );
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
                            theme: state.settings.theme,
                            units: state.settings.units,
                            portable: state.settings.portable,
                            auto_check_updates: state.settings.auto_check_updates,
                            include_prerelease: state.settings.include_prerelease,
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

pub fn about(ui: &mut egui::Ui, _state: &mut AppState, i18n: I18n) {
    widgets::page_header(ui, i18n, "nav.about", "page.about.sub");

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.columns(2, |columns| {
                widgets::card(&mut columns[0], |ui| {
                    ui.label(RichText::new(i18n.t("about.application")).strong());
                    ui.add_space(4.0);
                    ui.label(RichText::new("Device Viewer").size(22.0).strong());
                    ui.label(RichText::new(i18n.t("app.subtitle")).weak().size(12.0));
                    ui.add_space(8.0);
                    widgets::kv_grid(
                        ui,
                        "about-app",
                        &[
                            (i18n.t("about.author"), format!("{AUTHOR}  ·  {AUTHOR_URL}")),
                            (
                                i18n.t("about.version"),
                                format!("v{}", env!("CARGO_PKG_VERSION")),
                            ),
                            (i18n.t("about.license"), "Proprietary".to_string()),
                        ],
                    );
                    ui.horizontal(|ui| {
                        if ui
                            .link(RichText::new(i18n.t("about.github")).strong())
                            .clicked()
                        {
                            ui.ctx().open_url(egui::OpenUrl::new_tab(AUTHOR_URL));
                        }
                        if ui.link(i18n.t("about.repository")).clicked() {
                            ui.ctx().open_url(egui::OpenUrl::new_tab(REPOSITORY_URL));
                        }
                    });
                });

                widgets::card(&mut columns[0], |ui| {
                    ui.label(RichText::new(i18n.t("about.rust")).strong());
                    ui.add_space(4.0);
                    widgets::kv_grid(
                        ui,
                        "about-rust",
                        &[
                            (i18n.t("about.rustc"), env!("DV_RUSTC_VERSION").to_string()),
                            (i18n.t("about.target"), env!("DV_BUILD_TARGET").to_string()),
                            (
                                i18n.t("about.profile"),
                                env!("DV_BUILD_PROFILE").to_string(),
                            ),
                            (i18n.t("about.commit"), env!("DV_BUILD_COMMIT").to_string()),
                            (i18n.t("about.built"), build_timestamp()),
                        ],
                    );
                });

                widgets::card(&mut columns[1], |ui| {
                    ui.label(RichText::new(i18n.t("about.runtime")).strong());
                    ui.add_space(4.0);
                    widgets::kv_grid(
                        ui,
                        "about-runtime",
                        &[
                            (i18n.t("about.os"), std::env::consts::OS.to_string()),
                            (i18n.t("about.arch"), std::env::consts::ARCH.to_string()),
                            (i18n.t("about.family"), std::env::consts::FAMILY.to_string()),
                            (
                                i18n.t("about.pointer_width"),
                                format!("{}-bit", usize::BITS),
                            ),
                        ],
                    );
                });

                widgets::card(&mut columns[1], |ui| {
                    ui.label(RichText::new(i18n.t("about.legal")).strong());
                    ui.add_space(4.0);
                    ui.label(RichText::new(i18n.t("about.legal_hint")).weak().size(11.0));
                    ui.hyperlink_to(
                        i18n.t("about.license"),
                        format!("{REPOSITORY_URL}/blob/main/LICENSE"),
                    );
                    ui.hyperlink_to(
                        i18n.t("about.legal_statement"),
                        format!("{REPOSITORY_URL}/blob/main/LEGAL_STATEMENT.md"),
                    );
                    ui.hyperlink_to(
                        i18n.t("about.privacy"),
                        format!("{REPOSITORY_URL}/blob/main/PRIVACY.md"),
                    );
                    ui.hyperlink_to(
                        i18n.t("about.disclaimer"),
                        format!("{REPOSITORY_URL}/blob/main/DISCLAIMER.md"),
                    );
                });
            });

            ui.add_space(10.0);
            widgets::card(ui, |ui| {
                ui.label(
                    RichText::new(format!(
                        "{}  ·  yanruoxinsu2@outlook.com",
                        i18n.t("about.contact")
                    ))
                    .weak()
                    .size(11.0),
                );
            });
        });
}

pub fn updates(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    widgets::page_header(ui, i18n, "nav.updates", "page.updates.sub");

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            widgets::card(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "{}: v{}",
                            i18n.t("update.current"),
                            env!("CARGO_PKG_VERSION")
                        ))
                        .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let checking = state.update.is_checking();
                        if ui
                            .add_enabled(
                                !checking,
                                egui::Button::new(format!("⟳ {}", i18n.t("update.check_now"))),
                            )
                            .clicked()
                        {
                            state.ui.request_update_check = true;
                        }
                    });
                });

                ui.add_space(6.0);
                let mut automatic = state.settings.auto_check_updates;
                if ui.checkbox(&mut automatic, i18n.t("update.auto")).changed() {
                    state.settings.auto_check_updates = automatic;
                    state.mark_dirty();
                }
                let mut prerelease = state.settings.include_prerelease;
                if ui
                    .checkbox(&mut prerelease, i18n.t("update.include_prerelease"))
                    .changed()
                {
                    state.settings.include_prerelease = prerelease;
                    state.mark_dirty();
                }
                ui.label(RichText::new(i18n.t("update.hint")).weak().size(11.0));
            });

            ui.add_space(10.0);
            widgets::card(ui, |ui| match &state.update {
                UpdateState::Idle => {
                    ui.label(i18n.t("update.idle"));
                }
                UpdateState::Checking => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(i18n.t("update.checking"));
                    });
                }
                UpdateState::Failed(message) => {
                    badge(ui, i18n.t("update.failed"), BadgeKind::Error);
                    ui.add_space(4.0);
                    ui.label(RichText::new(message).weak().size(11.0));
                    ui.label(
                        RichText::new(i18n.t("update.failure_hint"))
                            .weak()
                            .size(11.0),
                    );
                }
                UpdateState::Ready(report) => {
                    ui.horizontal(|ui| {
                        if report.available {
                            badge(ui, i18n.t("update.available"), BadgeKind::Warning);
                        } else {
                            badge(ui, i18n.t("update.up_to_date"), BadgeKind::Ok);
                        }
                        if report.prerelease {
                            badge(ui, i18n.t("update.prerelease"), BadgeKind::Info);
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(format!(
                                    "{}: {}",
                                    i18n.t("update.checked"),
                                    format_timestamp(report.checked_at_ms)
                                ))
                                .weak()
                                .size(10.0),
                            );
                        });
                    });

                    ui.add_space(6.0);
                    widgets::kv_grid(
                        ui,
                        "update-report",
                        &[
                            (i18n.t("update.current"), format!("v{}", report.current)),
                            (i18n.t("update.latest"), format!("v{}", report.latest)),
                            (i18n.t("update.tag"), report.tag.clone()),
                            (i18n.t("update.name"), report.name.clone()),
                            (
                                i18n.t("update.published"),
                                report
                                    .published_at
                                    .clone()
                                    .unwrap_or_else(|| i18n.t("update.unknown").to_string()),
                            ),
                        ],
                    );

                    ui.add_space(6.0);
                    ui.hyperlink_to(i18n.t("update.open_release"), &report.url);
                }
            });
        });
}

pub fn apply_theme(ctx: &egui::Context, theme: ThemeMode) {
    ctx.set_theme(match theme {
        ThemeMode::System => egui::ThemePreference::System,
        ThemeMode::Dark => egui::ThemePreference::Dark,
        ThemeMode::Light => egui::ThemePreference::Light,
    });
}

fn build_timestamp() -> String {
    option_env!("DV_BUILD_TIMESTAMP")
        .and_then(|value| value.parse::<u64>().ok())
        .map(|seconds| format_timestamp(seconds.saturating_mul(1000)))
        .unwrap_or_else(|| "unknown".to_string())
}
