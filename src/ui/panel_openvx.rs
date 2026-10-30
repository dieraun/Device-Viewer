//! OpenVX availability.

use egui::RichText;

use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    widgets::page_header(ui, i18n, "nav.openvx", "page.openvx.sub");
    let Some(info) = state.snapshot.openvx.clone() else {
        widgets::empty(ui, i18n.t("overview.no_data"));
        return;
    };

    widgets::card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(i18n.t("openvx.runtime")).strong());
            if info.available {
                widgets::badge(ui, i18n.t("app.available"), widgets::BadgeKind::Ok);
            } else {
                widgets::badge(ui, i18n.t("app.not_available"), widgets::BadgeKind::Warning);
            }
        });
        ui.label(
            RichText::new(if info.available {
                "A VxContext was created and released successfully; the OpenVX runtime is present."
            } else {
                "No OpenVX runtime answered; build with the `openvx` feature and install an implementation to enable this panel."
            })
            .weak(),
        );
    });

    widgets::warnings(ui, i18n, "openvx-warnings", &info.warnings);
}
