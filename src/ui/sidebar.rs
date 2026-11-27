//! Left navigation rail: six sections, collapsible to an icon-only strip.

use egui::{Align, Layout, RichText, Sense, Vec2};

use crate::i18n::I18n;
use crate::state::{AppState, Page};

use super::theme::{self, Icon, Theme};

/// One top level navigation entry. Its pages are shown as a tab strip in the
/// content area, not in the rail.
pub struct Section {
    pub title_key: &'static str,
    pub icon: Icon,
    pub pages: &'static [Page],
}

pub const SECTIONS: [Section; 6] = [
    Section {
        title_key: "section.overview",
        icon: Icon::Overview,
        pages: &[Page::Overview, Page::Checklist],
    },
    Section {
        title_key: "section.performance",
        icon: Icon::Performance,
        pages: &[Page::Cpu, Page::Sensors],
    },
    Section {
        title_key: "section.graphics",
        icon: Icon::Graphics,
        pages: &[
            Page::Vulkan,
            Page::OpenGl,
            Page::Metal,
            Page::DirectX,
            Page::OpenCl,
            Page::OpenVx,
        ],
    },
    Section {
        title_key: "section.hardware",
        icon: Icon::Hardware,
        pages: &[
            Page::Pcie,
            Page::Usb,
            Page::Storage,
            Page::Display,
            Page::Network,
            Page::Audio,
            Page::Npu,
        ],
    },
    Section {
        title_key: "section.snapshots",
        icon: Icon::Snapshot,
        pages: &[Page::Snapshots, Page::Compare],
    },
    Section {
        title_key: "section.settings",
        icon: Icon::Settings,
        pages: &[Page::Settings, Page::About, Page::Updates, Page::Dev],
    },
];

/// The section containing the given page.
pub fn section_of(page: Page) -> &'static Section {
    SECTIONS
        .iter()
        .find(|section| section.pages.contains(&page))
        .unwrap_or(&SECTIONS[0])
}

fn collapsed_id() -> egui::Id {
    egui::Id::unique("device-viewer-sidebar-collapsed")
}

pub fn is_collapsed(ctx: &egui::Context) -> bool {
    ctx.data(|data| data.get_temp::<bool>(collapsed_id()).unwrap_or(false))
}

pub fn width(ctx: &egui::Context) -> f32 {
    if is_collapsed(ctx) {
        theme::SIDEBAR_COLLAPSED_WIDTH
    } else {
        theme::SIDEBAR_WIDTH
    }
}

fn set_collapsed(ctx: &egui::Context, value: bool) {
    ctx.data_mut(|data| data.insert_temp(collapsed_id(), value));
}

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    let t = theme::theme(ui);
    let collapsed = is_collapsed(ui.ctx());
    let current = section_of(state.settings.page).title_key;

    ui.add_space(theme::SPACE_3);
    if collapsed {
        ui.vertical_centered(|ui| {
            theme::icon_widget(ui, Icon::Hardware, 22.0);
        });
    } else {
        ui.horizontal(|ui| {
            theme::icon_widget(ui, Icon::Hardware, 22.0);
            ui.add_space(theme::SPACE_2);
            ui.label(
                RichText::new(i18n.t("app.title"))
                    .size(theme::FONT_H2)
                    .strong()
                    .color(t.text_primary),
            );
        });
    }
    ui.add_space(theme::SPACE_3);
    ui.horizontal(|ui| {
        if !collapsed {
            ui.label(
                RichText::new(i18n.t("app.subtitle"))
                    .size(theme::FONT_CAPTION)
                    .color(t.text_tertiary),
            );
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let (icon, hint) = if collapsed {
                (Icon::Expand, "nav.expand")
            } else {
                (Icon::Collapse, "nav.collapse")
            };
            if super::widgets::icon_button(ui, icon, i18n.t(hint)).clicked() {
                set_collapsed(ui.ctx(), !collapsed);
            }
        });
    });
    ui.add_space(theme::SPACE_3);

    let warnings = state.total_warnings();

    // 页脚占固定高度，导航列表只吃剩下的空间。之前用 bottom_up 布局把
    // 剩余高度整个吃掉，滚动区拿到 0 高度，六个分区一个都没画出来。
    let footer_height = 56.0;
    let nav_height = (ui.available_height() - footer_height).max(0.0);
    egui::ScrollArea::vertical()
        .id_salt("sidebar-scroll")
        .max_height(nav_height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for section in &SECTIONS {
                let selected = section.title_key == current;
                if nav_row(
                    ui,
                    &t,
                    section.icon,
                    i18n.t(section.title_key),
                    selected,
                    collapsed,
                )
                .clicked()
                {
                    state.settings.page = section.pages[0];
                    state.mark_dirty();
                }
                ui.add_space(theme::SPACE_1);
            }
        });

    ui.add_space(theme::SPACE_2);
    ui.separator();
    ui.add_space(theme::SPACE_2);
    if collapsed {
        ui.vertical_centered(|ui| {
            status_dot(ui, &t, warnings, i18n);
        });
    } else {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "{} {}",
                    i18n.t("app.version"),
                    env!("CARGO_PKG_VERSION")
                ))
                .size(theme::FONT_CAPTION)
                .color(t.text_tertiary),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                status_dot(ui, &t, warnings, i18n);
            });
        });
    }
    ui.add_space(theme::SPACE_2);
}

/// The single place that reports the number of warnings: a coloured dot plus
/// the count. Detail lives in the page level warning summary.
fn status_dot(ui: &mut egui::Ui, t: &Theme, warnings: usize, i18n: I18n) {
    let color = if warnings == 0 { t.ok } else { t.warn };
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(20.0), Sense::hover());
    ui.painter().circle_filled(rect.center(), 4.0, color);
    let label = format!("{}: {warnings}", i18n.t("app.warnings"));
    response.on_hover_text(label.clone());
    if !is_collapsed(ui.ctx()) {
        ui.label(
            RichText::new(warnings.to_string())
                .size(theme::FONT_CAPTION)
                .color(t.text_tertiary),
        );
    }
}

fn nav_row(
    ui: &mut egui::Ui,
    t: &Theme,
    icon: Icon,
    label: &str,
    selected: bool,
    collapsed: bool,
) -> egui::Response {
    let height = 36.0;
    let width = ui
        .available_width()
        .max(theme::SIDEBAR_COLLAPSED_WIDTH - 16.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
    if selected || response.hovered() {
        let fill = if selected { t.accent_subtle } else { t.raised };
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(theme::RADIUS_CONTROL), fill);
    }
    if selected {
        let bar = egui::Rect::from_min_size(
            rect.left_top(),
            Vec2::new(theme::SELECTION_BAR_WIDTH, rect.height()),
        );
        ui.painter()
            .rect_filled(bar, egui::CornerRadius::same(2), t.accent);
    }
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 20.0, rect.center().y),
        Vec2::splat(18.0),
    );
    let icon_color = if selected { t.accent } else { t.text_secondary };
    theme::paint_icon(ui.painter(), icon_rect, icon, icon_color);
    if !collapsed {
        ui.painter().text(
            egui::pos2(rect.left() + 38.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(theme::FONT_BODY),
            if selected {
                t.text_primary
            } else {
                t.text_secondary
            },
        );
    } else {
        response.clone().on_hover_text(label);
    }
    response
}
