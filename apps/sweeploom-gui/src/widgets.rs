//! Small reusable egui widgets.

use eframe::egui::{self, RichText};

/// Overview metric card.
pub fn metric_card(ui: &mut egui::Ui, title: &str, value: &str, sub: &str) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_min_width(200.0);
        ui.label(RichText::new(title).small().weak());
        ui.label(RichText::new(value).heading());
        ui.label(RichText::new(sub).weak());
    });
}

/// Placeholder copy for screens that are not P0 yet.
pub fn placeholder(ui: &mut egui::Ui, title: &str, body: &str) {
    ui.heading(title);
    ui.add_space(8.0);
    ui.label(body);
}
