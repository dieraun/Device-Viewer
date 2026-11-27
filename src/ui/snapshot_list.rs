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
                    state.ui.notify(error);
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
                        state.ui.notify(error);
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

    widgets::page_body(ui, |ui| {
        if state.snapshots.is_empty() {
            widgets::empty(ui, i18n.t("snapshot.empty"));
            return;
        }
        let entries = state.snapshots.clone();
        let t = theme::theme(ui);
        // 行式列表：每行只放名称、标签、时间、大小，操作收进行尾的 ⋯ 菜单。
        widgets::card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{} · {}",
                        i18n.t("common.snapshots"),
                        entries.len()
                    ))
                    .size(theme::FONT_CAPTION)
                    .color(t.text_tertiary),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(i18n.t("snapshot.double_click"))
                            .size(theme::FONT_CAPTION)
                            .color(t.text_tertiary),
                    );
                });
            });
            ui.add_space(theme::SPACE_1);
            for entry in &entries {
                ui.push_id(&entry.file, |ui| {
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), theme::ROW_HEIGHT),
                        egui::Sense::click(),
                    );
                    let menu_rect = egui::Rect::from_center_size(
                        egui::pos2(rect.right() - 20.0, rect.center().y),
                        egui::Vec2::splat(24.0),
                    );
                    let menu_response =
                        ui.interact(menu_rect, ui.id().with("row-menu"), egui::Sense::click());
                    let menu_open = state.ui.snapshot_menu.as_deref() == Some(entry.file.as_str());
                    if response.hovered() || menu_open {
                        ui.painter().rect_filled(
                            rect,
                            egui::CornerRadius::same(theme::RADIUS_CONTROL),
                            t.accent_subtle,
                        );
                    }
                    let label_rect = ui.painter().text(
                        egui::pos2(rect.left() + theme::SPACE_2, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        &entry.label,
                        egui::FontId::proportional(theme::FONT_BODY),
                        t.text_primary,
                    );
                    if !entry.tags.is_empty() {
                        ui.painter().text(
                            egui::pos2(label_rect.right() + theme::SPACE_2, rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            entry.tags.join(" · "),
                            egui::FontId::proportional(theme::FONT_CAPTION),
                            t.text_tertiary,
                        );
                    }
                    let size_rect = ui.painter().text(
                        egui::pos2(menu_rect.left() - theme::SPACE_3, rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        format!("{} KiB", entry.size_bytes.div_ceil(1024)),
                        egui::FontId::proportional(theme::FONT_CAPTION),
                        t.text_tertiary,
                    );
                    ui.painter().text(
                        egui::pos2(size_rect.left() - theme::SPACE_3, rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        format_timestamp(entry.created_at_unix_ms),
                        egui::FontId::proportional(theme::FONT_CAPTION),
                        t.text_tertiary,
                    );
                    if menu_response.hovered() {
                        ui.painter().rect_filled(
                            menu_rect,
                            egui::CornerRadius::same(theme::RADIUS_CONTROL),
                            t.raised,
                        );
                    }
                    theme::paint_icon(
                        ui.painter(),
                        menu_rect.shrink(4.0),
                        theme::Icon::More,
                        if menu_response.hovered() || menu_open {
                            t.text_primary
                        } else {
                            t.text_tertiary
                        },
                    );
                    ui.painter().hline(
                        rect.x_range(),
                        rect.bottom(),
                        egui::Stroke::new(theme::BORDER_WIDTH, t.border_subtle),
                    );
                    let _ = response
                        .clone()
                        .on_hover_text(entry.path(&state.snapshots_dir).display().to_string());

                    if response.double_clicked() {
                        state.ui.snapshot_menu = None;
                        if let Err(error) = state.load_compare(&entry.file) {
                            state.ui.notify(error);
                        } else {
                            state.settings.page = Page::Compare;
                        }
                    } else if menu_response.clicked() {
                        state.ui.snapshot_menu = if menu_open {
                            None
                        } else {
                            Some(entry.file.clone())
                        };
                    } else if response.clicked() {
                        state.ui.snapshot_menu = None;
                        ui.ctx()
                            .copy_text(entry.path(&state.snapshots_dir).display().to_string());
                        state.ui.notify(i18n.t("toast.copied"));
                    }

                    if menu_open {
                        let mut action = 0u8;
                        egui::Area::new(egui::Id::new("snapshot-row-menu"))
                            .order(egui::Order::Foreground)
                            .fixed_pos(egui::pos2(
                                menu_rect.right() - 160.0,
                                rect.bottom() + theme::SPACE_1,
                            ))
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
                                if let Err(error) = state.load_compare(&entry.file) {
                                    state.ui.notify(error);
                                } else {
                                    state.settings.page = Page::Compare;
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

                    if state.ui.tag_edit.as_deref() == Some(entry.file.as_str()) {
                        ui.horizontal(|ui| {
                            ui.add_space(theme::SPACE_2);
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
                                state.set_snapshot_tags(&entry.file, parsed);
                                state.ui.tag_edit = None;
                                state.ui.tag_edit_buffer.clear();
                            }
                            if ui.button(i18n.t("common.cancel")).clicked() {
                                state.ui.tag_edit = None;
                                state.ui.tag_edit_buffer.clear();
                            }
                        });
                    }
                });
            }
        });
    });

    // 删除前二次确认；只有明确点击“删除”才会真正移除文件。
    if let Some(file) = state.ui.pending_delete.clone() {
        let modal =
            egui::Modal::new(egui::Id::new("snapshot-delete-confirm")).show(ui.ctx(), |ui| {
                ui.set_max_width(360.0);
                ui.label(RichText::new(i18n.t("snapshot.delete_confirm")).strong());
                ui.add_space(4.0);
                ui.label(RichText::new(&file).monospace().weak());
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button(i18n.t("common.delete")).clicked() {
                        if let Err(error) = state.delete_snapshot(&file) {
                            state.ui.notify(error);
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
