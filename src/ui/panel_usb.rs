//! USB bus and device inventory.

use egui::RichText;

use crate::export::markdown_table;
use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    widgets::page_header(ui, i18n, "nav.usb", "page.usb.sub");
    let Some(info) = state.snapshot.usb.clone() else {
        widgets::empty(ui, i18n.t("usb.no_devices"));
        return;
    };

    if widgets::filter_input(ui, i18n, &mut state.settings.filters.usb_text).changed() {
        state.mark_dirty();
    }
    let filter = state.settings.filters.usb_text.to_lowercase();
    let devices: Vec<_> = info
        .devices
        .iter()
        .filter(|device| {
            filter.is_empty()
                || device
                    .vendor
                    .as_deref()
                    .unwrap_or_default()
                    .to_lowercase()
                    .contains(&filter)
                || device
                    .product
                    .as_deref()
                    .unwrap_or_default()
                    .to_lowercase()
                    .contains(&filter)
                || device
                    .serial
                    .as_deref()
                    .unwrap_or_default()
                    .to_lowercase()
                    .contains(&filter)
        })
        .collect();

    ui.horizontal(|ui| {
        widgets::badge(
            ui,
            format!("{}: {}", i18n.t("common.devices"), devices.len()),
            widgets::BadgeKind::Info,
        );
        let rows: Vec<Vec<String>> = devices
            .iter()
            .map(|device| {
                vec![
                    device
                        .vendor
                        .clone()
                        .or_else(|| device.product.clone())
                        .unwrap_or_else(|| "-".to_string()),
                    format!(
                        "{:04X}:{:04X}",
                        device.vendor_id.unwrap_or(0),
                        device.product_id.unwrap_or(0)
                    ),
                    device.class.clone().unwrap_or_else(|| "-".to_string()),
                    device
                        .speed_mbps
                        .map(|value| format!("{value:.1} Mb/s"))
                        .unwrap_or_else(|| "-".to_string()),
                ]
            })
            .collect();
        widgets::copy_table_button(
            ui,
            i18n,
            markdown_table(
                &[
                    i18n.t("usb.product"),
                    i18n.t("usb.vid_pid"),
                    i18n.t("common.class"),
                    i18n.t("common.speed"),
                ],
                &rows,
            ),
        );
    });

    ui.add_space(8.0);
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Grid::new("usb-grid")
                .num_columns(6)
                .striped(true)
                .spacing([14.0, 5.0])
                .show(ui, |ui| {
                    for header in [
                        i18n.t("usb.bus"),
                        i18n.t("usb.port"),
                        i18n.t("usb.product"),
                        i18n.t("usb.vid_pid"),
                        i18n.t("common.class"),
                        i18n.t("common.speed"),
                    ] {
                        ui.label(RichText::new(header).weak());
                    }
                    ui.end_row();
                    for device in &devices {
                        ui.label(
                            device
                                .bus
                                .map(|value| value.to_string())
                                .unwrap_or_else(|| "-".to_string()),
                        );
                        ui.label(
                            device
                                .address
                                .map(|value| value.to_string())
                                .unwrap_or_else(|| "-".to_string()),
                        );
                        ui.vertical(|ui| {
                            ui.label(
                                device
                                    .product
                                    .clone()
                                    .or_else(|| device.vendor.clone())
                                    .unwrap_or_else(|| i18n.t("app.unknown").to_string()),
                            );
                            if let Some(manufacturer) = &device.vendor {
                                ui.label(RichText::new(manufacturer).weak().size(10.0));
                            }
                        });
                        ui.monospace(format!(
                            "{:04X}:{:04X}",
                            device.vendor_id.unwrap_or(0),
                            device.product_id.unwrap_or(0)
                        ));
                        ui.label(device.class.clone().unwrap_or_else(|| "-".to_string()));
                        ui.label(
                            device
                                .speed_mbps
                                .map(|value| format!("{value:.1} Mb/s"))
                                .unwrap_or_else(|| "-".to_string()),
                        );
                        ui.end_row();
                    }
                });
        });

    widgets::warnings(ui, i18n, "usb-warnings", &info.warnings);
}
