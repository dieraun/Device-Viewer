//! OpenCL platforms and compute devices.

use egui::RichText;

use crate::device_query::format_bytes;
use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    widgets::page_header(ui, i18n, "nav.opencl", "page.opencl.sub");
    let Some(info) = state.snapshot.opencl.clone() else {
        widgets::empty(ui, i18n.t("overview.no_data"));
        return;
    };
    let units = state.settings.units;

    if !info.available {
        widgets::empty(ui, i18n.t("app.not_available"));
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (platform_index, platform) in info.platforms.iter().enumerate() {
                widgets::card(ui, |ui| {
                    ui.label(RichText::new(&platform.name).size(16.0).strong());
                    ui.label(
                        RichText::new(format!(
                            "{} · {} · {}",
                            platform.vendor, platform.version, platform.profile
                        ))
                        .weak()
                        .size(11.0),
                    );
                    ui.add_space(6.0);
                    for (device_index, device) in platform.devices.iter().enumerate() {
                        ui.push_id(format!("opencl-{platform_index}-{device_index}"), |ui| {
                            egui::CollapsingHeader::new(format!(
                                "{}  ·  {}",
                                device.name, device.device_type
                            ))
                            .id_salt(format!("opencl-device-{platform_index}-{device_index}"))
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    widgets::metric(
                                        ui,
                                        i18n.t("common.compute_units"),
                                        device.max_compute_units.to_string(),
                                        None,
                                    );
                                    widgets::metric(
                                        ui,
                                        i18n.t("common.frequency"),
                                        format!("{} MHz", device.max_clock_mhz),
                                        None,
                                    );
                                    widgets::metric(
                                        ui,
                                        i18n.t("common.global_memory"),
                                        format_bytes(device.global_memory_bytes, units),
                                        None,
                                    );
                                    widgets::metric(
                                        ui,
                                        i18n.t("common.local_memory"),
                                        format_bytes(device.local_memory_bytes, units),
                                        None,
                                    );
                                });
                                widgets::kv_grid(
                                    ui,
                                    &format!("opencl-grid-{platform_index}-{device_index}"),
                                    &[
                                        (i18n.t("common.vendor"), device.vendor.clone()),
                                        (i18n.t("common.driver"), device.driver_version.clone()),
                                        (i18n.t("common.version"), device.version.clone()),
                                        (
                                            i18n.t("common.opencl_c"),
                                            device.opencl_c_version.clone(),
                                        ),
                                        (
                                            i18n.t("common.work_group"),
                                            format!(
                                                "{} (items: {})",
                                                device.max_work_group_size,
                                                device
                                                    .max_work_item_sizes
                                                    .iter()
                                                    .map(|value| value.to_string())
                                                    .collect::<Vec<_>>()
                                                    .join(" × ")
                                            ),
                                        ),
                                        (
                                            i18n.t("common.compiler"),
                                            if device.compiler_available {
                                                i18n.t("app.enabled").to_string()
                                            } else {
                                                i18n.t("app.disabled").to_string()
                                            },
                                        ),
                                    ],
                                );
                                egui::CollapsingHeader::new(format!(
                                    "{} ({})",
                                    i18n.t("common.extensions"),
                                    device.extensions.len()
                                ))
                                .id_salt(format!("opencl-ext-{platform_index}-{device_index}"))
                                .show(ui, |ui| {
                                    for extension in &device.extensions {
                                        ui.monospace(extension);
                                    }
                                });
                            });
                        });
                    }
                });
                ui.add_space(8.0);
            }
        });

    widgets::warnings(ui, i18n, "opencl-warnings", &info.warnings);
}
