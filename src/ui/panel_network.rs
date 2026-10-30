//! Network interfaces.

use egui::RichText;

use crate::device_query::format_bytes;
use crate::export::markdown_table;
use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    widgets::page_header(ui, i18n, "nav.network", "page.network.sub");
    let Some(info) = state.snapshot.system.clone() else {
        widgets::empty(ui, i18n.t("overview.no_data"));
        return;
    };
    let units = state.settings.units;

    if info.networks.is_empty() {
        widgets::empty(ui, i18n.t("network.no_interfaces"));
    }

    let rows: Vec<Vec<String>> = info
        .networks
        .iter()
        .map(|network| {
            vec![
                network.name.clone(),
                network.mac_address.clone(),
                network.addresses.join(", "),
                format_bytes(network.received_bytes, units),
                format_bytes(network.transmitted_bytes, units),
            ]
        })
        .collect();
    widgets::copy_table_button(
        ui,
        i18n,
        markdown_table(
            &[
                i18n.t("common.name"),
                i18n.t("network.mac"),
                i18n.t("network.addresses"),
                i18n.t("network.received"),
                i18n.t("network.transmitted"),
            ],
            &rows,
        ),
    );

    ui.add_space(8.0);
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for network in &info.networks {
                widgets::card(ui, |ui| {
                    ui.label(RichText::new(&network.name).size(15.0).strong());
                    widgets::kv_grid(
                        ui,
                        &format!("net-{}", network.name),
                        &[
                            (i18n.t("network.mac"), network.mac_address.clone()),
                            (
                                i18n.t("network.addresses"),
                                if network.addresses.is_empty() {
                                    "-".to_string()
                                } else {
                                    network.addresses.join(", ")
                                },
                            ),
                            (
                                i18n.t("network.received"),
                                format_bytes(network.received_bytes, units),
                            ),
                            (
                                i18n.t("network.transmitted"),
                                format_bytes(network.transmitted_bytes, units),
                            ),
                        ],
                    );
                });
                ui.add_space(8.0);
            }
        });

    widgets::warnings(ui, i18n, "network-warnings", &info.warnings);
}
