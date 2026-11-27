//! Small, reusable presentation helpers shared by every panel.
//!
//! All colours, radii and font sizes come from crate::ui::theme; this module
//! contains no colour literals.

use egui::{Align, Layout, RichText, Sense, Stroke, Vec2};

use crate::i18n::I18n;
use crate::state::Page;

use super::theme::{self, Icon, Theme};

/// Install the application-wide spacing, typography and themed visuals.
///
/// Both themes are configured because ThemeMode::System can switch between
/// them at runtime. The call is intentionally independent from
/// Context::set_theme, which only selects the active theme.
pub fn install_style(ctx: &egui::Context) {
    for dark in [true, false] {
        let tokens = Theme::of(dark);
        let mode = if dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        };
        ctx.style_mut_of(mode, |style| {
            style.visuals = theme::visuals(tokens);
            style_spacing(style);
        });
    }
}

fn style_spacing(style: &mut egui::Style) {
    style.spacing.item_spacing = Vec2::new(theme::SPACE_2, 6.0);
    style.spacing.button_padding = Vec2::new(theme::SPACE_3, 6.0);
    style.spacing.interact_size.y = theme::CONTROL_HEIGHT;
    style.spacing.window_margin = egui::Margin::same(theme::SPACE_3 as i8);
    style.spacing.menu_margin = egui::Margin::same(theme::SPACE_2 as i8);
    style.spacing.scroll.bar_width = 8.0;
    style.spacing.scroll.floating = false;
    style.spacing.scroll.bar_inner_margin = 2.0;
    style.spacing.scroll.bar_outer_margin = 2.0;
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::proportional(theme::FONT_BODY),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::proportional(theme::FONT_BODY),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::proportional(theme::FONT_CAPTION),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::proportional(theme::FONT_H1),
    );
    style
        .text_styles
        .insert(egui::TextStyle::Monospace, egui::FontId::monospace(12.0));
}

/// A page section: a 24 px gap plus the section title.
pub fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(theme::SECTION_GAP);
    ui.label(
        RichText::new(title)
            .size(theme::FONT_H2)
            .strong()
            .color(theme::theme(ui).text_primary),
    );
    ui.add_space(theme::SPACE_2);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeKind {
    Ok,
    Warning,
    Error,
    Info,
}

impl BadgeKind {
    fn tone(self, t: Theme) -> egui::Color32 {
        match self {
            Self::Ok => t.ok,
            Self::Warning => t.warn,
            Self::Error => t.err,
            Self::Info => t.info,
        }
    }
}

/// Non-scrolling page section.
///
/// Page-level scrolling is owned by the shell (ui::show_page), so the page
/// header and the sidebar stay put; panels only wrap their own content.
pub fn page_body(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    add(ui);
}

pub fn card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    card_response(ui, add).inner
}

pub fn card_response<R>(
    ui: &mut egui::Ui,
    add: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    theme::theme(ui).card_frame().show(ui, |ui| {
        // 卡片默认按内容收缩：在单列布局里会缩成一小块，右侧留白。
        // 这里让卡片至少占满父容器给它的宽度，页面才会是整列对齐。
        let width = ui.available_width();
        if width.is_finite() {
            ui.set_min_width(width);
        }
        add(ui)
    })
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
    let t = theme::theme(ui);
    let tone = kind.tone(t);
    egui::Frame::new()
        .fill(tone.gamma_multiply(0.18))
        .corner_radius(egui::CornerRadius::same(theme::RADIUS_CONTROL))
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(
                RichText::new(text)
                    .color(tone)
                    .size(theme::FONT_CAPTION)
                    .strong(),
            );
        })
        .response
}

pub fn badge(ui: &mut egui::Ui, text: impl Into<String>, kind: BadgeKind) {
    let _ = badge_response(ui, text, kind);
}

/// A borderless icon+ttext button used in toolbars and rows.
pub fn icon_text_button(ui: &mut egui::Ui, icon: Icon, text: &str) -> egui::Response {
    let t = theme::theme(ui);
    let galley = ui
        .painter()
        .layout_no_wrap(format!("  {text}"), t.body_font(), t.text_primary);
    let size = Vec2::new(galley.size().x + 20.0, theme::CONTROL_HEIGHT);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let fill = if response.hovered() {
        t.raised
    } else {
        egui::Color32::TRANSPARENT
    };
    let painter = ui.painter();
    painter.rect_filled(rect, egui::CornerRadius::same(theme::RADIUS_CONTROL), fill);
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 16.0, rect.center().y),
        Vec2::splat(16.0),
    );
    theme::paint_icon(painter, icon_rect, icon, t.text_secondary);
    painter.galley(
        egui::pos2(rect.left() + 26.0, rect.center().y - galley.size().y / 2.0),
        galley,
        t.text_primary,
    );
    response
}

/// A small square icon-only button.
pub fn icon_button(ui: &mut egui::Ui, icon: Icon, hover: &str) -> egui::Response {
    let t = theme::theme(ui);
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(24.0), Sense::click());
    let fill = if response.hovered() {
        t.raised
    } else {
        egui::Color32::TRANSPARENT
    };
    let painter = ui.painter();
    painter.rect_filled(rect, egui::CornerRadius::same(theme::RADIUS_CONTROL), fill);
    let color = if response.hovered() {
        t.text_primary
    } else {
        t.text_tertiary
    };
    theme::paint_icon(painter, rect.shrink(5.0), icon, color);
    response.on_hover_text(hover)
}

pub fn copy_button(ui: &mut egui::Ui, i18n: I18n, text: impl Into<String>) {
    let text = text.into();
    if icon_text_button(ui, Icon::Copy, i18n.t("app.copy")).clicked() {
        ui.ctx().copy_text(text);
    }
}

pub fn copy_table_button(ui: &mut egui::Ui, i18n: I18n, markdown: String) {
    if icon_text_button(ui, Icon::Copy, i18n.t("export.markdown_table")).clicked() {
        ui.ctx().copy_text(markdown);
    }
}

/// Definition list: right aligned caption label, body value, hover copy.
pub fn kv_grid(ui: &mut egui::Ui, id: &str, rows: &[(&str, String)]) {
    let t = theme::theme(ui);
    let _ = id;
    for (key, value) in rows {
        let shown = if value.is_empty() {
            "\u{2014}"
        } else {
            value.as_str()
        };
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(theme::KV_LABEL_WIDTH, theme::ROW_HEIGHT),
                Layout::right_to_left(Align::Center),
                |ui| {
                    ui.label(
                        RichText::new(*key)
                            .size(theme::FONT_CAPTION)
                            .color(t.text_secondary),
                    );
                },
            );
            ui.add_space(theme::SPACE_3);
            let available = (ui.available_width() - 30.0).max(80.0);
            ui.allocate_ui_with_layout(
                Vec2::new(available, theme::ROW_HEIGHT),
                Layout::left_to_right(Align::Center),
                |ui| {
                    let response = ui.add(
                        egui::Label::new(
                            RichText::new(shown)
                                .size(theme::FONT_BODY)
                                .color(t.text_primary),
                        )
                        .truncate(),
                    );
                    if shown != value.as_str() {
                        response.on_hover_text(shown);
                    } else if !value.is_empty() {
                        response.on_hover_text(value);
                    }
                },
            );
            if !value.is_empty() {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if icon_button(ui, Icon::Copy, "copy").clicked() {
                        ui.ctx().copy_text(value.clone());
                    }
                });
            }
        });
        let line = egui::Rect::from_min_size(ui.cursor().min, Vec2::new(ui.available_width(), 1.0));
        ui.painter()
            .rect_filled(line, egui::CornerRadius::ZERO, t.border_subtle);
        ui.add_space(1.0);
    }
}

/// One cell of a [`data_table`] row.
///
/// Cells are painted by the table itself so a whole row can highlight and react
/// as one unit, which egui::Grid cannot express.
pub enum Cell {
    Text(String),
    Strong(String),
    Muted(String),
    Mono(String),
    Badge(String, BadgeKind),
    Secondary { primary: String, secondary: String },
    Bar(f32),
}

impl Cell {
    /// Plain text used for clipboard copies and for the row context menu.
    pub fn copy_text(&self) -> String {
        match self {
            Cell::Text(text)
            | Cell::Strong(text)
            | Cell::Muted(text)
            | Cell::Mono(text)
            | Cell::Badge(text, _) => text.clone(),
            Cell::Secondary { primary, secondary } => {
                if secondary.is_empty() {
                    primary.clone()
                } else {
                    format!("{primary} ({secondary})")
                }
            }
            Cell::Bar(fraction) => format!("{:.0}%", fraction * 100.0),
        }
    }
}

/// One row of a [`data_table`].
pub struct Row {
    pub cells: Vec<Cell>,
    pub copy: String,
    pub tint: Option<egui::Color32>,
    pub action: Option<(Icon, String)>,
}

impl Row {
    pub fn new(cells: Vec<Cell>) -> Self {
        let copy = cells
            .iter()
            .map(Cell::copy_text)
            .collect::<Vec<_>>()
            .join("\t");
        Self {
            cells,
            copy,
            tint: None,
            action: None,
        }
    }

    /// Optional 4 px colour bar on the leading edge (used by the compare page).
    pub fn with_tint(mut self, tint: egui::Color32) -> Self {
        self.tint = Some(tint);
        self
    }

    /// Optional trailing icon action, for example the snapshot row menu.
    pub fn with_action(mut self, icon: Icon, hover: impl Into<String>) -> Self {
        self.action = Some((icon, hover.into()));
        self
    }
}

/// What the user did with a [`data_table`] this frame.
#[derive(Default)]
pub struct TableResult {
    pub clicked_row: Option<usize>,
    pub double_clicked_row: Option<usize>,
    pub action_row: Option<usize>,
    /// Screen position of the clicked row action, so callers can anchor a menu.
    pub action_pos: Option<egui::Pos2>,
}

fn fit_cell_text(
    painter: &egui::Painter,
    text: &str,
    font: egui::FontId,
    max_width: f32,
) -> (String, bool) {
    let probe = |value: &str| {
        painter
            .layout_no_wrap(value.to_owned(), font.clone(), egui::Color32::WHITE)
            .size()
            .x
    };
    if probe(text) <= max_width {
        return (text.to_owned(), false);
    }
    let budget = (max_width - probe("\u{2026}")).max(0.0);
    let mut shown = String::new();
    for character in text.chars() {
        shown.push(character);
        if probe(&shown) > budget {
            shown.pop();
            break;
        }
    }
    shown.push('\u{2026}');
    (shown, true)
}

fn paint_cell_text(
    painter: &egui::Painter,
    rect: egui::Rect,
    top: f32,
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
    truncated: &mut Option<String>,
) {
    let (shown, elided) = fit_cell_text(painter, text, font.clone(), rect.width());
    if elided && truncated.is_none() {
        *truncated = Some(text.to_owned());
    }
    painter.text(
        egui::pos2(rect.left(), top),
        egui::Align2::LEFT_TOP,
        shown,
        font,
        color,
    );
}

fn paint_cell(
    ui: &egui::Ui,
    t: &Theme,
    rect: egui::Rect,
    cell: &Cell,
    truncated: &mut Option<String>,
) {
    let painter = ui.painter();
    let line_top = rect.center().y - theme::FONT_BODY / 2.0 - 1.0;
    match cell {
        Cell::Text(text) => paint_cell_text(
            painter,
            rect,
            line_top,
            text,
            egui::FontId::proportional(theme::FONT_BODY),
            t.text_primary,
            truncated,
        ),
        Cell::Strong(text) => paint_cell_text(
            painter,
            rect,
            line_top,
            text,
            egui::FontId::proportional(theme::FONT_BODY),
            t.text_primary,
            truncated,
        ),
        Cell::Muted(text) => paint_cell_text(
            painter,
            rect,
            line_top + 1.0,
            text,
            egui::FontId::proportional(theme::FONT_CAPTION),
            t.text_tertiary,
            truncated,
        ),
        Cell::Mono(text) => paint_cell_text(
            painter,
            rect,
            line_top,
            text,
            egui::FontId::monospace(theme::FONT_CAPTION),
            t.text_secondary,
            truncated,
        ),
        Cell::Badge(text, kind) => {
            let tone = kind.tone(*t);
            let font = egui::FontId::proportional(theme::FONT_CAPTION);
            let galley = painter.layout_no_wrap(text.clone(), font, tone);
            let width = (galley.size().x + theme::SPACE_3).min(rect.width());
            let badge = egui::Rect::from_min_size(
                egui::pos2(rect.left(), rect.center().y - 9.0),
                Vec2::new(width, 18.0),
            );
            painter.rect_filled(
                badge,
                egui::CornerRadius::same(theme::RADIUS_CONTROL),
                tone.gamma_multiply(0.18),
            );
            painter.with_clip_rect(badge).galley(
                egui::pos2(
                    badge.left() + theme::SPACE_2 / 2.0,
                    badge.center().y - galley.size().y / 2.0,
                ),
                galley,
                tone,
            );
            if width + theme::SPACE_1 >= rect.width() && truncated.is_none() {
                *truncated = Some(text.clone());
            }
        }
        Cell::Secondary { primary, secondary } => {
            paint_cell_text(
                painter,
                rect,
                rect.top() + theme::SPACE_1,
                primary,
                egui::FontId::proportional(theme::FONT_BODY),
                t.text_primary,
                truncated,
            );
            if !secondary.is_empty() {
                paint_cell_text(
                    painter,
                    rect,
                    rect.top() + theme::SPACE_1 + theme::FONT_BODY + 2.0,
                    secondary,
                    egui::FontId::proportional(theme::FONT_CAPTION),
                    t.text_tertiary,
                    truncated,
                );
            }
        }
        Cell::Bar(fraction) => {
            let track = egui::Rect::from_center_size(rect.center(), Vec2::new(rect.width(), 6.0));
            painter.rect_filled(
                track,
                egui::CornerRadius::same(theme::RADIUS_CONTROL),
                t.border_subtle,
            );
            let filled = (fraction.clamp(0.0, 1.0) * track.width()).max(0.5);
            let fill = egui::Rect::from_min_size(track.min, Vec2::new(filled, track.height()));
            painter.rect_filled(
                fill,
                egui::CornerRadius::same(theme::RADIUS_CONTROL),
                t.accent,
            );
        }
    }
}

/// Table with whole-row hover, click-to-copy and a copy context menu.
///
/// Used by every hardware panel, the snapshot list and the compare page so the
/// behaviour (and the truncation tooltip) is defined exactly once.
pub fn data_table(
    ui: &mut egui::Ui,
    i18n: I18n,
    id: &str,
    headers: &[&str],
    weights: &[f32],
    rows: &[Row],
    sticky_header: bool,
) -> TableResult {
    let t = theme::theme(ui);
    let width = ui.available_width();
    let columns = headers.len().max(1);
    let has_action = rows.iter().any(|row| row.action.is_some());
    let has_tint = rows.iter().any(|row| row.tint.is_some());
    let action_width = if has_action { theme::SPACE_6 } else { 0.0 };
    let tint_width = if has_tint { theme::SPACE_1 } else { 0.0 };
    let gap = theme::SPACE_3;
    let usable =
        (width - action_width - tint_width - gap * columns.saturating_sub(1) as f32).max(120.0);
    let total_weight: f32 = (0..columns)
        .map(|index| weights.get(index).copied().unwrap_or(1.0))
        .sum::<f32>()
        .max(0.1);
    let widths: Vec<f32> = (0..columns)
        .map(|index| usable * weights.get(index).copied().unwrap_or(1.0) / total_weight)
        .collect();
    let header_height = 28.0;
    let mut result = TableResult::default();

    let paint_header = |ui: &egui::Ui, origin: egui::Pos2, background: Option<egui::Color32>| {
        let painter = ui.painter();
        let rect = egui::Rect::from_min_size(origin, Vec2::new(width, header_height));
        if let Some(color) = background {
            painter.rect_filled(rect, egui::CornerRadius::ZERO, color);
        }
        let mut x = origin.x + tint_width;
        for (index, header) in headers.iter().enumerate() {
            painter.text(
                egui::pos2(x, rect.center().y),
                egui::Align2::LEFT_CENTER,
                *header,
                egui::FontId::proportional(theme::FONT_CAPTION),
                t.text_tertiary,
            );
            x += widths[index] + gap;
        }
        painter.hline(
            rect.x_range(),
            rect.bottom(),
            Stroke::new(1.0, t.border_subtle),
        );
        rect
    };

    let header_origin = ui.cursor().min;
    ui.allocate_exact_size(Vec2::new(width, header_height), Sense::hover());
    let header_rect = paint_header(ui, header_origin, None);

    let mut ranges: Vec<(f32, f32)> = Vec::with_capacity(columns);
    let mut x = header_origin.x + tint_width;
    for column_width in widths.iter().take(columns) {
        ranges.push((x, x + column_width));
        x += column_width + gap;
    }

    for (index, row) in rows.iter().enumerate() {
        let origin = ui.cursor().min;
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, theme::ROW_HEIGHT), Sense::hover());
        let response = ui.interact(rect, ui.scope_id().with((id, index)), Sense::click());
        if response.hovered() {
            ui.painter()
                .rect_filled(rect, egui::CornerRadius::ZERO, t.accent_subtle);
        }
        if let Some(tint) = row.tint {
            let bar = egui::Rect::from_min_size(rect.min, Vec2::new(tint_width, rect.height()));
            ui.painter()
                .rect_filled(bar, egui::CornerRadius::ZERO, tint);
        }
        let mut truncated: Option<String> = None;
        let mut cell_x = origin.x + tint_width;
        for (column, cell) in row.cells.iter().enumerate().take(columns) {
            let cell_rect = egui::Rect::from_min_size(
                egui::pos2(cell_x, rect.top()),
                Vec2::new(widths[column], rect.height()),
            );
            paint_cell(ui, &t, cell_rect, cell, &mut truncated);
            cell_x += widths[column] + gap;
        }
        ui.painter().hline(
            rect.x_range(),
            rect.bottom(),
            Stroke::new(1.0, t.border_subtle),
        );

        if let Some((icon, hover)) = &row.action {
            let action_rect = egui::Rect::from_center_size(
                egui::pos2(rect.right() - action_width / 2.0, rect.center().y),
                Vec2::splat(theme::SPACE_6),
            );
            let action = ui.interact(
                action_rect,
                ui.scope_id().with((id, index, "action")),
                Sense::click(),
            );
            let color = if action.hovered() {
                t.text_primary
            } else {
                t.text_tertiary
            };
            theme::paint_icon(
                ui.painter(),
                action_rect.shrink(theme::SPACE_1),
                *icon,
                color,
            );
            if action.clicked() {
                result.action_row = Some(index);
                result.action_pos = Some(action_rect.left_bottom());
            }
            if action.hovered() {
                action.clone().on_hover_text(hover);
            }
        }

        if response.double_clicked() {
            result.double_clicked_row = Some(index);
        } else if response.clicked() {
            result.clicked_row = Some(index);
            if let Some(pointer) = response.interact_pointer_pos() {
                let column = ranges
                    .iter()
                    .position(|(start, end)| pointer.x >= *start && pointer.x <= *end)
                    .unwrap_or(0);
                if let Some(cell) = row.cells.get(column) {
                    ui.ctx().copy_text(cell.copy_text());
                }
            }
        }
        if response.secondary_clicked()
            && let Some(pointer) = response.interact_pointer_pos()
        {
            let column = ranges
                .iter()
                .position(|(start, end)| pointer.x >= *start && pointer.x <= *end)
                .unwrap_or(0);
            ui.memory_mut(|memory| {
                memory
                    .data
                    .insert_temp(egui::Id::unique((id, "context-cell")), column);
            });
        }
        if response.hovered()
            && let Some(text) = truncated
        {
            response.show_tooltip_text(text);
        }
        let menu_cell = egui::Id::unique((id, "context-cell"));
        response.context_menu(|ui| {
            let column = ui
                .memory(|memory| memory.data.get_temp::<usize>(menu_cell))
                .unwrap_or(0);
            if ui.button(i18n.t("table.copy_cell")).clicked() {
                if let Some(cell) = row.cells.get(column) {
                    ui.ctx().copy_text(cell.copy_text());
                }
                ui.close();
            }
            if ui.button(i18n.t("table.copy_row")).clicked() {
                ui.ctx().copy_text(row.copy.clone());
                ui.close();
            }
        });
    }

    if sticky_header {
        let clip = ui.clip_rect();
        if header_rect.bottom() < clip.top() && clip.top() < ui.cursor().top() {
            paint_header(
                ui,
                egui::pos2(header_rect.left(), clip.top()),
                Some(t.canvas),
            );
        }
    }

    result
}

/// Empty state: 32 px icon, 15 px title, 12 px description, optional action.
pub fn empty_state(
    ui: &mut egui::Ui,
    t: Theme,
    icon: Icon,
    title: &str,
    detail: &str,
    action: Option<&str>,
) -> bool {
    let mut clicked = false;
    ui.add_space(theme::SPACE_8);
    ui.vertical_centered(|ui| {
        theme::icon_widget(ui, icon, 32.0);
        ui.add_space(theme::SPACE_2);
        ui.label(
            RichText::new(title)
                .size(theme::FONT_H2)
                .color(t.text_primary),
        );
        if !detail.is_empty() {
            ui.label(
                RichText::new(detail)
                    .size(theme::FONT_CAPTION)
                    .color(t.text_secondary),
            );
        }
        if let Some(label) = action {
            ui.add_space(theme::SPACE_3);
            clicked = ui.add(primary_button(label).fill(t.accent)).clicked();
        }
    });
    ui.add_space(theme::SPACE_8);
    clicked
}

pub fn empty(ui: &mut egui::Ui, text: impl Into<String>) {
    let t = theme::theme(ui);
    let text: String = text.into();
    let _ = empty_state(ui, t, Icon::Info, &text, "", None);
}

/// A secondary warning block: caption text with a 3 px tone bar on the left.
pub fn warning_list(ui: &mut egui::Ui, i18n: I18n, warnings: &[String]) {
    let t = theme::theme(ui);
    for warning in warnings {
        ui.horizontal_top(|ui| {
            let (bar, _) =
                ui.allocate_exact_size(Vec2::new(3.0, theme::FONT_CAPTION + 4.0), Sense::hover());
            ui.painter()
                .rect_filled(bar, egui::CornerRadius::same(1), t.warn);
            ui.add_space(theme::SPACE_2);
            ui.label(
                RichText::new(warning)
                    .size(theme::FONT_CAPTION)
                    .color(t.text_secondary),
            );
        });
        ui.add_space(theme::SPACE_1);
    }
    let _ = i18n;
}

pub fn warnings(ui: &mut egui::Ui, i18n: I18n, id: &str, warnings: &[String]) {
    if warnings.is_empty() {
        return;
    }
    collapsible(
        ui,
        id,
        format!("{} ({})", i18n.t("app.warnings"), warnings.len()),
        false,
        |ui| {
            warning_list(ui, i18n, warnings);
        },
    );
}

/// Number of columns for a content grid at the current width.
/// Lays out two panes side by side, stacking them on narrow windows.
pub fn two_pane(
    ui: &mut egui::Ui,
    left: impl FnOnce(&mut egui::Ui),
    right: impl FnOnce(&mut egui::Ui),
) {
    if columns(ui) >= 2 {
        ui.columns(2, |panes| {
            left(&mut panes[0]);
            right(&mut panes[1]);
        });
    } else {
        left(ui);
        ui.add_space(theme::SPACE_3);
        right(ui);
    }
}

pub fn columns(ui: &egui::Ui) -> usize {
    let width = ui.available_width();
    if width >= 1120.0 {
        4
    } else if width >= 840.0 {
        3
    } else if width >= 560.0 {
        2
    } else {
        1
    }
}

/// A responsive row of metrics. Cards are not involved: a metric is a block of
/// text, so it sits directly on the canvas.
pub fn metric_grid(ui: &mut egui::Ui, metrics: &[(&str, String, Option<String>)]) {
    if metrics.is_empty() {
        return;
    }
    let spacing = theme::SPACE_6;
    let width = ui.available_width();
    // 指标数量固定（一般为 4 个），列数要按"放得下几个"来定，
    // 否则 4 个指标在 3 列断点会把最后一个挤到下一行。
    let mut cols = 1;
    for candidate in (1..=metrics.len().min(4)).rev() {
        let cell = (width - spacing * (candidate as f32 - 1.0)) / candidate as f32;
        if cell >= 200.0 {
            cols = candidate;
            break;
        }
    }
    let cell = ((width - spacing * (cols as f32 - 1.0)) / cols as f32).max(120.0);
    egui::Grid::new("metric-grid")
        .num_columns(cols)
        .spacing([spacing, theme::SPACE_4])
        .show(ui, |ui| {
            for (index, (label, value, detail)) in metrics.iter().enumerate() {
                ui.allocate_ui_with_layout(
                    Vec2::new(cell, 0.0),
                    Layout::top_down(Align::Min),
                    |ui| {
                        // 子 Ui 必须自己撑满格宽，否则 Grid 会按内容收缩，
                        // 四个指标就会一个比一个靠左。
                        ui.set_min_width(cell);
                        metric(ui, label, value.clone(), detail.clone());
                    },
                );
                if (index + 1) % cols == 0 {
                    ui.end_row();
                }
            }
        });
}

/// 页面级警告入口：贴在页签行右端，点开是明细。
///
/// 之前是一整条横跨页面宽度的横幅，和居中的单列表单列错开，看上去
/// 像是布局没对齐；收进页签行后，页面只剩一条竖直的内容轴。
pub fn warnings_chip(ui: &mut egui::Ui, i18n: I18n, failures: &[(String, String)], total: usize) {
    if total == 0 {
        return;
    }
    let t = theme::theme(ui);
    let title = RichText::new(format!("{} ({total})", i18n.t("warnings.summary")))
        .size(theme::FONT_CAPTION)
        .color(t.warn);
    if failures.is_empty() {
        ui.label(title);
        return;
    }
    ui.menu_button(title, |ui| {
        ui.set_min_width(420.0);
        ui.set_max_width(560.0);
        for (module, message) in failures {
            ui.horizontal_top(|ui| {
                let (bar, _) = ui
                    .allocate_exact_size(Vec2::new(3.0, theme::FONT_CAPTION + 4.0), Sense::hover());
                ui.painter()
                    .rect_filled(bar, egui::CornerRadius::same(1), t.warn);
                ui.add_space(theme::SPACE_2);
                ui.label(
                    RichText::new(format!("{module} · {message}"))
                        .size(theme::FONT_CAPTION)
                        .color(t.text_secondary),
                );
            });
            ui.add_space(theme::SPACE_1);
        }
    });
}

/// A metric: caption label, numeral value, caption detail.
pub fn metric(
    ui: &mut egui::Ui,
    label: &str,
    value: impl Into<String>,
    detail: Option<String>,
) -> egui::Response {
    let t = theme::theme(ui);
    ui.vertical(|ui| {
        ui.label(
            RichText::new(label)
                .size(theme::FONT_CAPTION)
                .color(t.text_secondary),
        );
        ui.add_space(theme::SPACE_1);
        ui.label(theme::numeral(value, t));
        if let Some(detail) = detail {
            ui.add_space(theme::SPACE_1);
            ui.add(
                egui::Label::new(
                    RichText::new(detail)
                        .size(theme::FONT_CAPTION)
                        .color(t.text_tertiary),
                )
                .truncate(),
            );
        }
    })
    .response
}

pub fn filter_input(ui: &mut egui::Ui, i18n: I18n, value: &mut String) -> egui::Response {
    ui.horizontal(|ui| {
        theme::icon_widget(ui, Icon::Search, 16.0);
        let response = ui.add(
            egui::TextEdit::singleline(value)
                .hint_text(i18n.t("app.search"))
                .desired_width(240.0)
                .margin(Vec2::new(theme::SPACE_2, 6.0)),
        );
        if !value.is_empty() && icon_button(ui, Icon::Close, i18n.t("app.clear")).clicked() {
            value.clear();
        }
        response
    })
    .inner
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SparkKind {
    Utilization,
    Memory,
}

/// A 2 px sparkline with a gradient fill, four grid lines and a hover
/// crosshair. The chart carries no title of its own: the heading lives above
/// the canvas so text never sits inside the plot.
pub fn sparkline(
    ui: &mut egui::Ui,
    id: &str,
    values: &[f32],
    maximum: f32,
    kind: SparkKind,
    height: f32,
) {
    let t = theme::theme(ui);
    let color = match kind {
        SparkKind::Utilization => t.info,
        SparkKind::Memory => t.warn,
    };
    let width = ui.available_width().max(180.0);
    let (response, painter) = ui.allocate_painter(Vec2::new(width, height), Sense::hover());
    let rect = response.rect;
    painter.rect_filled(rect, egui::CornerRadius::same(theme::RADIUS_CARD), t.raised);

    for step in 1..=4 {
        let y = rect.top() + rect.height() * step as f32 / 5.0;
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            Stroke::new(1.0, t.border_subtle),
        );
    }
    painter.line_segment(
        [
            egui::pos2(rect.left(), rect.bottom() - 1.0),
            egui::pos2(rect.right(), rect.bottom() - 1.0),
        ],
        Stroke::new(1.0, t.border_strong),
    );

    if values.len() < 2 {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "—",
            t.body_font(),
            t.text_tertiary,
        );
        return;
    }

    let maximum = maximum.max(1.0);
    let inner = rect.shrink2(Vec2::new(theme::SPACE_2, theme::SPACE_2));
    let step = inner.width() / (values.len() - 1) as f32;
    let points: Vec<egui::Pos2> = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let normalized = (value / maximum).clamp(0.0, 1.0);
            egui::pos2(
                inner.left() + index as f32 * step,
                inner.bottom() - normalized * inner.height(),
            )
        })
        .collect();

    let mut mesh = egui::Mesh::default();
    let transparent = color.gamma_multiply(0.0);
    for pair in points.windows(2) {
        let (left, right) = (pair[0], pair[1]);
        let base = mesh.vertices.len() as u32;
        mesh.colored_vertex(left, color.gamma_multiply(0.22));
        mesh.colored_vertex(right, color.gamma_multiply(0.22));
        mesh.colored_vertex(egui::pos2(right.x, inner.bottom()), transparent);
        mesh.colored_vertex(egui::pos2(left.x, inner.bottom()), transparent);
        mesh.add_triangle(base, base + 1, base + 2);
        mesh.add_triangle(base, base + 2, base + 3);
    }
    painter.add(egui::Shape::mesh(mesh));
    painter.add(egui::Shape::line(points.clone(), Stroke::new(2.0, color)));

    if let Some(value) = values.last() {
        painter.text(
            rect.right_top() + Vec2::new(-theme::SPACE_2, theme::SPACE_2),
            egui::Align2::RIGHT_TOP,
            format!("{value:.1}"),
            t.caption_font(),
            t.text_primary,
        );
    }

    if let Some(pointer) = response.hover_pos() {
        let x = pointer.x.clamp(inner.left(), inner.right());
        painter.line_segment(
            [egui::pos2(x, inner.top()), egui::pos2(x, inner.bottom())],
            Stroke::new(1.0, t.border_strong),
        );
        let index = (((x - inner.left()) / step).round() as usize).min(values.len() - 1);
        painter.circle_filled(points[index], 2.5, color);
        response.on_hover_text(format!("{:.1}", values[index]));
    }
    let _ = id;
}

pub fn percent_bar(ui: &mut egui::Ui, label: &str, percent: f32) {
    let t = theme::theme(ui);
    ui.label(
        RichText::new(format!("{label}: {percent:.1}%"))
            .size(theme::FONT_CAPTION)
            .color(t.text_secondary),
    );
    ui.add(
        egui::ProgressBar::new((percent / 100.0).clamp(0.0, 1.0))
            .desired_height(8.0)
            .corner_radius(egui::CornerRadius::same(theme::RADIUS_CONTROL))
            .fill(t.accent),
    );
}

/// Primary action: accent fill with a white label.
pub fn primary_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text.to_owned()).color(egui::Color32::WHITE))
}

/// Secondary action: the default framed button.
pub fn secondary_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(text.to_owned())
}

/// Ghost action: no frame until hovered, for row and toolbar actions.
pub fn ghost_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(text.to_owned()).frame(false)
}

/// Horizontal tab strip for the pages of the current navigation section.
/// Returns the page the user selected, if any.
pub fn tabs(ui: &mut egui::Ui, i18n: I18n, pages: &'static [Page], current: Page) -> Option<Page> {
    let t = theme::theme(ui);
    let mut selected = None;
    ui.horizontal(|ui| {
        for page in pages {
            let is_current = *page == current;
            let response = ui.add(
                egui::Button::new(
                    RichText::new(i18n.t(page.title_key()))
                        .size(theme::FONT_BODY)
                        .color(if is_current {
                            t.text_primary
                        } else {
                            t.text_secondary
                        }),
                )
                .frame(false)
                .min_size(Vec2::new(0.0, theme::CONTROL_HEIGHT)),
            );
            if is_current {
                let line = egui::Rect::from_min_size(
                    egui::pos2(response.rect.left(), response.rect.bottom() - 2.0),
                    Vec2::new(response.rect.width(), 2.0),
                );
                ui.painter()
                    .rect_filled(line, egui::CornerRadius::same(1), t.accent);
            }
            if response.clicked() && !is_current {
                selected = Some(*page);
            }
            ui.add_space(theme::SPACE_2);
        }
    });
    ui.add_space(theme::SPACE_2);
    selected
}
