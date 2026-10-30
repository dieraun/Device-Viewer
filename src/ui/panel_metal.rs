//! Metal devices on macOS.

use egui::RichText;

use crate::device_query::format_bytes;
use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    widgets::page_header(ui, i18n, "nav.metal", "page.metal.sub");
    let Some(info) = state.snapshot.metal.clone() else {
        widgets::empty(ui, i18n.t("overview.no_data"));
        return;
    };
    let units = state.settings.units;

    if !info.available {
        widgets::badge(ui, i18n.t("app.not_available"), widgets::BadgeKind::Warning);
        ui.add_space(6.0);
    }

    for (index, device) in info.devices.iter().enumerate() {
        widgets::card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&device.name).size(16.0).strong());
                if device.unified_memory {
                    widgets::badge(ui, i18n.t("metal.unified"), widgets::BadgeKind::Ok);
                }
            });
            widgets::kv_grid(
                ui,
                &format!("metal-{index}"),
                &[
                    ("Registry ID", format!("0x{:X}", device.registry_id)),
                    (
                        i18n.t("common.recommended_vram"),
                        format_bytes(device.recommended_max_working_set_bytes, units),
                    ),
                    (
                        i18n.t("common.threadgroup"),
                        format!(
                            "{} × {} × {}",
                            device.max_threads_per_threadgroup[0],
                            device.max_threads_per_threadgroup[1],
                            device.max_threads_per_threadgroup[2]
                        ),
                    ),
                ],
            );
        });
        ui.add_space(8.0);
    }

    widgets::warnings(ui, i18n, "metal-warnings", &info.warnings);
}
