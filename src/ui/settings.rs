//! Appearance, language, updates, About and storage locations.

use egui::RichText;

use crate::device_query::UnitSystem;
use crate::i18n::{I18n, LanguagePreference};
use crate::state::{AppState, Page, Settings, ThemeMode, UpdateState, format_timestamp};
use crate::ui::theme;
use crate::ui::widgets::{self, BadgeKind, badge};

const AUTHOR: &str = "dieraun";
const AUTHOR_URL: &str = "https://www.github.com/dieraun";
const REPOSITORY_URL: &str = "https://github.com/dieraun/Device-Viewer";

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    widgets::page_body(ui, |ui| {
        single_column(ui, |ui| {
            widgets::card(ui, |ui| {
                ui.label(RichText::new(i18n.t("settings.appearance")).strong());

                ui.label(
                    RichText::new(i18n.t("settings.theme"))
                        .size(theme::FONT_CAPTION)
                        .weak(),
                );
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
                                // 主题由主循环的 apply_theme 统一应用（按上次
                                // 偏好去重），这里只请求重绘，避免两条入口重复
                                // 调用 set_theme。
                                ui.ctx().request_repaint();
                            }
                        }
                    });
                ui.label(
                    RichText::new(i18n.t("settings.theme_hint"))
                        .weak()
                        .size(theme::FONT_CAPTION),
                );

                ui.add_space(theme::SECTION_GAP);
                ui.label(
                    RichText::new(i18n.t("settings.language"))
                        .size(theme::FONT_CAPTION)
                        .weak(),
                );
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
                        .size(theme::FONT_CAPTION),
                );
                // Linux 发行版不一定自带中文字体，缺字时明确提示而不是让
                // 用户看到一排方块。
                #[cfg(target_os = "linux")]
                if !crate::i18n::cjk_available() {
                    ui.label(
                        RichText::new(i18n.t("settings.font_missing"))
                            .size(theme::FONT_CAPTION)
                            .color(theme::theme(ui).err),
                    );
                }

                ui.add_space(theme::SECTION_GAP);
                ui.label(RichText::new(i18n.t("settings.units")).strong());
                let mut units = state.settings.units;
                ui.radio_value(&mut units, UnitSystem::Binary, i18n.t("settings.binary"));
                ui.radio_value(&mut units, UnitSystem::Decimal, i18n.t("settings.decimal"));
                if units != state.settings.units {
                    state.settings.units = units;
                    state.mark_dirty();
                }
            });

            widgets::card(ui, |ui| {
                ui.label(RichText::new(i18n.t("settings.portable")).strong());
                let mut portable = state.settings.portable;
                if ui
                    .checkbox(&mut portable, i18n.t("settings.portable"))
                    .changed()
                {
                    state.set_portable(portable);
                    state.ui.notify(if portable {
                        i18n.t("toast.portable_on")
                    } else {
                        i18n.t("toast.portable_off")
                    });
                }
                ui.label(
                    RichText::new(i18n.t("settings.portable_hint"))
                        .weak()
                        .size(theme::FONT_CAPTION),
                );
                ui.label(
                    RichText::new(i18n.t("settings.path_hint"))
                        .weak()
                        .size(theme::FONT_CAPTION),
                );
            });

            widgets::card(ui, |ui| {
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
                ui.label(
                    RichText::new(i18n.t("update.hint"))
                        .weak()
                        .size(theme::FONT_CAPTION),
                );
                if ui
                    .button(format!("⇧ {}", i18n.t("update.open_updates")))
                    .clicked()
                {
                    state.settings.page = Page::Updates;
                    state.mark_dirty();
                }
            });

            widgets::card(ui, |ui| {
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

        ui.add_space(theme::SECTION_GAP);
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

        ui.add_space(theme::SECTION_GAP);
        widgets::card(ui, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add(widgets::secondary_button(i18n.t("settings.reset")))
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
                    state.ui.notify(i18n.t("settings.reset_done"));
                }
                if ui
                    .add(widgets::primary_button(i18n.t("app.save")).fill(theme::theme(ui).accent))
                    .clicked()
                {
                    state.save_config();
                    state.ui.notify(i18n.t("toast.config_saved"));
                }
            });
        });
    });
}

pub fn about(ui: &mut egui::Ui, _state: &mut AppState, i18n: I18n) {
    widgets::page_body(ui, |ui| {
        single_column(ui, |ui| {
            widgets::card(ui, |ui| {
                ui.label(RichText::new(i18n.t("about.application")).strong());
                ui.add_space(theme::SPACE_1);
                ui.label(RichText::new("Device Viewer").size(theme::FONT_H1).strong());
                ui.label(
                    RichText::new(i18n.t("app.subtitle"))
                        .weak()
                        .size(theme::FONT_CAPTION),
                );
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
                        (i18n.t("about.license"), "MIT OR Apache-2.0".to_string()),
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

            widgets::card(ui, |ui| {
                ui.label(RichText::new(i18n.t("about.rust")).strong());
                ui.add_space(theme::SPACE_1);
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

            widgets::card(ui, |ui| {
                ui.label(RichText::new(i18n.t("about.runtime")).strong());
                ui.add_space(theme::SPACE_1);
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

            widgets::card(ui, |ui| {
                ui.label(RichText::new(i18n.t("about.legal")).strong());
                ui.add_space(theme::SPACE_1);
                ui.label(
                    RichText::new(i18n.t("about.legal_hint"))
                        .weak()
                        .size(theme::FONT_CAPTION),
                );
                ui.hyperlink_to(
                    i18n.t("about.license"),
                    format!("{REPOSITORY_URL}/blob/main/LICENSE-MIT"),
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

                ui.add_space(theme::SPACE_2);
                ui.label(
                    RichText::new(format!(
                        "{}  ·  yanruoxinsu2@outlook.com",
                        i18n.t("about.contact")
                    ))
                    .weak()
                    .size(theme::FONT_CAPTION),
                );
            });
        });
    });
}

pub fn updates(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    widgets::page_body(ui, |ui| {
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
            ui.label(
                RichText::new(i18n.t("update.hint"))
                    .weak()
                    .size(theme::FONT_CAPTION),
            );
        });

        ui.add_space(theme::SECTION_GAP);
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
                ui.add_space(theme::SPACE_1);
                ui.label(RichText::new(message).weak().size(theme::FONT_CAPTION));
                ui.label(
                    RichText::new(i18n.t("update.failure_hint"))
                        .weak()
                        .size(theme::FONT_CAPTION),
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
                            .size(theme::FONT_CAPTION),
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

fn single_column(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    let width = ui.available_width().min(theme::CONTENT_MAX_WIDTH);
    let pad = ((ui.available_width() - width) / 2.0).max(0.0);
    ui.horizontal_top(|ui| {
        ui.add_space(pad);
        ui.vertical(|ui| {
            ui.set_width(width);
            add(ui);
        });
    });
}

fn build_timestamp() -> String {
    option_env!("DV_BUILD_TIMESTAMP")
        .and_then(|value| value.parse::<u64>().ok())
        .map(|seconds| format_timestamp(seconds.saturating_mul(1000)))
        .unwrap_or_else(|| "unknown".to_string())
}
