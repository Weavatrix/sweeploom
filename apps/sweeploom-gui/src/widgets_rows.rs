//! Clickable list rows whose title truncates around fixed metrics.

use eframe::egui::{self, CornerRadius, Margin, RichText, Sense};

use super::{hover_stroke, pointer};
use crate::theme;

/// Clickable listing row. Returns the row response.
pub fn list_row_at(ui: &mut egui::Ui, title: &str, meta: &str, detail: &str) -> egui::Response {
    let id = ui.id().with(title);
    let inner = egui::Frame::default()
        .fill(theme::card_fill(ui))
        .stroke(theme::card_stroke(ui))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(14, 10))
        .show(ui, |ui| {
            // Right to left: meta keeps its width, detail gets at most 45% of
            // the rest, and the title truncates into whatever remains.
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // A fixed row height first, so every item centres on the same line.
                ui.set_min_height(ui.spacing().interact_size.y);
                if !meta.is_empty() {
                    ui.add(egui::Label::new(RichText::new(meta).strong()).selectable(false));
                    ui.add_space(theme::SM);
                }
                if !detail.is_empty() {
                    let text = RichText::new(detail).size(12.5).color(theme::muted(ui));
                    let width = text_width(ui, &text).min(ui.available_width() * 0.45);
                    let height = ui.spacing().interact_size.y;
                    ui.allocate_ui_with_layout(
                        egui::vec2(width, height),
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            ui.add(egui::Label::new(text).selectable(false).truncate());
                        },
                    );
                    ui.add_space(theme::SM);
                }
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.add(
                        egui::Label::new(RichText::new(title).strong())
                            .selectable(false)
                            .truncate(),
                    )
                    .on_hover_text(title);
                });
            });
        });
    let response = pointer(inner.response.interact(Sense::click()));
    hover_stroke(ui, id, response.rect, response.hovered(), 8.0);
    ui.add_space(6.0);
    response
}

/// Unwrapped width of `text` in this ui's style.
fn text_width(ui: &egui::Ui, text: &RichText) -> f32 {
    egui::WidgetText::from(text.clone())
        .into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            f32::INFINITY,
            egui::TextStyle::Body,
        )
        .size()
        .x
}
