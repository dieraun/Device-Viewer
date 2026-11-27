//! USB bus and device inventory.

use crate::export::markdown_table;
use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
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
    widgets::page_body(ui, |ui| {
        let rows: Vec<widgets::Row> = devices
            .iter()
            .map(|device| {
                let product = device
                    .product
                    .clone()
                    .or_else(|| device.vendor.clone())
                    .unwrap_or_else(|| i18n.t("app.unknown").to_string());
                widgets::Row::new(vec![
                    widgets::Cell::Text(
                        device
                            .bus
                            .map(|value| value.to_string())
                            .unwrap_or_else(|| "-".to_string()),
                    ),
                    widgets::Cell::Text(
                        device
                            .address
                            .map(|value| value.to_string())
                            .unwrap_or_else(|| "-".to_string()),
                    ),
                    widgets::Cell::Secondary {
                        primary: product,
                        secondary: device.vendor.clone().unwrap_or_default(),
                    },
                    widgets::Cell::Mono(format!(
                        "{:04X}:{:04X}",
                        device.vendor_id.unwrap_or(0),
                        device.product_id.unwrap_or(0)
                    )),
                    widgets::Cell::Text(device.class.clone().unwrap_or_else(|| "-".to_string())),
                    widgets::Cell::Muted(
                        device
                            .speed_mbps
                            .map(|value| format!("{value:.1} Mb/s"))
                            .unwrap_or_else(|| "-".to_string()),
                    ),
                ])
            })
            .collect();
        let table = widgets::data_table(
            ui,
            i18n,
            "usb-table",
            &[
                i18n.t("usb.bus"),
                i18n.t("usb.port"),
                i18n.t("usb.product"),
                i18n.t("usb.vid_pid"),
                i18n.t("common.class"),
                i18n.t("common.speed"),
            ],
            &[0.7, 0.7, 2.6, 1.5, 1.4, 1.1],
            &rows,
            false,
        );
        if table.clicked_row.is_some() {
            state.ui.notify(i18n.t("toast.copied"));
        }
    });

    widgets::warnings(ui, i18n, "usb-warnings", &info.warnings);
}
