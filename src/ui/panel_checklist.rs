//! Capability checklist across every collector.

use egui::RichText;

use crate::i18n::I18n;
use crate::state::{AppState, Page};
use crate::ui::widgets;

struct Row {
    page: Page,
    available: bool,
    detail: String,
}

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    widgets::page_header(ui, i18n, "nav.checklist", "page.checklist.sub");
    let snapshot = state.snapshot.clone();

    let mut rows = Vec::new();
    let mut push = |page: Page, available: bool, detail: String| {
        rows.push(Row {
            page,
            available,
            detail,
        })
    };

    push(
        Page::Cpu,
        snapshot.cpu.is_some(),
        snapshot
            .cpu
            .as_ref()
            .map(|cpu| format!("{} threads", cpu.logical_cores))
            .unwrap_or_else(|| "-".to_string()),
    );
    push(
        Page::Cpu,
        snapshot.memory.is_some(),
        snapshot
            .memory
            .as_ref()
            .map(|memory| format!("{} modules", memory.modules.len()))
            .unwrap_or_else(|| "-".to_string()),
    );
    push(
        Page::Pcie,
        snapshot.pcie.is_some(),
        snapshot
            .pcie
            .as_ref()
            .map(|pcie| format!("{} devices", pcie.devices.len()))
            .unwrap_or_else(|| "-".to_string()),
    );
    push(
        Page::Vulkan,
        snapshot.vulkan.as_ref().is_some_and(|info| info.available),
        snapshot
            .vulkan
            .as_ref()
            .map(|info| format!("{} devices", info.devices.len()))
            .unwrap_or_else(|| "-".to_string()),
    );
    push(
        Page::OpenGl,
        snapshot.opengl.as_ref().is_some_and(|info| info.available),
        snapshot
            .opengl
            .as_ref()
            .and_then(|info| info.renderer.clone())
            .unwrap_or_else(|| i18n.t("app.not_available").to_string()),
    );
    push(
        Page::OpenCl,
        snapshot.opencl.as_ref().is_some_and(|info| info.available),
        snapshot
            .opencl
            .as_ref()
            .map(|info| format!("{} platforms", info.platforms.len()))
            .unwrap_or_else(|| "-".to_string()),
    );
    push(
        Page::Npu,
        snapshot
            .npu
            .as_ref()
            .is_some_and(|info| !info.devices.is_empty()),
        snapshot
            .npu
            .as_ref()
            .map(|info| format!("{} devices", info.devices.len()))
            .unwrap_or_else(|| "-".to_string()),
    );
    push(
        Page::Usb,
        snapshot
            .usb
            .as_ref()
            .is_some_and(|info| !info.devices.is_empty()),
        snapshot
            .usb
            .as_ref()
            .map(|info| format!("{} devices", info.devices.len()))
            .unwrap_or_else(|| "-".to_string()),
    );
    push(
        Page::Storage,
        snapshot
            .storage
            .as_ref()
            .is_some_and(|info| !info.devices.is_empty()),
        snapshot
            .storage
            .as_ref()
            .map(|info| format!("{} devices", info.devices.len()))
            .unwrap_or_else(|| "-".to_string()),
    );
    push(
        Page::Display,
        snapshot
            .display
            .as_ref()
            .is_some_and(|info| !info.monitors.is_empty()),
        snapshot
            .display
            .as_ref()
            .map(|info| format!("{} monitors", info.monitors.len()))
            .unwrap_or_else(|| "-".to_string()),
    );
    push(
        Page::DirectX,
        snapshot.directx.as_ref().is_some_and(|info| info.available),
        snapshot
            .directx
            .as_ref()
            .map(|info| format!("{} adapters", info.adapters.len()))
            .unwrap_or_else(|| "-".to_string()),
    );
    push(
        Page::Metal,
        snapshot.metal.as_ref().is_some_and(|info| info.available),
        snapshot
            .metal
            .as_ref()
            .map(|info| format!("{} devices", info.devices.len()))
            .unwrap_or_else(|| "-".to_string()),
    );
    push(
        Page::OpenVx,
        snapshot.openvx.as_ref().is_some_and(|info| info.available),
        snapshot
            .openvx
            .as_ref()
            .map(|info| {
                if info.available {
                    i18n.t("app.available").to_string()
                } else {
                    i18n.t("app.not_available").to_string()
                }
            })
            .unwrap_or_else(|| "-".to_string()),
    );
    push(
        Page::Sensors,
        snapshot
            .sensors
            .as_ref()
            .is_some_and(|info| !info.temperatures.is_empty()),
        snapshot
            .sensors
            .as_ref()
            .map(|info| format!("{} sensors", info.temperatures.len()))
            .unwrap_or_else(|| "-".to_string()),
    );
    push(
        Page::Network,
        snapshot
            .system
            .as_ref()
            .is_some_and(|info| !info.networks.is_empty()),
        snapshot
            .system
            .as_ref()
            .map(|info| format!("{} interfaces", info.networks.len()))
            .unwrap_or_else(|| "-".to_string()),
    );
    push(
        Page::Audio,
        snapshot
            .system
            .as_ref()
            .is_some_and(|info| !info.audio.is_empty()),
        snapshot
            .system
            .as_ref()
            .map(|info| format!("{} endpoints", info.audio.len()))
            .unwrap_or_else(|| "-".to_string()),
    );
    push(
        Page::Overview,
        snapshot.bandwidth.is_some(),
        snapshot
            .bandwidth
            .as_ref()
            .map(|info| {
                info.read_megabytes_per_second
                    .map(|value| format!("{value:.0} MB/s read"))
                    .unwrap_or_else(|| "-".to_string())
            })
            .unwrap_or_else(|| "-".to_string()),
    );

    ui.horizontal(|ui| {
        widgets::badge(
            ui,
            format!(
                "{}: {}",
                i18n.t("checklist.items"),
                rows.iter().filter(|row| row.available).count()
            ),
            widgets::BadgeKind::Ok,
        );
        if state.permission.elevated {
            widgets::badge(ui, i18n.t("permission.elevated"), widgets::BadgeKind::Info);
        } else {
            widgets::badge(
                ui,
                i18n.t("permission.standard"),
                widgets::BadgeKind::Warning,
            );
        }
    });
    ui.add_space(6.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Grid::new("checklist-grid")
                .num_columns(3)
                .striped(true)
                .spacing([16.0, 6.0])
                .show(ui, |ui| {
                    for header in [
                        i18n.t("common.page"),
                        i18n.t("common.status"),
                        i18n.t("common.detail"),
                    ] {
                        ui.label(RichText::new(header).weak());
                    }
                    ui.end_row();
                    for row in &rows {
                        ui.label(i18n.t(row.page.title_key()));
                        if row.available {
                            widgets::badge(ui, i18n.t("app.available"), widgets::BadgeKind::Ok);
                        } else {
                            widgets::badge(
                                ui,
                                i18n.t("app.not_available"),
                                widgets::BadgeKind::Warning,
                            );
                        }
                        ui.label(&row.detail);
                        ui.end_row();
                    }
                });
        });

    if !state.permission.items.is_empty() {
        ui.add_space(8.0);
        widgets::card(ui, |ui| {
            ui.label(RichText::new(i18n.t("permission.items")).strong());
            for item in &state.permission.items {
                ui.label(item);
            }
        });
    }

    if !snapshot.failures.is_empty() {
        ui.add_space(8.0);
        widgets::card(ui, |ui| {
            ui.label(RichText::new(i18n.t("common.warnings")).strong());
            for failure in &snapshot.failures {
                ui.label(RichText::new(format!("{}: {}", failure.module, failure.message)).weak());
            }
        });
    }
}
