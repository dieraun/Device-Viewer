//! Small, reusable presentation helpers shared by every panel.

use egui::{Align, Color32, CornerRadius, FontId, Layout, RichText, Stroke, Vec2};

use crate::i18n::I18n;

const PAGE_BG: Color32 = Color32::from_rgb(11, 15, 20);
const CARD_BG: Color32 = Color32::from_rgb(21, 28, 37);
const CARD_BG_ALT: Color32 = Color32::from_rgb(18, 25, 34);
const OVERLAY_BG: Color32 = Color32::from_rgb(27, 36, 48);
const BORDER: Color32 = Color32::from_rgb(38, 50, 65);
const BORDER_SOFT: Color32 = Color32::from_rgb(31, 42, 55);
const TEXT: Color32 = Color32::from_rgb(220, 229, 239);
const TEXT_MUTED: Color32 = Color32::from_rgb(143, 160, 181);
const ACCENT: Color32 = Color32::from_rgb(76, 154, 255);

/// Install the application-wide spacing, typography and dark/light visuals.
///
/// Both themes are configured because `ThemeMode::System` can switch between
/// them at runtime. The call is intentionally independent from
/// `Context::set_theme`, which only selects the active theme.
pub fn install_style(ctx: &egui::Context) {
    ctx.style_mut_of(egui::Theme::Dark, |style| {
        style.visuals = dark_visuals();
        style_spacing(style);
    });
    ctx.style_mut_of(egui::Theme::Light, |style| {
        style.visuals = light_visuals();
        style_spacing(style);
    });
}

fn style_spacing(style: &mut egui::Style) {
    style.spacing.item_spacing = Vec2::new(8.0, 7.0);
    style.spacing.button_padding = Vec2::new(10.0, 5.0);
    style.spacing.interact_size.y = 28.0;
    style.spacing.window_margin = egui::Margin::same(10);
    style.spacing.menu_margin = egui::Margin::same(8);
    style.spacing.scroll.bar_width = 8.0;
    style.spacing.scroll.floating = false;
    style
        .text_styles
        .insert(egui::TextStyle::Body, FontId::proportional(13.5));
    style
        .text_styles
        .insert(egui::TextStyle::Button, FontId::proportional(13.0));
    style
        .text_styles
        .insert(egui::TextStyle::Small, FontId::proportional(11.0));
    style
        .text_styles
        .insert(egui::TextStyle::Heading, FontId::proportional(23.0));
    style
        .text_styles
        .insert(egui::TextStyle::Monospace, FontId::monospace(12.0));
}

fn widget_visuals(
    bg: Color32,
    weak_bg: Color32,
    stroke: Color32,
    fg: Color32,
    radius: u8,
) -> egui::style::WidgetVisuals {
    egui::style::WidgetVisuals {
        bg_fill: bg,
        weak_bg_fill: weak_bg,
        bg_stroke: Stroke::new(1.0, stroke),
        corner_radius: CornerRadius::same(radius),
        fg_stroke: Stroke::new(1.0, fg),
        expansion: 0.0,
    }
}

fn dark_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = PAGE_BG;
    visuals.window_fill = OVERLAY_BG;
    visuals.extreme_bg_color = Color32::from_rgb(8, 12, 17);
    visuals.text_edit_bg_color = Some(Color32::from_rgb(10, 16, 23));
    visuals.faint_bg_color = CARD_BG_ALT;
    visuals.code_bg_color = Color32::from_rgb(15, 22, 32);
    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.window_corner_radius = CornerRadius::same(12);
    visuals.menu_corner_radius = CornerRadius::same(10);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 10],
        blur: 28,
        spread: 0,
        color: Color32::from_black_alpha(110),
    };
    visuals.popup_shadow = egui::epaint::Shadow {
        offset: [0, 6],
        blur: 18,
        spread: 0,
        color: Color32::from_black_alpha(95),
    };
    visuals.hyperlink_color = Color32::from_rgb(110, 168, 254);
    visuals.warn_fg_color = Color32::from_rgb(242, 184, 75);
    visuals.error_fg_color = Color32::from_rgb(248, 113, 113);
    visuals.weak_text_color = Some(TEXT_MUTED);
    visuals.weak_text_alpha = 0.72;
    visuals.striped = true;
    visuals.button_frame = true;
    visuals.selection.bg_fill = Color32::from_rgb(30, 58, 95);
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(207, 228, 255));
    visuals.widgets.noninteractive = widget_visuals(CARD_BG, CARD_BG, BORDER, TEXT, 10);
    visuals.widgets.inactive = widget_visuals(
        CARD_BG_ALT,
        CARD_BG_ALT,
        BORDER_SOFT,
        Color32::from_rgb(201, 213, 227),
        6,
    );
    visuals.widgets.hovered = widget_visuals(
        Color32::from_rgb(32, 45, 60),
        Color32::from_rgb(32, 45, 60),
        ACCENT,
        Color32::WHITE,
        6,
    );
    visuals.widgets.active = widget_visuals(
        Color32::from_rgb(29, 78, 216),
        Color32::from_rgb(29, 78, 216),
        Color32::from_rgb(96, 165, 250),
        Color32::WHITE,
        6,
    );
    visuals.widgets.open = visuals.widgets.hovered;
    visuals
}

fn light_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::light();
    visuals.panel_fill = Color32::from_rgb(244, 247, 251);
    visuals.window_fill = Color32::WHITE;
    visuals.faint_bg_color = Color32::from_rgb(248, 250, 252);
    visuals.extreme_bg_color = Color32::WHITE;
    visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(214, 222, 232));
    visuals.window_corner_radius = CornerRadius::same(12);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 8],
        blur: 24,
        spread: 0,
        color: Color32::from_black_alpha(36),
    };
    visuals.weak_text_color = Some(Color32::from_rgb(91, 102, 117));
    visuals.weak_text_alpha = 0.72;
    visuals.widgets.noninteractive = widget_visuals(
        Color32::WHITE,
        Color32::WHITE,
        Color32::from_rgb(219, 226, 235),
        Color32::from_rgb(35, 45, 58),
        10,
    );
    visuals.widgets.inactive = widget_visuals(
        Color32::from_rgb(248, 250, 252),
        Color32::from_rgb(248, 250, 252),
        Color32::from_rgb(219, 226, 235),
        Color32::from_rgb(55, 68, 84),
        6,
    );
    visuals.widgets.hovered = widget_visuals(
        Color32::from_rgb(238, 244, 252),
        Color32::from_rgb(238, 244, 252),
        Color32::from_rgb(59, 130, 246),
        Color32::from_rgb(28, 43, 62),
        6,
    );
    visuals.widgets.active = widget_visuals(
        Color32::from_rgb(37, 99, 235),
        Color32::from_rgb(37, 99, 235),
        Color32::from_rgb(37, 99, 235),
        Color32::WHITE,
        6,
    );
    visuals.widgets.open = visuals.widgets.hovered;
    visuals
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeKind {
    Ok,
    Warning,
    Error,
    Info,
}

impl BadgeKind {
    fn colors(self) -> (Color32, Color32) {
        match self {
            Self::Ok => (
                Color32::from_rgb(70, 180, 110),
                Color32::from_rgb(24, 58, 38),
            ),
            Self::Warning => (
                Color32::from_rgb(230, 180, 70),
                Color32::from_rgb(64, 50, 22),
            ),
            Self::Error => (
                Color32::from_rgb(230, 90, 90),
                Color32::from_rgb(70, 28, 28),
            ),
            Self::Info => (
                Color32::from_rgb(100, 160, 230),
                Color32::from_rgb(26, 46, 70),
            ),
        }
    }
}

pub fn page_header(
    ui: &mut egui::Ui,
    i18n: I18n,
    title_key: &'static str,
    subtitle_key: &'static str,
) {
    ui.add_space(4.0);
    ui.label(RichText::new(i18n.t(title_key)).size(24.0).strong());
    ui.label(RichText::new(i18n.t(subtitle_key)).weak());
    ui.add_space(10.0);
}

pub fn card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    card_response(ui, add).inner
}

pub fn card_response<R>(
    ui: &mut egui::Ui,
    add: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let frame = if ui.visuals().dark_mode {
        egui::Frame::new()
            .fill(CARD_BG)
            .stroke(Stroke::new(1.0, BORDER))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(egui::Margin::same(14))
            .shadow(egui::epaint::Shadow {
                offset: [0, 4],
                blur: 14,
                spread: 0,
                color: Color32::from_black_alpha(80),
            })
    } else {
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(14))
            .corner_radius(CornerRadius::same(10))
    };
    frame.show(ui, add)
}

pub fn collapsible(
    ui: &mut egui::Ui,
    id: &str,
    title: impl Into<egui::WidgetText>,
    default_open: bool,
    add: impl FnOnce(&mut egui::Ui),
) {
    let _ = egui::CollapsingHeader::new(title)
        .id_salt(id)
        .default_open(default_open)
        .show(ui, add);
}

pub fn badge_response(
    ui: &mut egui::Ui,
    text: impl Into<String>,
    kind: BadgeKind,
) -> egui::Response {
    let (foreground, background) = kind.colors();
    egui::Frame::new()
        .fill(background)
        .corner_radius(CornerRadius::same(4))
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(text).color(foreground).size(12.0).strong());
        })
        .response
}

pub fn badge(ui: &mut egui::Ui, text: impl Into<String>, kind: BadgeKind) {
    let _ = badge_response(ui, text, kind);
}

pub fn copy_button(ui: &mut egui::Ui, i18n: I18n, text: impl Into<String>) {
    let text = text.into();
    if ui
        .button(format!("⧉ {}", i18n.t("app.copy")))
        .on_hover_text(i18n.t("app.copy"))
        .clicked()
    {
        ui.ctx().copy_text(text);
    }
}

pub fn copy_table_button(ui: &mut egui::Ui, i18n: I18n, markdown: String) {
    if ui
        .button(format!("⧉ {}", i18n.t("export.markdown_table")))
        .clicked()
    {
        ui.ctx().copy_text(markdown);
    }
}

pub fn kv_grid(ui: &mut egui::Ui, id: &str, rows: &[(&str, String)]) {
    egui::Grid::new(id)
        .num_columns(2)
        .spacing([18.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            for (key, value) in rows {
                ui.label(RichText::new(*key).weak());
                ui.label(if value.is_empty() { "—" } else { value });
                ui.end_row();
            }
        });
}

pub fn empty(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.add_space(16.0);
    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        ui.label(RichText::new("◌").size(28.0).weak());
        ui.label(RichText::new(text.into()).weak());
    });
    ui.add_space(16.0);
}

pub fn warnings(ui: &mut egui::Ui, i18n: I18n, id: &str, warnings: &[String]) {
    if warnings.is_empty() {
        return;
    }
    collapsible(
        ui,
        id,
        format!("{} ({})", i18n.t("app.warnings"), warnings.len()),
        true,
        |ui| {
            for warning in warnings {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new("!")
                            .color(ui.visuals().warn_fg_color)
                            .strong(),
                    );
                    ui.label(warning);
                });
            }
        },
    );
}

pub fn metric(
    ui: &mut egui::Ui,
    label: &str,
    value: impl Into<String>,
    detail: Option<String>,
) -> egui::Response {
    card_response(ui, |ui| {
        ui.set_min_width(160.0);
        ui.label(RichText::new(label).weak().size(11.5));
        ui.add_space(3.0);
        ui.label(RichText::new(value.into()).size(24.0).strong());
        if let Some(detail) = detail {
            ui.add_space(2.0);
            ui.label(RichText::new(detail).weak().size(10.5));
        }
    })
    .response
}

pub fn filter_input(ui: &mut egui::Ui, i18n: I18n, value: &mut String) -> egui::Response {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", i18n.t("app.filter")));
        let response = ui.add(
            egui::TextEdit::singleline(value)
                .hint_text(i18n.t("app.search"))
                .desired_width(240.0),
        );
        if !value.is_empty() && ui.button("✕").clicked() {
            value.clear();
        }
        response
    })
    .inner
}

pub fn sparkline(
    ui: &mut egui::Ui,
    id: &str,
    values: &[f32],
    maximum: f32,
    color: Color32,
    height: f32,
) {
    let width = ui.available_width().max(180.0);
    let (response, painter) = ui.allocate_painter(Vec2::new(width, height), egui::Sense::hover());
    let rect = response.rect;
    painter.rect_filled(rect, CornerRadius::same(6), ui.visuals().extreme_bg_color);
    if values.len() < 2 {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "—",
            FontId::proportional(13.0),
            ui.visuals().weak_text_color(),
        );
        return;
    }
    let maximum = maximum.max(1.0);
    let step = if values.len() > 1 {
        rect.width() / (values.len() - 1) as f32
    } else {
        rect.width()
    };
    let points: Vec<egui::Pos2> = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let x = rect.left() + index as f32 * step;
            let normalized = (value / maximum).clamp(0.0, 1.0);
            let y = rect.bottom() - 2.0 - normalized * (rect.height() - 4.0);
            egui::pos2(x, y)
        })
        .collect();
    painter.line(points, Stroke::new(1.5, color));
    painter.text(
        rect.left_top() + egui::vec2(6.0, 4.0),
        egui::Align2::LEFT_TOP,
        id,
        FontId::proportional(11.0),
        ui.visuals().weak_text_color(),
    );
}

pub fn percent_bar(ui: &mut egui::Ui, label: &str, percent: f32) {
    ui.label(RichText::new(format!("{label}: {percent:.1}%")).size(12.0));
    let fill = if ui.visuals().dark_mode {
        ACCENT
    } else {
        Color32::from_rgb(37, 99, 235)
    };
    ui.add(
        egui::ProgressBar::new((percent / 100.0).clamp(0.0, 1.0))
            .desired_height(8.0)
            .corner_radius(CornerRadius::same(4))
            .fill(fill),
    );
}
