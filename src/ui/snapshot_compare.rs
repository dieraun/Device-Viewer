//! Snapshot comparison with added / removed / changed highlighting.

use egui::{Align2, FontId, Rect, RichText};
use serde_json::Value;

use crate::i18n::I18n;
use crate::state::AppState;
use crate::ui::theme;
use crate::ui::widgets;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChangeKind {
    Added,
    Removed,
    Changed,
}

impl ChangeKind {
    fn label(self, i18n: I18n) -> &'static str {
        match self {
            Self::Added => i18n.t("compare.added"),
            Self::Removed => i18n.t("compare.removed"),
            Self::Changed => i18n.t("compare.changed"),
        }
    }

    fn color(self, t: theme::Theme) -> egui::Color32 {
        match self {
            Self::Added => t.ok,
            Self::Removed => t.err,
            Self::Changed => t.warn,
        }
    }
}

struct DiffRow {
    path: String,
    kind: ChangeKind,
    before: String,
    after: String,
}

/// Splits "a.b[2].c" into the leaf name and the remaining parent path.
fn split_path(path: &str) -> (&str, &str) {
    match path.rfind('.') {
        Some(index) => (&path[index + 1..], &path[..index]),
        None => (path, ""),
    }
}

fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let mut short: String = text.chars().take(max_chars.saturating_sub(1)).collect();
        short.push('…');
        short
    }
}

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    let t = theme::theme(ui);
    widgets::card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(i18n.t("compare.baseline"))
                    .size(theme::FONT_CAPTION)
                    .color(t.text_tertiary),
            );
            let baseline_name = state
                .compare_file
                .clone()
                .unwrap_or_else(|| state.compare_label.clone());
            ui.label(
                RichText::new(if baseline_name.is_empty() {
                    i18n.t("app.none").to_string()
                } else {
                    baseline_name
                })
                .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(i18n.t("snapshot.choose")).clicked() {
                    state.settings.page = crate::state::Page::Snapshots;
                }
            });
        });
    });

    let Some(before) = state.compare.clone() else {
        widgets::empty_state(
            ui,
            &t,
            theme::Icon::Snapshot,
            i18n.t("compare.no_baseline"),
            i18n.t("snapshot.choose"),
            None,
        );
        return;
    };
    let before = serde_json::to_value(&before).unwrap_or(Value::Null);
    let after = serde_json::to_value(&state.snapshot).unwrap_or(Value::Null);

    let mut rows = Vec::new();
    diff_values("", &before, &after, &mut rows);

    let added = rows.iter().filter(|r| r.kind == ChangeKind::Added).count();
    let removed = rows
        .iter()
        .filter(|r| r.kind == ChangeKind::Removed)
        .count();
    let changed = rows
        .iter()
        .filter(|r| r.kind == ChangeKind::Changed)
        .count();

    ui.add_space(theme::SPACE_4);
    ui.horizontal(|ui| {
        for kind in [
            None,
            Some(ChangeKind::Added),
            Some(ChangeKind::Removed),
            Some(ChangeKind::Changed),
        ] {
            let (label, count) = match kind {
                None => (i18n.t("compare.all"), rows.len()),
                Some(kind) => (
                    kind.label(i18n),
                    match kind {
                        ChangeKind::Added => added,
                        ChangeKind::Removed => removed,
                        ChangeKind::Changed => changed,
                    },
                ),
            };
            let selected = match kind {
                None => {
                    !state.ui.compare_show_added
                        && !state.ui.compare_show_removed
                        && !state.ui.compare_show_changed
                }
                Some(ChangeKind::Added) => state.ui.compare_show_added,
                Some(ChangeKind::Removed) => state.ui.compare_show_removed,
                Some(ChangeKind::Changed) => state.ui.compare_show_changed,
            };
            if ui
                .selectable_label(selected, format!("{label} {count}"))
                .clicked()
            {
                let (a, r, c) = match kind {
                    None => (false, false, false),
                    Some(ChangeKind::Added) => (true, false, false),
                    Some(ChangeKind::Removed) => (false, true, false),
                    Some(ChangeKind::Changed) => (false, false, true),
                };
                state.ui.compare_show_added = a;
                state.ui.compare_show_removed = r;
                state.ui.compare_show_changed = c;
            }
        }
        ui.add_space(theme::SPACE_3);
        widgets::filter_input(ui, i18n, &mut state.ui.compare_filter);
    });

    let filter = state.ui.compare_filter.to_lowercase();
    let no_kind_filter = !state.ui.compare_show_added
        && !state.ui.compare_show_removed
        && !state.ui.compare_show_changed;
    rows.retain(|row| {
        let kind_ok = match row.kind {
            ChangeKind::Added => no_kind_filter || state.ui.compare_show_added,
            ChangeKind::Removed => no_kind_filter || state.ui.compare_show_removed,
            ChangeKind::Changed => no_kind_filter || state.ui.compare_show_changed,
        };
        kind_ok && (filter.is_empty() || row.path.to_lowercase().contains(&filter))
    });

    ui.add_space(theme::SPACE_4);
    if rows.is_empty() {
        widgets::empty_state(
            ui,
            &t,
            theme::Icon::Check,
            i18n.t("compare.identical"),
            i18n.t("compare.no_changes"),
            None,
        );
        return;
    }

    let total = ui.available_width();
    let before_width = (total * 0.24).clamp(140.0, 320.0);
    let after_width = before_width;
    let bar_width = 4.0;
    let field_width =
        (total - bar_width - before_width - after_width - theme::SPACE_3 * 4.0).max(180.0);

    // The header sits outside the scroll area so it stays visible.
    ui.horizontal(|ui| {
        ui.add_space(bar_width + theme::SPACE_2);
        let mut header = |ui: &mut egui::Ui, text: &str, width: f32| {
            ui.allocate_ui_with_layout(
                egui::vec2(width, theme::ROW_HEIGHT),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.label(
                        RichText::new(text)
                            .size(theme::FONT_CAPTION)
                            .color(t.text_tertiary),
                    );
                },
            );
        };
        header(ui, i18n.t("compare.field"), field_width);
        header(ui, i18n.t("compare.change"), theme::SPACE_8 * 3.0);
        header(ui, i18n.t("compare.before"), before_width);
        header(ui, i18n.t("compare.after"), after_width);
    });
    ui.add_space(theme::SPACE_1);
    ui.separator();

    // 特例：表头已固定在滚动区外，这里的滚动容器只负责行区，是「全站单层滚动」
    // 规则的唯一例外（1.3.0 验收裁决第 3 条），合并内层滚动会让表头一起滚走。
    egui::ScrollArea::vertical()
        .id_salt("compare-rows")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for row in &rows {
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(total, theme::ROW_HEIGHT),
                    egui::Sense::click(),
                );
                let painter = ui.painter().with_clip_rect(rect);
                if response.hovered() {
                    painter.rect_filled(rect, egui::CornerRadius::ZERO, t.accent_subtle);
                }
                painter.rect_filled(
                    Rect::from_min_size(rect.min, egui::vec2(bar_width, rect.height())),
                    egui::CornerRadius::ZERO,
                    row.kind.color(t),
                );

                let mut x = rect.left() + bar_width + theme::SPACE_2;
                let (leaf, parent) = split_path(&row.path);
                painter.text(
                    egui::pos2(x, rect.center().y - 6.0),
                    Align2::LEFT_CENTER,
                    truncate(leaf, 40),
                    FontId::new(theme::FONT_BODY, egui::FontFamily::Proportional),
                    t.text_primary,
                );
                painter.text(
                    egui::pos2(x, rect.center().y + 8.0),
                    Align2::LEFT_CENTER,
                    truncate(parent, 48),
                    FontId::new(theme::FONT_CAPTION, egui::FontFamily::Monospace),
                    t.text_tertiary,
                );
                if response.hovered() {
                    response.clone().on_hover_text(&row.path);
                }

                x += field_width + theme::SPACE_3;
                let chip = Rect::from_min_size(
                    egui::pos2(x, rect.center().y - 10.0),
                    egui::vec2(64.0, 20.0),
                );
                painter.rect_filled(
                    chip,
                    theme::RADIUS_CONTROL,
                    row.kind.color(t).gamma_multiply(0.22),
                );
                painter.text(
                    chip.center(),
                    Align2::CENTER_CENTER,
                    row.kind.label(i18n),
                    FontId::new(theme::FONT_CAPTION, egui::FontFamily::Proportional),
                    row.kind.color(t),
                );

                x += 64.0 + theme::SPACE_3;
                painter.text(
                    egui::pos2(x, rect.center().y),
                    Align2::LEFT_CENTER,
                    truncate(&row.before, (before_width / 7.0) as usize),
                    FontId::new(theme::FONT_BODY, egui::FontFamily::Proportional),
                    t.text_secondary,
                );
                x += before_width + theme::SPACE_3;
                painter.text(
                    egui::pos2(x, rect.center().y),
                    Align2::LEFT_CENTER,
                    truncate(&row.after, (after_width / 7.0) as usize),
                    FontId::new(theme::FONT_BODY, egui::FontFamily::Proportional),
                    t.text_primary,
                );

                if response.clicked() {
                    ui.ctx()
                        .copy_text(format!("{}	{}	{}", row.path, row.before, row.after));
                    state.ui.notify(i18n.t("toast.copied"));
                }
            }
        });
}

fn diff_values(path: &str, before: &Value, after: &Value, rows: &mut Vec<DiffRow>) {
    match (before, after) {
        (Value::Object(left), Value::Object(right)) => {
            for (key, value) in left {
                let child = format!("{path}.{key}");
                match right.get(key) {
                    Some(other) => diff_values(&child, value, other, rows),
                    None => rows.push(DiffRow {
                        path: child,
                        kind: ChangeKind::Removed,
                        before: compact(value),
                        after: String::new(),
                    }),
                }
            }
            for (key, value) in right {
                if !left.contains_key(key) {
                    rows.push(DiffRow {
                        path: format!("{path}.{key}"),
                        kind: ChangeKind::Added,
                        before: String::new(),
                        after: compact(value),
                    });
                }
            }
        }
        (Value::Array(left), Value::Array(right)) => {
            let length = left.len().max(right.len());
            for index in 0..length {
                let child = format!("{path}[{index}]");
                match (left.get(index), right.get(index)) {
                    (Some(value), Some(other)) => diff_values(&child, value, other, rows),
                    (Some(value), None) => rows.push(DiffRow {
                        path: child,
                        kind: ChangeKind::Removed,
                        before: compact(value),
                        after: String::new(),
                    }),
                    (None, Some(value)) => rows.push(DiffRow {
                        path: child,
                        kind: ChangeKind::Added,
                        before: String::new(),
                        after: compact(value),
                    }),
                    (None, None) => {}
                }
            }
        }
        _ if before != after => rows.push(DiffRow {
            path: path.to_string(),
            kind: ChangeKind::Changed,
            before: compact(before),
            after: compact(after),
        }),
        _ => {}
    }
}

fn compact(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::String(text) => text.clone(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => number.to_string(),
        Value::Array(items) => {
            if items.len() > 4 {
                format!("[{} items]", items.len())
            } else {
                serde_json::to_string(items).unwrap_or_default()
            }
        }
        Value::Object(map) => {
            if map.len() > 3 {
                format!("{{{} fields}}", map.len())
            } else {
                serde_json::to_string(map).unwrap_or_default()
            }
        }
    }
}
