//! Design tokens: the single place where colours, radii, spacing and icon
//! geometry live.
//!
//! Every other UI module reads colours from Theme; Color32 literals are
//! deliberately confined to this file so the palette can change in one place.
//! Dark is the primary palette, light is derived from it.

use egui::{Color32, CornerRadius, FontId, Pos2, Rect, Stroke, Vec2};

/// Spacing scale. Only these six steps are used across the UI.
pub const SPACE_1: f32 = 4.0;
pub const SPACE_2: f32 = 8.0;
pub const SPACE_3: f32 = 12.0;
pub const SPACE_4: f32 = 16.0;
pub const SPACE_6: f32 = 24.0;
pub const SPACE_8: f32 = 32.0;

/// Card interior padding and the gap between two sections of a page.
pub const CARD_PADDING: f32 = SPACE_4;
pub const SECTION_GAP: f32 = SPACE_6;

/// Table rows and interactive controls share one height.
pub const ROW_HEIGHT: f32 = 32.0;
pub const CONTROL_HEIGHT: f32 = 32.0;

pub const RADIUS_CONTROL: u8 = 6;
pub const RADIUS_CARD: u8 = 10;
pub const RADIUS_OVERLAY: u8 = 14;

pub const BORDER_WIDTH: f32 = 1.0;
pub const FOCUS_WIDTH: f32 = 2.0;

/// Sidebar geometry: expanded rail and the collapsed icon-only rail.
pub const SIDEBAR_WIDTH: f32 = 240.0;
pub const SIDEBAR_COLLAPSED_WIDTH: f32 = 56.0;
/// Width of the accent bar marking the selected navigation entry.
pub const SELECTION_BAR_WIDTH: f32 = 3.0;

/// Type scale. There are exactly five steps; panels must not invent sizes.
pub const FONT_DISPLAY: f32 = 30.0;
pub const FONT_H1: f32 = 20.0;
pub const FONT_H2: f32 = 15.0;
pub const FONT_BODY: f32 = 13.0;
pub const FONT_CAPTION: f32 = 12.0;
/// Metric numerals: larger than H1 on purpose, monospaced for stable widths.
pub const FONT_NUMERAL: f32 = 28.0;

/// Page width cap for single column pages such as About and Settings.
pub const CONTENT_MAX_WIDTH: f32 = 720.0;

/// Width of the label column in definition lists.
pub const KV_LABEL_WIDTH: f32 = 132.0;

/// The palette for one theme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    pub dark: bool,
    /// Page background.
    pub canvas: Color32,
    /// Card surface.
    pub surface: Color32,
    /// Raised surface: skeleton blocks, inputs.
    pub raised: Color32,
    /// Menus, tooltips, toasts.
    pub overlay: Color32,
    pub border_subtle: Color32,
    pub border_strong: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_tertiary: Color32,
    pub accent: Color32,
    pub accent_pressed: Color32,
    pub accent_subtle: Color32,
    pub ok: Color32,
    pub warn: Color32,
    pub err: Color32,
    pub info: Color32,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            dark: true,
            canvas: Color32::from_rgb(0x10, 0x12, 0x16),
            surface: Color32::from_rgb(0x17, 0x1A, 0x1F),
            raised: Color32::from_rgb(0x1E, 0x23, 0x2A),
            overlay: Color32::from_rgb(0x23, 0x2A, 0x33),
            border_subtle: Color32::from_rgb(0x26, 0x2B, 0x33),
            border_strong: Color32::from_rgb(0x33, 0x3B, 0x45),
            text_primary: Color32::from_rgb(0xE8, 0xEC, 0xF2),
            text_secondary: Color32::from_rgb(0xA7, 0xB1, 0xC0),
            text_tertiary: Color32::from_rgb(0x77, 0x82, 0x8F),
            accent: Color32::from_rgb(0x4C, 0x8D, 0xFF),
            accent_pressed: Color32::from_rgb(0x2F, 0x6B, 0xD8),
            accent_subtle: Color32::from_rgba_unmultiplied(0x4C, 0x8D, 0xFF, 36),
            ok: Color32::from_rgb(0x4C, 0xC3, 0x8A),
            warn: Color32::from_rgb(0xE5, 0xB4, 0x54),
            err: Color32::from_rgb(0xE5, 0x64, 0x5E),
            info: Color32::from_rgb(0x6A, 0xA9, 0xFF),
        }
    }

    pub fn light() -> Self {
        Self {
            dark: false,
            canvas: Color32::from_rgb(0xF7, 0xF8, 0xFA),
            surface: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            raised: Color32::from_rgb(0xF1, 0xF3, 0xF7),
            overlay: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            border_subtle: Color32::from_rgb(0xE6, 0xE9, 0xEF),
            border_strong: Color32::from_rgb(0xD3, 0xD9, 0xE2),
            text_primary: Color32::from_rgb(0x16, 0x19, 0x1F),
            text_secondary: Color32::from_rgb(0x5A, 0x64, 0x72),
            text_tertiary: Color32::from_rgb(0x8A, 0x93, 0xA0),
            accent: Color32::from_rgb(0x25, 0x63, 0xEB),
            accent_pressed: Color32::from_rgb(0x1D, 0x4E, 0xD8),
            accent_subtle: Color32::from_rgba_unmultiplied(0x25, 0x63, 0xEB, 26),
            ok: Color32::from_rgb(0x1F, 0x9A, 0x63),
            warn: Color32::from_rgb(0xB4, 0x7A, 0x0E),
            err: Color32::from_rgb(0xCF, 0x3B, 0x36),
            info: Color32::from_rgb(0x25, 0x63, 0xEB),
        }
    }

    pub fn of(dark: bool) -> Self {
        if dark { Self::dark() } else { Self::light() }
    }

    /// A card: surface fill, subtle border, no shadow.
    pub fn card_frame(self) -> egui::Frame {
        egui::Frame::new()
            .fill(self.surface)
            .stroke(Stroke::new(BORDER_WIDTH, self.border_subtle))
            .corner_radius(CornerRadius::same(RADIUS_CARD))
            .inner_margin(egui::Margin::same(CARD_PADDING as i8))
    }

    /// A floating surface (menu, popup, toast): the only place shadows appear.
    pub fn overlay_frame(self) -> egui::Frame {
        egui::Frame::new()
            .fill(self.overlay)
            .stroke(Stroke::new(BORDER_WIDTH, self.border_strong))
            .corner_radius(CornerRadius::same(RADIUS_OVERLAY))
            .inner_margin(egui::Margin::same(SPACE_3 as i8))
            .shadow(egui::epaint::Shadow {
                offset: [0, 8],
                blur: 24,
                spread: 0,
                color: Color32::from_black_alpha(70),
            })
    }

    pub fn body_font(self) -> FontId {
        FontId::proportional(FONT_BODY)
    }

    pub fn caption_font(self) -> FontId {
        FontId::proportional(FONT_CAPTION)
    }

    pub fn numeral_font(self) -> FontId {
        FontId::monospace(FONT_NUMERAL)
    }
}

/// The theme of the current Ui, derived from the active egui theme.
pub fn theme(ui: &egui::Ui) -> Theme {
    Theme::of(ui.visuals().dark_mode)
}

/// Build egui visuals from the tokens.
pub fn visuals(t: Theme) -> egui::Visuals {
    let mut visuals = if t.dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    visuals.panel_fill = t.canvas;
    visuals.window_fill = t.overlay;
    visuals.extreme_bg_color = if t.dark {
        Color32::from_rgb(0x0B, 0x0D, 0x10)
    } else {
        t.raised
    };
    visuals.text_edit_bg_color = Some(visuals.extreme_bg_color);
    visuals.faint_bg_color = t.raised;
    visuals.code_bg_color = t.raised;
    visuals.window_stroke = Stroke::new(BORDER_WIDTH, t.border_strong);
    visuals.window_corner_radius = CornerRadius::same(RADIUS_OVERLAY);
    visuals.menu_corner_radius = CornerRadius::same(RADIUS_OVERLAY);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 8],
        blur: 24,
        spread: 0,
        color: Color32::from_black_alpha(70),
    };
    visuals.popup_shadow = visuals.window_shadow;
    visuals.hyperlink_color = t.accent;
    visuals.warn_fg_color = t.warn;
    visuals.error_fg_color = t.err;
    visuals.weak_text_color = Some(t.text_secondary);
    visuals.weak_text_alpha = 1.0;
    visuals.striped = false;
    visuals.button_frame = true;
    visuals.selection.bg_fill = t.accent_subtle;
    // 焦点环固定 2px，键盘导航时能看清当前控件。
    visuals.selection.stroke = Stroke::new(FOCUS_WIDTH, t.accent);
    visuals.widgets.noninteractive = widget_visuals(
        t.surface,
        t.surface,
        t.border_subtle,
        t.text_primary,
        RADIUS_CARD,
    );
    visuals.widgets.inactive = widget_visuals(
        t.raised,
        t.raised,
        t.border_subtle,
        t.text_secondary,
        RADIUS_CONTROL,
    );
    visuals.widgets.hovered = widget_visuals(
        t.raised,
        t.raised,
        t.border_strong,
        t.text_primary,
        RADIUS_CONTROL,
    );
    visuals.widgets.active = widget_visuals(
        t.accent_pressed,
        t.accent_pressed,
        t.accent_pressed,
        Color32::WHITE,
        RADIUS_CONTROL,
    );
    visuals.widgets.open = visuals.widgets.hovered;
    visuals
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
        bg_stroke: Stroke::new(BORDER_WIDTH, stroke),
        corner_radius: CornerRadius::same(radius),
        fg_stroke: Stroke::new(1.0, fg),
        expansion: 0.0,
    }
}

/// Vector icons. Glyph characters are never used as icons.
///
/// The set is a small design system; a few glyphs (trash, save, update and the
/// chevrons) are drawn for screens that land in 1.4, hence the allowance.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Overview,
    Performance,
    Graphics,
    Hardware,
    Snapshot,
    Settings,
    Refresh,
    Export,
    More,
    Copy,
    Close,
    Check,
    Warning,
    Info,
    Search,
    ChevronLeft,
    ChevronRight,
    Plus,
    Trash,
    Save,
    Update,
    Collapse,
    Expand,
}

/// Draw an icon centred in rect with a 1.6 px stroke.
pub fn paint_icon(painter: &egui::Painter, rect: Rect, icon: Icon, color: Color32) {
    let stroke = Stroke::new(1.6, color);
    let side = rect.width().min(rect.height());
    let rect = Rect::from_center_size(rect.center(), Vec2::splat(side));
    let p = |x: f32, y: f32| Pos2::new(rect.left() + x * side, rect.top() + y * side);
    let line = |a: (f32, f32), b: (f32, f32)| {
        painter.line_segment([p(a.0, a.1), p(b.0, b.1)], stroke);
    };
    let round = CornerRadius::same(1);
    match icon {
        Icon::Overview => {
            for (x, y) in [(0.12, 0.12), (0.56, 0.12), (0.12, 0.56), (0.56, 0.56)] {
                painter.rect_stroke(
                    Rect::from_min_max(p(x, y), p(x + 0.32, y + 0.32)),
                    round,
                    stroke,
                    egui::StrokeKind::Inside,
                );
            }
        }
        Icon::Performance => {
            let centre = p(0.5, 0.6);
            let radius = side * 0.36;
            let points: Vec<Pos2> = (0..=24)
                .map(|i| {
                    let angle = std::f32::consts::PI + i as f32 / 24.0 * std::f32::consts::PI;
                    Pos2::new(
                        centre.x + radius * angle.cos(),
                        centre.y + radius * angle.sin(),
                    )
                })
                .collect();
            painter.add(egui::Shape::line(points, stroke));
            line((0.5, 0.6), (0.74, 0.34));
            painter.circle_filled(centre, side * 0.045, color);
        }
        Icon::Graphics => {
            painter.rect_stroke(
                Rect::from_min_max(p(0.1, 0.16), p(0.9, 0.66)),
                round,
                stroke,
                egui::StrokeKind::Inside,
            );
            line((0.5, 0.66), (0.5, 0.8));
            line((0.3, 0.86), (0.7, 0.86));
        }
        Icon::Hardware => {
            painter.rect_stroke(
                Rect::from_min_max(p(0.24, 0.24), p(0.76, 0.76)),
                round,
                stroke,
                egui::StrokeKind::Inside,
            );
            painter.rect_stroke(
                Rect::from_min_max(p(0.38, 0.38), p(0.62, 0.62)),
                round,
                stroke,
                egui::StrokeKind::Inside,
            );
            for offset in [0.34, 0.5, 0.66] {
                line((offset, 0.1), (offset, 0.24));
                line((offset, 0.76), (offset, 0.9));
                line((0.1, offset), (0.24, offset));
                line((0.76, offset), (0.9, offset));
            }
        }
        Icon::Snapshot => {
            painter.rect_stroke(
                Rect::from_min_max(p(0.1, 0.26), p(0.9, 0.82)),
                round,
                stroke,
                egui::StrokeKind::Inside,
            );
            painter.rect_stroke(
                Rect::from_min_max(p(0.36, 0.14), p(0.6, 0.26)),
                round,
                stroke,
                egui::StrokeKind::Inside,
            );
            painter.circle_stroke(p(0.5, 0.54), side * 0.15, stroke);
        }
        Icon::Settings => {
            painter.circle_stroke(p(0.5, 0.5), side * 0.18, stroke);
            for i in 0..8 {
                let angle = i as f32 * std::f32::consts::FRAC_PI_4;
                let (sin, cos) = angle.sin_cos();
                line(
                    (0.5 + cos * 0.28, 0.5 + sin * 0.28),
                    (0.5 + cos * 0.42, 0.5 + sin * 0.42),
                );
            }
        }
        Icon::Refresh => {
            let centre = p(0.5, 0.5);
            let radius = side * 0.32;
            let points: Vec<Pos2> = (0..=20)
                .map(|i| {
                    let angle =
                        -std::f32::consts::FRAC_PI_2 + i as f32 / 20.0 * 1.6 * std::f32::consts::PI;
                    Pos2::new(
                        centre.x + radius * angle.cos(),
                        centre.y + radius * angle.sin(),
                    )
                })
                .collect();
            painter.add(egui::Shape::line(points, stroke));
            line((0.72, 0.2), (0.86, 0.32));
            line((0.86, 0.32), (0.7, 0.4));
        }
        Icon::Export => {
            line((0.5, 0.12), (0.5, 0.56));
            line((0.32, 0.4), (0.5, 0.58));
            line((0.68, 0.4), (0.5, 0.58));
            line((0.16, 0.62), (0.16, 0.86));
            line((0.16, 0.86), (0.84, 0.86));
            line((0.84, 0.86), (0.84, 0.62));
        }
        Icon::More => {
            for x in [0.28, 0.5, 0.72] {
                painter.circle_filled(p(x, 0.5), side * 0.06, color);
            }
        }
        Icon::Copy => {
            painter.rect_stroke(
                Rect::from_min_max(p(0.34, 0.12), p(0.86, 0.64)),
                round,
                stroke,
                egui::StrokeKind::Inside,
            );
            painter.rect_stroke(
                Rect::from_min_max(p(0.14, 0.34), p(0.66, 0.86)),
                round,
                stroke,
                egui::StrokeKind::Inside,
            );
        }
        Icon::Close => {
            line((0.24, 0.24), (0.76, 0.76));
            line((0.76, 0.24), (0.24, 0.76));
        }
        Icon::Check => {
            line((0.2, 0.52), (0.42, 0.74));
            line((0.42, 0.74), (0.8, 0.28));
        }
        Icon::Warning => {
            line((0.5, 0.12), (0.92, 0.86));
            line((0.92, 0.86), (0.08, 0.86));
            line((0.08, 0.86), (0.5, 0.12));
            line((0.5, 0.38), (0.5, 0.62));
            painter.circle_filled(p(0.5, 0.73), side * 0.04, color);
        }
        Icon::Info => {
            painter.circle_stroke(p(0.5, 0.5), side * 0.36, stroke);
            line((0.5, 0.44), (0.5, 0.68));
            painter.circle_filled(p(0.5, 0.33), side * 0.04, color);
        }
        Icon::Search => {
            painter.circle_stroke(p(0.44, 0.44), side * 0.26, stroke);
            line((0.64, 0.64), (0.84, 0.84));
        }
        Icon::ChevronLeft => {
            line((0.62, 0.2), (0.36, 0.5));
            line((0.36, 0.5), (0.62, 0.8));
        }
        Icon::ChevronRight => {
            line((0.38, 0.2), (0.64, 0.5));
            line((0.64, 0.5), (0.38, 0.8));
        }
        Icon::Plus => {
            line((0.5, 0.16), (0.5, 0.84));
            line((0.16, 0.5), (0.84, 0.5));
        }
        Icon::Trash => {
            line((0.14, 0.26), (0.86, 0.26));
            line((0.38, 0.26), (0.38, 0.14));
            line((0.38, 0.14), (0.62, 0.14));
            line((0.62, 0.14), (0.62, 0.26));
            painter.rect_stroke(
                Rect::from_min_max(p(0.22, 0.26), p(0.78, 0.86)),
                round,
                stroke,
                egui::StrokeKind::Inside,
            );
        }
        Icon::Save => {
            painter.rect_stroke(
                Rect::from_min_max(p(0.14, 0.14), p(0.86, 0.86)),
                round,
                stroke,
                egui::StrokeKind::Inside,
            );
            painter.rect_stroke(
                Rect::from_min_max(p(0.32, 0.14), p(0.68, 0.4)),
                round,
                stroke,
                egui::StrokeKind::Inside,
            );
            line((0.3, 0.86), (0.3, 0.58));
            line((0.3, 0.58), (0.7, 0.58));
            line((0.7, 0.58), (0.7, 0.86));
        }
        Icon::Update => {
            painter.circle_stroke(p(0.5, 0.5), side * 0.34, stroke);
            line((0.5, 0.26), (0.5, 0.52));
            line((0.5, 0.52), (0.68, 0.62));
        }
        Icon::Collapse => {
            line((0.62, 0.2), (0.36, 0.5));
            line((0.36, 0.5), (0.62, 0.8));
        }
        Icon::Expand => {
            line((0.38, 0.2), (0.64, 0.5));
            line((0.64, 0.5), (0.38, 0.8));
        }
    }
}

/// A single icon rendered as a widget.
pub fn icon_widget(ui: &mut egui::Ui, icon: Icon, size: f32) -> egui::Response {
    let (response, painter) = ui.allocate_painter(Vec2::splat(size), egui::Sense::hover());
    let color = ui.visuals().weak_text_color();
    paint_icon(&painter, response.rect, icon, color);
    response
}

/// RichText in the numeral style: monospaced digits, private-use content.
pub fn numeral(text: impl Into<String>, t: Theme) -> egui::RichText {
    egui::RichText::new(text.into())
        .font(t.numeral_font())
        .color(t.text_primary)
        .strong()
}
