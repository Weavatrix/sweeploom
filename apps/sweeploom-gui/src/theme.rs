//! Dark theme with readable type. SweepLoom is not a dense dashboard.

use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle};

/// Apply fonts, spacing, and a quieter dark palette.
pub fn apply(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.dark_mode = true;
    visuals.window_fill = Color32::from_rgb(16, 18, 22);
    visuals.panel_fill = Color32::from_rgb(20, 22, 28);
    visuals.extreme_bg_color = Color32::from_rgb(12, 13, 16);
    visuals.faint_bg_color = Color32::from_rgb(28, 31, 38);
    visuals.override_text_color = Some(Color32::from_rgb(230, 232, 236));
    visuals.selection.bg_fill = Color32::from_rgb(196, 140, 64);
    visuals.selection.stroke = Stroke::new(1.0_f32, Color32::from_rgb(240, 196, 120));
    visuals.widgets.noninteractive.fg_stroke =
        Stroke::new(1.0_f32, Color32::from_rgb(210, 214, 220));
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(220, 224, 230));
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(42, 46, 56);
    visuals.widgets.active.bg_fill = Color32::from_rgb(52, 56, 68);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(6);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(6);
    visuals.widgets.active.corner_radius = CornerRadius::same(6);
    let mut style = (*ctx.style()).clone();
    style.visuals = visuals;
    style.spacing.item_spacing = egui::vec2(12.0, 10.0);
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    style.spacing.indent = 18.0;
    style.spacing.interact_size.y = 30.0;
    style.spacing.scroll.bar_width = 12.0;
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(28.0, FontFamily::Proportional),
    );
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(17.0, FontFamily::Proportional));
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(16.5, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(14.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(15.0, FontFamily::Monospace),
    );
    ctx.set_style(style);
}
