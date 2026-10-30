//! Saved snapshots, tags and drag-and-drop import.

use egui::RichText;

use crate::i18n::I18n;
use crate::state::{AppState, Page, format_timestamp};
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
    widgets::page_header(ui, i18n, "nav.snapshots", "page.snapshots.sub");

    let dropped = ui.ctx().input(|input| input.raw.dropped_files.clone());
    for file in dropped {
        let path = file.path();
        if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                extension.eq_ignore_ascii_case("dvsnap") || extension.eq_ignore_ascii_case("json")
            })
        {
            match state.import_snapshot_file(path) {
                Ok(entry) => {
                    state.ui.toast = Some(format!("Imported {}", entry.label));
                }
                Err(error) => {
                    state.ui.toast = Some(error);
                }
            }
        }
    }

    widgets::card(ui, |ui| {
        ui.label(RichText::new(i18n.t("snapshot.save")).strong());
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.snapshot_label)
                    .hint_text(i18n.t("snapshot.label")),
            );
            ui.add(
                egui::TextEdit::singleline(&mut state.ui.tag_buffer)
                    .hint_text(i18n.t("snapshot.tags_hint")),
            );
            if ui.button(i18n.t("common.save")).clicked() {
                let tags: Vec<String> = state
                    .ui
                    .tag_buffer
                    .split(',')
                    .map(|tag| tag.trim().to_string())
                    .filter(|tag| !tag.is_empty())
                    .collect();
                let label = state.ui.snapshot_label.clone();
                match state.save_current_snapshot(&label, tags) {
                    Ok(entry) => {
                        state.ui.toast = Some(format!("Saved {}", entry.file));
                        state.ui.snapshot_label.clear();
                        state.ui.tag_buffer.clear();
                    }
                    Err(error) => {
                        state.ui.toast = Some(error);
                    }
                }
            }
        });
        ui.label(
            RichText::new(format!(
                "{}: {}",
                i18n.t("snapshot.directory"),
                state.snapshots_dir.display()
            ))
            .weak()
            .size(10.0),
        );
    });

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        widgets::badge(
            ui,
            format!("{}: {}", i18n.t("common.snapshots"), state.snapshots.len()),
            widgets::BadgeKind::Info,
        );
        if ui.button(i18n.t("common.refresh")).clicked() {
            state.refresh_snapshots();
        }
    });
    ui.add_space(6.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if state.snapshots.is_empty() {
                widgets::empty(ui, i18n.t("snapshot.empty"));
                return;
            }
            let entries = state.snapshots.clone();
            for entry in &entries {
                ui.push_id(&entry.file, |ui| {
                    let response = widgets::card_response(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&entry.label).strong());
                            for tag in &entry.tags {
                                widgets::badge(ui, tag, widgets::BadgeKind::Info);
                            }
                            ui.label(
                                RichText::new(format_timestamp(entry.created_at_unix_ms))
                                    .weak()
                                    .size(10.0),
                            );
                            ui.label(
                                RichText::new(format!("{} KiB", entry.size_bytes.div_ceil(1024)))
                                    .weak()
                                    .size(10.0),
                            );
                        });
                        ui.horizontal(|ui| {
                            if ui.button(i18n.t("snapshot.compare")).clicked() {
                                if let Err(error) = state.load_compare(&entry.file) {
                                    state.ui.toast = Some(error);
                                } else {
                                    state.settings.page = Page::Compare;
                                }
                            }
                            if ui.button(i18n.t("snapshot.tags")).clicked() {
                                state.ui.tag_edit = Some(entry.file.clone());
                            }
                            if ui.button(i18n.t("common.delete")).clicked() {
                                if let Err(error) = state.delete_snapshot(&entry.file) {
                                    state.ui.toast = Some(error);
                                }
                            }
                            ui.label(RichText::new(&entry.file).monospace().weak().size(10.0))
                                .on_hover_text(
                                    entry.path(&state.snapshots_dir).display().to_string(),
                                );
                        });
                        if state.ui.tag_edit.as_deref() == Some(entry.file.as_str()) {
                            let mut tags = entry.tags.join(", ");
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut tags).hint_text("tag, tag"));
                                if ui.button(i18n.t("common.apply")).clicked() {
                                    let parsed: Vec<String> = tags
                                        .split(',')
                                        .map(|tag| tag.trim().to_string())
                                        .filter(|tag| !tag.is_empty())
                                        .collect();
                                    state.set_snapshot_tags(&entry.file, parsed);
                                    state.ui.tag_edit = None;
                                }
                            });
                        }
                    });
                    if response.response.double_clicked() {
                        if let Err(error) = state.load_compare(&entry.file) {
                            state.ui.toast = Some(error);
                        } else {
                            state.settings.page = Page::Compare;
                        }
                    }
                });
                ui.add_space(6.0);
            }
        });
}
