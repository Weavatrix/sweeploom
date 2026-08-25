//! Small reusable egui widgets.

use eframe::egui::{self, Color32, RichText};

/// Overview metric card.
pub fn metric_card(ui: &mut egui::Ui, title: &str, value: &str, sub: &str) {
    egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.set_min_width(210.0);
            ui.set_min_height(92.0);
            ui.label(
                RichText::new(title)
                    .size(13.0)
                    .color(Color32::from_rgb(168, 174, 186)),
            );
            ui.add_space(4.0);
            ui.label(RichText::new(value).size(26.0).strong());
            ui.label(
                RichText::new(sub)
                    .size(14.0)
                    .color(Color32::from_rgb(168, 174, 186)),
            );
        });
}

/// Screen title plus a one-line hint.
pub fn page_title(ui: &mut egui::Ui, title: &str, hint: &str) {
    ui.heading(RichText::new(title).size(28.0).strong());
    ui.add_space(4.0);
    ui.label(
        RichText::new(hint)
            .size(15.0)
            .color(Color32::from_rgb(168, 174, 186)),
    );
    ui.add_space(12.0);
}
