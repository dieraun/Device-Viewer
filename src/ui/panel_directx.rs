//! DirectX / DXGI adapters.

use egui::RichText;

use crate::device_query::format_bytes;
use crate::export::markdown_table;
use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    widgets::page_header(ui, i18n, "nav.directx", "page.directx.sub");
    let Some(info) = state.snapshot.directx.clone() else {
        widgets::empty(ui, i18n.t("overview.no_data"));
        return;
    };
    let units = state.settings.units;

    if !info.available {
        widgets::badge(ui, i18n.t("app.not_available"), widgets::BadgeKind::Warning);
        ui.add_space(6.0);
    }

    let rows: Vec<Vec<String>> = info
        .adapters
        .iter()
        .map(|adapter| {
            vec![
                adapter.name.clone(),
                format!("{:04X}:{:04X}", adapter.vendor_id, adapter.device_id),
                format_bytes(adapter.dedicated_video_memory, units),
                format_bytes(adapter.shared_system_memory, units),
            ]
        })
        .collect();
    ui.horizontal(|ui| {
        widgets::badge(
            ui,
            format!("{}: {}", i18n.t("common.adapters"), info.adapters.len()),
            widgets::BadgeKind::Info,
        );
        widgets::copy_table_button(
            ui,
            i18n,
            markdown_table(
                &[
                    i18n.t("common.name"),
                    i18n.t("common.vid_pid"),
                    i18n.t("common.vram"),
                    i18n.t("common.shared_memory"),
                ],
                &rows,
            ),
        );
    });

    ui.add_space(8.0);
    for (index, adapter) in info.adapters.iter().enumerate() {
        widgets::card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&adapter.name).size(16.0).strong());
                if adapter.software {
                    widgets::badge(ui, "Software", widgets::BadgeKind::Warning);
                }
                if adapter.remote {
                    widgets::badge(ui, "Remote", widgets::BadgeKind::Warning);
                }
            });
            widgets::kv_grid(
                ui,
                &format!("dxgi-{index}"),
                &[
                    (
                        "VID:PID",
                        format!("{:04X}:{:04X}", adapter.vendor_id, adapter.device_id),
                    ),
                    (
                        i18n.t("common.vram"),
                        format_bytes(adapter.dedicated_video_memory, units),
                    ),
                    (
                        i18n.t("common.shared_memory"),
                        format_bytes(adapter.shared_system_memory, units),
                    ),
                    (
                        i18n.t("common.system_memory"),
                        format_bytes(adapter.dedicated_system_memory, units),
                    ),
                ],
            );
        });
        ui.add_space(8.0);
    }

    widgets::warnings(ui, i18n, "directx-warnings", &info.warnings);
}
