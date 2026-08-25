//! Small reusable egui widgets.

use eframe::egui::{self, CornerRadius, Margin, RichText};

use crate::theme;

/// Overview metric card.
pub fn metric_card(ui: &mut egui::Ui, title: &str, value: &str, sub: &str) {
    let fill = ui.visuals().faint_bg_color;
    egui::Frame::default()
        .fill(fill)
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            ui.set_min_width(188.0);
            ui.set_min_height(88.0);
            ui.label(
                RichText::new(title)
                    .size(12.0)
                    .color(theme::muted(ui))
                    .strong(),
            );
            ui.add_space(6.0);
            ui.label(RichText::new(value).size(24.0).strong());
            ui.label(RichText::new(sub).size(13.0).color(theme::muted(ui)));
        });
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
    egui::Frame::default()
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(title).strong());
                if !meta.is_empty() {
                    ui.label(RichText::new(meta).color(theme::muted(ui)));
                }
                if !detail.is_empty() {
                    ui.label(RichText::new(detail).color(theme::muted(ui)));
                }
            });
        });
    ui.add_space(4.0);
}

/// Screen title plus a one-line hint.
pub fn page_title(ui: &mut egui::Ui, title: &str, hint: &str) {
    ui.heading(RichText::new(title).size(26.0).strong());
    ui.add_space(2.0);
    ui.label(RichText::new(hint).size(14.5).color(theme::muted(ui)));
    ui.add_space(10.0);
}
