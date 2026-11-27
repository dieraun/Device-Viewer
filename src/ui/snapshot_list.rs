//! Saved snapshots, tags and drag-and-drop import.

use egui::RichText;

use crate::i18n::I18n;
use crate::state::{AppState, Page, format_timestamp};
use crate::ui::theme;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, state: &mut AppState, i18n: I18n) {
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
                    state
                        .ui
                        .notify_page(format!("Imported {}", entry.label), Page::Snapshots);
                }
                Err(error) => {
                    state.ui.notify_error(error);
                }
            }
        } else if let Some(name) = path.file_name() {
            state.ui.notify_warning(format!(
                "{}: {}",
                name.to_string_lossy(),
                i18n.t("snapshot.unsupported")
            ));
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
            if ui
                .add(widgets::primary_button(i18n.t("common.save")).fill(theme::theme(ui).accent))
                .clicked()
            {
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
                        state
                            .ui
                            .notify_page(format!("Saved {}", entry.file), Page::Snapshots);
                        state.ui.snapshot_label.clear();
                        state.ui.tag_buffer.clear();
                    }
                    Err(error) => {
                        state.ui.notify_error(error);
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
            .size(theme::FONT_CAPTION),
        );
    });

    ui.add_space(theme::SECTION_GAP);
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
    ui.add_space(theme::SPACE_3);

    widgets::page_body(ui, |ui| {
        if state.snapshots.is_empty() {
            widgets::empty_state(
                ui,
                theme::theme(ui),
                theme::Icon::Snapshot,
                i18n.t("snapshot.empty"),
                i18n.t("snapshot.drop_hint"),
                None,
            );
        } else {
            let entries = state.snapshots.clone();
            // 列表复用通用表格：整行 hover、单击复制单元格、右键复制整行（复制内容为完整路径）。
            let rows: Vec<widgets::Row> = entries
                .iter()
                .map(|entry| {
                    let mut row = widgets::Row::new(vec![
                        widgets::Cell::Strong(entry.label.clone()),
                        widgets::Cell::Muted(entry.tags.join(" · ")),
                        widgets::Cell::Muted(format_timestamp(entry.created_at_unix_ms)),
                        widgets::Cell::Muted(format!("{} KiB", entry.size_bytes.div_ceil(1024))),
                    ]);
                    row.copy = entry.path(&state.snapshots_dir).display().to_string();
                    row.with_action(theme::Icon::More, i18n.t("shell.more"))
                })
                .collect();
            let table = widgets::data_table(
                ui,
                i18n,
                "snapshot-table",
                &[
                    i18n.t("snapshot.label"),
                    i18n.t("snapshot.tags"),
                    i18n.t("snapshot.created"),
                    i18n.t("snapshot.size"),
                ],
                &[2.4, 1.8, 1.6, 0.9],
                &rows,
                false,
            );

            if let Some(index) = table.double_clicked_row {
                if let Some(entry) = entries.get(index) {
                    state.ui.snapshot_menu = None;
                    match state.load_compare(&entry.file) {
                        Ok(()) => state.settings.page = Page::Compare,
                        Err(error) => state.ui.notify_error(error),
                    }
                }
            } else if let Some(index) = table.action_row {
                if let Some(entry) = entries.get(index) {
                    let same = state.ui.snapshot_menu.as_deref() == Some(entry.file.as_str());
                    state.ui.snapshot_menu = if same { None } else { Some(entry.file.clone()) };
                    state.ui.snapshot_menu_pos = table.action_pos;
                }
            } else if table.clicked_row.is_some() {
                state.ui.snapshot_menu = None;
                state.ui.notify(i18n.t("toast.copied"));
            }

            if let (Some(file), Some(pos)) =
                (state.ui.snapshot_menu.clone(), state.ui.snapshot_menu_pos)
                && let Some(entry) = entries.iter().find(|entry| entry.file == file)
            {
                let t = theme::theme(ui);
                let mut action = 0u8;
                egui::Area::new(egui::Id::unique("snapshot-row-menu"))
                    .order(egui::Order::Foreground)
                    .fixed_pos(egui::pos2(pos.x - 160.0, pos.y + theme::SPACE_1))
                    .show(ui.ctx(), |ui| {
                        t.overlay_frame().show(ui, |ui| {
                            ui.set_min_width(160.0);
                            if ui.button(i18n.t("snapshot.compare")).clicked() {
                                action = 1;
                            }
                            if ui.button(i18n.t("snapshot.tags")).clicked() {
                                action = 2;
                            }
                            if ui.button(i18n.t("common.delete")).clicked() {
                                action = 3;
                            }
                        });
                    });
                match action {
                    1 => {
                        state.ui.snapshot_menu = None;
                        match state.load_compare(&entry.file) {
                            Ok(()) => state.settings.page = Page::Compare,
                            Err(error) => state.ui.notify_error(error),
                        }
                    }
                    2 => {
                        state.ui.snapshot_menu = None;
                        // 缓冲只在打开另一个快照时重建，输入不会被每帧重置。
                        state.ui.begin_tag_edit(&entry.file, &entry.tags);
                    }
                    3 => {
                        state.ui.snapshot_menu = None;
                        state.ui.pending_delete = Some(entry.file.clone());
                    }
                    _ => {}
                }
            }

            if let Some(file) = state.ui.tag_edit.clone() {
                ui.add_space(theme::SPACE_2);
                widgets::card(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(i18n.t("snapshot.tags"))
                                .size(theme::FONT_CAPTION)
                                .color(theme::theme(ui).text_tertiary),
                        );
                        ui.add(
                            egui::TextEdit::singleline(&mut state.ui.tag_edit_buffer)
                                .hint_text(i18n.t("snapshot.tags_hint")),
                        );
                        if ui.button(i18n.t("common.apply")).clicked() {
                            let parsed: Vec<String> = state
                                .ui
                                .tag_edit_buffer
                                .split(',')
                                .map(|tag| tag.trim().to_string())
                                .filter(|tag| !tag.is_empty())
                                .collect();
                            state.set_snapshot_tags(&file, parsed);
                            state.ui.tag_edit = None;
                            state.ui.tag_edit_buffer.clear();
                        }
                        if ui.button(i18n.t("common.cancel")).clicked() {
                            state.ui.tag_edit = None;
                            state.ui.tag_edit_buffer.clear();
                        }
                    });
                });
            }
        }
    });

    // 删除前二次确认；只有明确点击“删除”才会真正移除文件。
    if let Some(file) = state.ui.pending_delete.clone() {
        let modal =
            egui::Modal::new(egui::Id::unique("snapshot-delete-confirm")).show(ui.ctx(), |ui| {
                ui.set_max_width(360.0);
                ui.label(RichText::new(i18n.t("snapshot.delete_confirm")).strong());
                ui.add_space(theme::SPACE_1);
                ui.label(RichText::new(&file).monospace().weak());
                ui.add_space(theme::SPACE_3);
                ui.horizontal(|ui| {
                    if ui.button(i18n.t("common.delete")).clicked() {
                        match state.delete_snapshot(&file) {
                            Ok(()) => state
                                .ui
                                .notify_success(i18n.t("snapshot.deleted").to_string()),
                            Err(error) => state.ui.notify_error(error),
                        }
                        state.ui.pending_delete = None;
                    }
                    if ui.button(i18n.t("common.cancel")).clicked() {
                        state.ui.pending_delete = None;
                    }
                });
            });
        if modal.should_close() {
            state.ui.pending_delete = None;
        }
    }
}
