//! DirectX / DXGI adapters.

use egui::RichText;

use crate::device_query::format_bytes;
use crate::device_query::vendor_map;
use crate::export::markdown_table;
use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::theme;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    let Some(info) = state.snapshot.directx.clone() else {
        widgets::empty(ui, i18n.t("overview.no_data"));
        return;
    };
    let units = state.settings.units;

    if !info.available {
        widgets::badge(ui, i18n.t("app.not_available"), widgets::BadgeKind::Warning);
        ui.add_space(theme::SPACE_2);
        widgets::hint(ui, i18n.t("directx.explain"));
    }

    let not_reported = || i18n.t("value.not_reported").to_string();
    let dxgi_detail = if info.dxgi_version.is_some() {
        i18n.t("directx.dxgi_detail")
    } else {
        i18n.t("directx.dxgi_unknown")
    };
    let runtime_detail = if info.runtime_version.is_some() {
        i18n.t("directx.runtime_detail")
    } else {
        i18n.t("directx.runtime_unknown")
    };
    widgets::metric_grid(
        ui,
        &[
            (
                i18n.t("directx.dxgi_version"),
                info.dxgi_version.clone().unwrap_or_else(not_reported),
                Some(dxgi_detail.to_string()),
            ),
            (
                i18n.t("directx.runtime_version"),
                info.runtime_version.clone().unwrap_or_else(not_reported),
                Some(runtime_detail.to_string()),
            ),
            (
                i18n.t("common.adapters"),
                info.adapters.len().to_string(),
                Some(i18n.t("directx.adapters_detail").to_string()),
            ),
        ],
    );

    ui.add_space(theme::SPACE_3);
    widgets::card(ui, |ui| {
        ui.label(RichText::new(i18n.t("directx.explain_title")).strong());
        ui.add_space(theme::SPACE_1);
        widgets::hint(ui, i18n.t("directx.explain"));
    });

    let d3d12 =
        |adapter: &crate::device_query::directx::DirectXAdapter| match adapter.d3d12_supported {
            Some(true) => i18n.t("directx.d3d12_yes").to_string(),
            Some(false) => i18n.t("directx.d3d12_no").to_string(),
            None => i18n.t("directx.d3d12_unknown").to_string(),
        };
    if info.adapters.is_empty() {
        ui.add_space(theme::SPACE_3);
        widgets::empty(ui, i18n.t("directx.no_adapters"));
        widgets::warnings(ui, i18n, "directx-warnings", &info.warnings);
        return;
    }

    let rows: Vec<Vec<String>> = info
        .adapters
        .iter()
        .map(|adapter| {
            vec![
                adapter.name.clone(),
                format!("{:04X}:{:04X}", adapter.vendor_id, adapter.device_id),
                adapter.feature_level.clone().unwrap_or_else(not_reported),
                d3d12(adapter),
                format_bytes(adapter.dedicated_video_memory, units),
                format_bytes(adapter.shared_system_memory, units),
            ]
        })
        .collect();
    ui.add_space(theme::SPACE_3);
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
                    i18n.t("directx.feature_level"),
                    i18n.t("directx.d3d12"),
                    i18n.t("common.vram"),
                    i18n.t("common.shared_memory"),
                ],
                &rows,
            ),
        );
    });

    ui.add_space(theme::SPACE_3);
    for (index, adapter) in info.adapters.iter().enumerate() {
        widgets::card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&adapter.name).size(theme::FONT_H2).strong());
                if adapter.software {
                    widgets::badge(ui, i18n.t("common.software"), widgets::BadgeKind::Warning);
                }
                if adapter.remote {
                    widgets::badge(ui, i18n.t("common.remote"), widgets::BadgeKind::Warning);
                }
            });
            if adapter.software {
                widgets::hint(ui, i18n.t("directx.software_note"));
            }
            let vendor = vendor_map::vendor_name(adapter.vendor_id as u16)
                .map(str::to_string)
                .unwrap_or_else(not_reported);
            widgets::kv_grid(
                ui,
                &format!("dxgi-{index}"),
                &[
                    (i18n.t("directx.vendor"), vendor),
                    (
                        i18n.t("common.vid_pid"),
                        format!("{:04X}:{:04X}", adapter.vendor_id, adapter.device_id),
                    ),
                    (i18n.t("directx.luid"), format!("0x{:016X}", adapter.luid)),
                    (
                        i18n.t("directx.subsystem"),
                        format!("0x{:08X}", adapter.subsystem_id),
                    ),
                    (i18n.t("directx.revision"), adapter.revision.to_string()),
                    (
                        i18n.t("directx.feature_level"),
                        adapter.feature_level.clone().unwrap_or_else(not_reported),
                    ),
                    (i18n.t("directx.d3d12"), d3d12(adapter)),
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
        ui.add_space(theme::SPACE_3);
    }

    widgets::warnings(ui, i18n, "directx-warnings", &info.warnings);
}
