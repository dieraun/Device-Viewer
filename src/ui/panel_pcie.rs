//! PCI/PCIe device list with link details.

use egui::RichText;

use crate::device_query::pcie::PcieDevice;
use crate::export::markdown_table;
use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    widgets::page_header(ui, i18n, "nav.pcie", "page.pcie.sub");
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
    });

    ui.add_space(8.0);
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (index, device) in devices.iter().enumerate() {
                let title = format!(
                    "{}  ·  {:04X}:{:04X}  ·  {}",
                    device.address, device.vendor_id, device.device_id, device.class_name
                );
                egui::CollapsingHeader::new(title)
                    .id_salt(format!("pcie-{index}"))
                    .show(ui, |ui| {
                        widgets::kv_grid(
                            ui,
                            &format!("pcie-grid-{index}"),
                            &[
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
                                        (Some(vendor), Some(device)) => {
                                            format!("{vendor:04X}:{device:04X}")
                                        }
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
                            ],
                        );
                        if let Some(link) = &device.link {
                            ui.add_space(6.0);
                            ui.label(RichText::new(i18n.t("common.link")).strong());
                            widgets::kv_grid(
                                ui,
                                &format!("pcie-link-{index}"),
                                &[
                                    (
                                        i18n.t("common.generation"),
                                        link.generation.clone().unwrap_or_else(|| "-".to_string()),
                                    ),
                                    (
                                        i18n.t("common.speed"),
                                        format!(
                                            "{} / {}",
                                            link.current_speed.as_deref().unwrap_or("-"),
                                            link.max_speed.as_deref().unwrap_or("-")
                                        ),
                                    ),
                                    (
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
                                    ),
                                ],
                            );
                        }
                    });
            }
        });

    widgets::warnings(ui, i18n, "pcie-warnings", &info.warnings);
}
