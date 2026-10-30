//! Audio endpoints.

use egui::RichText;

use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    widgets::page_header(ui, i18n, "nav.audio", "page.audio.sub");
    let Some(info) = state.snapshot.system.clone() else {
        widgets::empty(ui, i18n.t("overview.no_data"));
        return;
    };

    if info.audio.is_empty() {
        widgets::empty(ui, i18n.t("audio.no_devices"));
    }

    for device in &info.audio {
        widgets::card(ui, |ui| {
            ui.label(RichText::new(&device.name).size(15.0).strong());
            if let Some(vendor) = &device.vendor {
                ui.label(RichText::new(vendor).weak());
            }
        });
        ui.add_space(6.0);
    }

    widgets::warnings(ui, i18n, "audio-warnings", &info.warnings);
}
