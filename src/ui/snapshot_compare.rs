//! Snapshot comparison with added / removed / changed highlighting.

use egui::RichText;
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

    fn badge(self) -> widgets::BadgeKind {
        match self {
            Self::Added => widgets::BadgeKind::Ok,
            Self::Removed => widgets::BadgeKind::Error,
            Self::Changed => widgets::BadgeKind::Warning,
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
            t,
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
            t,
            theme::Icon::Check,
            i18n.t("compare.identical"),
            i18n.t("compare.no_changes"),
            None,
        );
        return;
    }

    // 与其它面板共用同一张表：整行 hover、左侧变更色条、单击复制、右键复制字段/整行。
    // 表头吸顶由 data_table 在滚动区顶部按需重绘实现，页面保持全站单层滚动。
    let table_rows: Vec<widgets::Row> = rows
        .iter()
        .map(|row| {
            let (leaf, parent) = split_path(&row.path);
            let mut table_row = widgets::Row::new(vec![
                widgets::Cell::Secondary {
                    primary: leaf.to_string(),
                    secondary: parent.to_string(),
                },
                widgets::Cell::Badge(row.kind.label(i18n).to_string(), row.kind.badge()),
                widgets::Cell::Text(row.before.clone()),
                widgets::Cell::Text(row.after.clone()),
            ]);
            table_row.copy = format!("{}\t{}\t{}", row.path, row.before, row.after);
            table_row.with_tint(row.kind.color(t))
        })
        .collect();
    let table = widgets::data_table(
        ui,
        i18n,
        "compare-table",
        &[
            i18n.t("compare.field"),
            i18n.t("compare.change"),
            i18n.t("compare.before"),
            i18n.t("compare.after"),
        ],
        &[2.6, 0.9, 2.0, 2.0],
        &table_rows,
        true,
    );
    if table.clicked_row.is_some() {
        state.ui.notify(i18n.t("toast.copied"));
    }
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
