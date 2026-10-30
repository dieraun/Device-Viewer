//! AI accelerators and their runtime libraries.

use egui::RichText;

use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    widgets::page_header(ui, i18n, "nav.npu", "page.npu.sub");
    let Some(info) = state.snapshot.npu.clone() else {
        widgets::empty(ui, i18n.t("overview.no_data"));
        return;
    };

    if info.devices.is_empty() {
        widgets::empty(ui, i18n.t("overview.no_data"));
    }

    for (index, device) in info.devices.iter().enumerate() {
        widgets::card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&device.name).size(16.0).strong());
                widgets::badge(ui, &device.class_name, widgets::BadgeKind::Info);
            });
            widgets::kv_grid(
                ui,
                &format!("npu-{index}"),
                &[
                    (i18n.t("common.address"), device.address.clone()),
                    (
                        i18n.t("common.vendor"),
                        device
                            .vendor
                            .clone()
                            .unwrap_or_else(|| i18n.t("app.unknown").to_string()),
                    ),
                    (
                        "VID:PID",
                        format!("{:04X}:{:04X}", device.vendor_id, device.device_id),
                    ),
                    (
                        i18n.t("common.driver"),
                        device.driver.clone().unwrap_or_else(|| "-".to_string()),
                    ),
                ],
            );
        });
        ui.add_space(8.0);
    }

    widgets::card(ui, |ui| {
        ui.label(RichText::new(i18n.t("common.libraries")).strong());
        if info.runtime_libraries.is_empty() {
            ui.label(RichText::new(i18n.t("app.none")).weak());
        } else {
            for library in &info.runtime_libraries {
                ui.monospace(library);
            }
        }
    });

    widgets::warnings(ui, i18n, "npu-warnings", &info.warnings);
}
