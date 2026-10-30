//! Left navigation rail.

use egui::{Align, Layout, RichText};

use crate::i18n::I18n;
use crate::state::{AppState, Page};
use crate::ui::widgets::{BadgeKind, badge};

const GROUPS: [(&str, &[Page]); 4] = [
    (
        "group.monitoring",
        &[
            Page::Overview,
            Page::Cpu,
            Page::Vulkan,
            Page::OpenGl,
            Page::OpenCl,
            Page::Sensors,
        ],
    ),
    (
        "app.hardware",
        &[
            Page::Pcie,
            Page::Npu,
            Page::Usb,
            Page::Storage,
            Page::Display,
            Page::Network,
            Page::Audio,
        ],
    ),
    (
        "app.system",
        &[Page::DirectX, Page::Metal, Page::OpenVx, Page::Checklist],
    ),
    (
        "app.tools",
        &[Page::Dev, Page::Snapshots, Page::Compare, Page::Settings],
    ),
];

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new("◈").size(22.0).strong());
        ui.vertical(|ui| {
            ui.label(RichText::new(i18n.t("app.title")).size(16.0).strong());
            ui.label(RichText::new(i18n.t("app.subtitle")).weak().size(10.0));
        });
    });
    ui.add_space(8.0);
    ui.separator();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (group, pages) in GROUPS {
                ui.add_space(8.0);
                ui.label(RichText::new(i18n.t(group)).size(10.0).weak().strong());
                for page in pages {
                    let selected = state.settings.page == *page;
                    let response = ui.add_sized(
                        [ui.available_width(), 28.0],
                        egui::Button::selectable(
                            selected,
                            format!("{}  {}", page.icon(), i18n.t(page.title_key())),
                        ),
                    );
                    if response.clicked() {
                        state.settings.page = *page;
                        state.mark_dirty();
                    }
                }
            }
        });

    ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
        ui.add_space(6.0);
        ui.label(
            RichText::new(format!(
                "{} {}",
                i18n.t("app.version"),
                env!("CARGO_PKG_VERSION")
            ))
            .weak()
            .size(10.0),
        );
        ui.separator();
        let warnings = state.total_warnings();
        ui.horizontal(|ui| {
            badge(
                ui,
                format!("{}: {}", i18n.t("app.warnings"), warnings),
                if warnings == 0 {
                    BadgeKind::Ok
                } else {
                    BadgeKind::Warning
                },
            );
            if state.permission.elevated {
                badge(ui, i18n.t("app.elevated"), BadgeKind::Info);
            }
        });
    });
}
