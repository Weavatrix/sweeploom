//! Small reusable egui widgets.

use eframe::egui::{self, CornerRadius, CursorIcon, Margin, RichText, Sense};

use crate::icons::{self, Glyph};
use crate::theme;

/// One overview metric.
pub struct Metric {
    /// Stroke icon.
    pub icon: Glyph,
    /// Uppercase caption.
    pub title: String,
    /// Primary value.
    pub value: String,
    /// Supporting line.
    pub sub: String,
}

/// Pointing hand on anything the user can click.
pub fn pointer(response: egui::Response) -> egui::Response {
    response.on_hover_cursor(CursorIcon::PointingHand)
}

/// Overview metric cards that wrap instead of clipping.
pub fn metric_grid(ui: &mut egui::Ui, cards: &[Metric]) {
    if cards.is_empty() {
        return;
    }
    let gap = ui.spacing().item_spacing.x;
    let avail = ui.available_width();
    let min_w = 220.0;
    let cols = ((avail + gap) / (min_w + gap)).floor().clamp(1.0, 4.0) as usize;
    let width = ((avail - gap * cols.saturating_sub(1) as f32) / cols as f32).max(160.0);
    for chunk in cards.chunks(cols) {
        ui.horizontal(|ui| {
            for card in chunk {
                metric_card(ui, card, width);
            }
        });
        ui.add_space(8.0);
    }
}

fn metric_card(ui: &mut egui::Ui, card: &Metric, width: f32) {
    let id = ui.id().with(&card.title);
    let inner_w = (width - 24.0).max(120.0);
    ui.scope(|ui| {
        ui.set_width(width);
        ui.set_max_width(width);
        let inner = egui::Frame::default()
            .fill(ui.visuals().faint_bg_color)
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                ui.set_width(inner_w);
                ui.set_max_width(inner_w);
                ui.set_min_height(88.0);
                ui.horizontal(|ui| {
                    icons::show(ui, card.icon, 18.0, theme::accent());
                    ui.add(
                        egui::Label::new(
                            RichText::new(&card.title)
                                .size(12.0)
                                .color(theme::muted(ui))
                                .strong(),
                        )
                        .selectable(false),
                    );
                });
                ui.add_space(6.0);
                ui.add(
                    egui::Label::new(RichText::new(&card.value).size(22.0).strong())
                        .selectable(false)
                        .wrap(),
                );
                ui.add(
                    egui::Label::new(RichText::new(&card.sub).size(13.0).color(theme::muted(ui)))
                        .selectable(false)
                        .wrap(),
                );
            });
        gold_stroke(ui, id, inner.response.rect, inner.response.hovered(), 10.0);
    });
}

fn gold_stroke(ui: &egui::Ui, id: egui::Id, rect: egui::Rect, hovered: bool, radius: f32) {
    let t = ui.ctx().animate_bool_with_time(id, hovered, 0.12);
    if t > 0.02 {
        ui.painter().rect_stroke(
            rect,
            radius,
            egui::Stroke::new(
                1.2_f32,
                theme::lerp(egui::Color32::TRANSPARENT, theme::accent(), t),
            ),
            egui::StrokeKind::Inside,
        );
    }
}

/// Grouped settings/content card.
pub fn section(
    ui: &mut egui::Ui,
    title: &str,
    hint: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    egui::Frame::default()
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            ui.label(RichText::new(title).size(16.0).strong());
            if !hint.is_empty() {
                ui.label(RichText::new(hint).size(13.5).color(theme::muted(ui)));
            }
            ui.add_space(8.0);
            add_contents(ui);
        });
    ui.add_space(12.0);
}

/// One opportunity / history / listing row.
pub fn list_row(ui: &mut egui::Ui, title: &str, meta: &str, detail: &str) {
    let _ = list_row_at(ui, title, meta, detail);
}

/// Clickable listing row. Returns the row response.
pub fn list_row_at(ui: &mut egui::Ui, title: &str, meta: &str, detail: &str) -> egui::Response {
    let id = ui.id().with(title);
    let inner = egui::Frame::default()
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.add(egui::Label::new(RichText::new(title).strong()).selectable(false));
                if !meta.is_empty() {
                    ui.add(
                        egui::Label::new(RichText::new(meta).color(theme::muted(ui)))
                            .selectable(false),
                    );
                }
                if !detail.is_empty() {
                    ui.add(
                        egui::Label::new(RichText::new(detail).color(theme::muted(ui)))
                            .selectable(false),
                    );
                }
            });
        });
    let response = pointer(inner.response.interact(Sense::click()));
    gold_stroke(ui, id, response.rect, response.hovered(), 8.0);
    ui.add_space(4.0);
    response
}

/// Height that keeps a table scrolling inside the remaining window.
#[must_use]
pub fn table_scroll_height(ui: &egui::Ui) -> f32 {
    ui.available_height().max(180.0)
}

/// Screen title plus a one-line hint.
pub fn page_title(ui: &mut egui::Ui, title: &str, hint: &str) {
    ui.heading(RichText::new(title).size(26.0).strong());
    ui.add_space(2.0);
    ui.label(RichText::new(hint).size(14.5).color(theme::muted(ui)));
    ui.add_space(10.0);
}
