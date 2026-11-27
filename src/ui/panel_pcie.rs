//! PCI/PCIe device list with link details.

use crate::device_query::pcie::PcieDevice;
use crate::export::markdown_table;
use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::theme;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    let Some(info) = state.snapshot.pcie.clone() else {
        widgets::empty(ui, i18n.t("pcie.no_devices"));
        return;
    };

    if widgets::filter_input(ui, i18n, &mut state.settings.filters.pcie_text).changed() {
        state.mark_dirty();
    }
    let filter = state.settings.filters.pcie_text.to_lowercase();
    let devices: Vec<PcieDevice> = info
        .devices
        .into_iter()
        .filter(|device| {
            filter.is_empty()
                || device.address.to_lowercase().contains(&filter)
                || device.class_name.to_lowercase().contains(&filter)
                || device
                    .vendor_name
                    .as_deref()
                    .unwrap_or_default()
                    .to_lowercase()
                    .contains(&filter)
                || device
                    .device_name
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
                    device.address.clone(),
                    format!("{:04X}:{:04X}", device.vendor_id, device.device_id),
                    device
                        .vendor_name
                        .clone()
                        .or_else(|| device.device_name.clone())
                        .unwrap_or_else(|| "-".to_string()),
                    device.class_name.clone(),
                    device.driver.clone().unwrap_or_else(|| "-".to_string()),
                ]
            })
            .collect();
        widgets::copy_table_button(
            ui,
            i18n,
            markdown_table(
                &[
                    i18n.t("common.address"),
                    "VID:PID",
                    i18n.t("common.name"),
                    i18n.t("common.class"),
                    i18n.t("common.driver"),
                ],
                &rows,
            ),
        );
        if state.ui.pcie_detail.is_some() {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(widgets::ghost_button(i18n.t("app.close"))).clicked() {
                    state.ui.pcie_detail = None;
                }
            });
        }
    });

    ui.add_space(theme::SPACE_3);
    let table_rows: Vec<widgets::Row> = devices
        .iter()
        .map(|device| {
            let name = device
                .device_name
                .clone()
                .or_else(|| device.vendor_name.clone())
                .unwrap_or_else(|| i18n.t("app.unknown").to_string());
            let vendor = device
                .vendor_name
                .clone()
                .unwrap_or_else(|| "-".to_string());
            let link = device
                .link
                .as_ref()
                .map(|link| {
                    let generation = link.generation.clone().unwrap_or_default();
                    let width = link
                        .current_width
                        .map(|width| format!("x{width}"))
                        .unwrap_or_default();
                    let speed = link.current_speed.clone().unwrap_or_default();
                    [generation, width, speed]
                        .into_iter()
                        .filter(|part| !part.is_empty())
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .filter(|link| !link.is_empty())
                .unwrap_or_else(|| "-".to_string());
            let mut row = widgets::Row::new(vec![
                widgets::Cell::Mono(device.address.clone()),
                widgets::Cell::Secondary {
                    primary: name.clone(),
                    secondary: vendor,
                },
                widgets::Cell::Mono(format!("{:04X}:{:04X}", device.vendor_id, device.device_id)),
                widgets::Cell::Text(device.class_name.clone()),
                widgets::Cell::Muted(link),
                widgets::Cell::Muted(device.driver.clone().unwrap_or_else(|| "-".to_string())),
            ]);
            row.copy = format!(
                "{}\t{}\t{:04X}:{:04X}\t{}\t{}",
                device.address,
                name,
                device.vendor_id,
                device.device_id,
                device.class_name,
                device.driver.as_deref().unwrap_or("-")
            );
            row
        })
        .collect();
    let table = widgets::data_table(
        ui,
        i18n,
        "pcie-table",
        &[
            i18n.t("common.address"),
            i18n.t("common.name"),
            "VID:PID",
            i18n.t("common.class"),
            i18n.t("common.link"),
            i18n.t("common.driver"),
        ],
        &[1.2, 2.6, 1.1, 1.5, 1.2, 1.1],
        &table_rows,
        false,
    );
    if table.clicked_row.is_some() {
        state.ui.notify(i18n.t("toast.copied"));
    }
    if let Some(index) = table.double_clicked_row {
        state.ui.pcie_detail = devices.get(index).map(|device| device.address.clone());
    }

    if let Some(address) = &state.ui.pcie_detail
        && let Some(device) = devices.iter().find(|device| &device.address == address)
    {
        ui.add_space(theme::SPACE_4);
        widgets::card(ui, |ui| {
            widgets::section(
                ui,
                &format!("{} · {}", i18n.t("common.details"), device.address),
            );
            let mut rows = vec![
                (
                    i18n.t("common.vendor"),
                    device
                        .vendor_name
                        .clone()
                        .unwrap_or_else(|| i18n.t("app.unknown").to_string()),
                ),
                (
                    i18n.t("common.model"),
                    device
                        .device_name
                        .clone()
                        .unwrap_or_else(|| i18n.t("app.unknown").to_string()),
                ),
                (
                    i18n.t("common.revision"),
                    device
                        .revision
                        .map(|value| format!("{value:02X}"))
                        .unwrap_or_else(|| "-".to_string()),
                ),
                (
                    i18n.t("common.subsystem"),
                    match (device.subsystem_vendor_id, device.subsystem_device_id) {
                        (Some(vendor), Some(device)) => format!("{vendor:04X}:{device:04X}"),
                        _ => "-".to_string(),
                    },
                ),
                (
                    i18n.t("common.class"),
                    format!(
                        "{} ({:02X}:{:02X})",
                        device.class_name, device.class_code, device.subclass_code
                    ),
                ),
                (
                    i18n.t("common.driver"),
                    device.driver.clone().unwrap_or_else(|| "-".to_string()),
                ),
            ];
            if let Some(link) = &device.link {
                rows.push((
                    i18n.t("common.generation"),
                    link.generation.clone().unwrap_or_else(|| "-".to_string()),
                ));
                rows.push((
                    i18n.t("common.speed"),
                    format!(
                        "{} / {}",
                        link.current_speed.as_deref().unwrap_or("-"),
                        link.max_speed.as_deref().unwrap_or("-")
                    ),
                ));
                rows.push((
                    i18n.t("common.width"),
                    format!(
                        "{} / {}",
                        link.current_width
                            .map(|value| format!("x{value}"))
                            .unwrap_or_else(|| "-".to_string()),
                        link.max_width
                            .map(|value| format!("x{value}"))
                            .unwrap_or_else(|| "-".to_string())
                    ),
                ));
            }
            widgets::kv_grid(ui, "pcie-detail", &rows);
        });
    }

    widgets::warnings(ui, i18n, "pcie-warnings", &info.warnings);
}
