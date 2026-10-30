//! Snapshot comparison with added / removed / changed highlighting.

use egui::RichText;
use serde_json::Value;

use crate::i18n::I18n;
use crate::state::AppState;
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

    fn color(self) -> egui::Color32 {
        match self {
            Self::Added => egui::Color32::from_rgb(90, 200, 130),
            Self::Removed => egui::Color32::from_rgb(230, 95, 95),
            Self::Changed => egui::Color32::from_rgb(230, 185, 80),
        }
    }
}

struct DiffRow {
    path: String,
    kind: ChangeKind,
    before: String,
    after: String,
}

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    widgets::page_header(ui, i18n, "nav.compare", "page.compare.sub");

    ui.horizontal(|ui| {
        ui.label(i18n.t("compare.baseline"));
        let baseline_name = state
            .compare_file
            .clone()
            .unwrap_or_else(|| state.compare_label.clone());
        ui.monospace(if baseline_name.is_empty() {
            i18n.t("app.none").to_string()
        } else {
            baseline_name
        });
        if ui.button(i18n.t("snapshot.choose")).clicked() {
            state.settings.page = crate::state::Page::Snapshots;
        }
    });
    widgets::filter_input(ui, i18n, &mut state.ui.compare_filter);
    ui.add_space(6.0);

    let Some(before) = state.compare.clone() else {
        widgets::empty(ui, i18n.t("compare.no_baseline"));
        return;
    };
    let before = serde_json::to_value(&before).unwrap_or(Value::Null);
    let after = serde_json::to_value(&state.snapshot).unwrap_or(Value::Null);

    let mut rows = Vec::new();
    diff_values("", &before, &after, &mut rows);
    let filter = state.ui.compare_filter.to_lowercase();
    rows.retain(|row| filter.is_empty() || row.path.to_lowercase().contains(&filter));

    let added = rows
        .iter()
        .filter(|row| row.kind == ChangeKind::Added)
        .count();
    let removed = rows
        .iter()
        .filter(|row| row.kind == ChangeKind::Removed)
        .count();
    let changed = rows
        .iter()
        .filter(|row| row.kind == ChangeKind::Changed)
        .count();

    ui.horizontal(|ui| {
        widgets::badge(ui, format!("+{added}"), widgets::BadgeKind::Ok);
        widgets::badge(ui, format!("-{removed}"), widgets::BadgeKind::Error);
        widgets::badge(ui, format!("~{changed}"), widgets::BadgeKind::Warning);
    });
    ui.add_space(6.0);

    if rows.is_empty() {
        widgets::empty(ui, i18n.t("compare.identical"));
        return;
    }

    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Grid::new("compare-grid")
                .num_columns(4)
                .striped(true)
                .spacing([14.0, 5.0])
                .show(ui, |ui| {
                    for header in [
                        i18n.t("compare.field"),
                        i18n.t("compare.change"),
                        i18n.t("compare.before"),
                        i18n.t("compare.after"),
                    ] {
                        ui.label(RichText::new(header).weak());
                    }
                    ui.end_row();
                    for row in &rows {
                        ui.monospace(&row.path);
                        ui.label(
                            RichText::new(row.kind.label(i18n))
                                .color(row.kind.color())
                                .strong(),
                        );
                        ui.label(&row.before);
                        ui.label(&row.after);
                        ui.end_row();
                    }
                });
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
