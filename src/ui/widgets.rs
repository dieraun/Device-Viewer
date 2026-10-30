//! Small, reusable presentation helpers shared by every panel.

use egui::{Align, Color32, CornerRadius, FontId, Layout, RichText, Stroke, Vec2};

use crate::i18n::I18n;

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
    egui::Frame::group(ui.style())
        .inner_margin(12)
        .corner_radius(8)
        .show(ui, add)
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

pub fn badge(ui: &mut egui::Ui, text: impl Into<String>, kind: BadgeKind) {
    let (foreground, background) = kind.colors();
    egui::Frame::new()
        .fill(background)
        .corner_radius(CornerRadius::same(4))
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(text).color(foreground).size(12.0).strong());
        });
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
                            .color(Color32::from_rgb(230, 180, 70))
                            .strong(),
                    );
                    ui.label(warning);
                });
            }
        },
    );
}

pub fn metric(ui: &mut egui::Ui, label: &str, value: impl Into<String>, detail: Option<String>) {
    card(ui, |ui| {
        ui.set_min_width(150.0);
        ui.label(RichText::new(label).weak().size(12.0));
        ui.label(RichText::new(value.into()).size(22.0).strong());
        if let Some(detail) = detail {
            ui.label(RichText::new(detail).weak().size(11.0));
        }
    });
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
    ui.add(egui::ProgressBar::new((percent / 100.0).clamp(0.0, 1.0)).desired_height(8.0));
}
