//! Light, dark, and auto palettes. Accent stays SweepLoom gold.

use eframe::egui::{
    self, Color32, CornerRadius, CursorIcon, FontFamily, FontId, Stroke, TextStyle, Theme,
};

use crate::prefs::ThemeMode;

const GOLD: Color32 = Color32::from_rgb(196, 140, 64);

/// SweepLoom gold, used for selected chrome and accents.
#[must_use]
pub const fn accent() -> Color32 {
    GOLD
}

/// Apply fonts, spacing, and the resolved palette.
pub fn apply(ctx: &egui::Context, mode: ThemeMode, scale: f32) {
    ctx.set_pixels_per_point(scale.clamp(0.8, 1.6));
    let dark = match mode {
        ThemeMode::Dark => true,
        ThemeMode::Light => false,
        ThemeMode::Auto => ctx.system_theme() != Some(Theme::Light),
    };
    let mut style = (*ctx.style()).clone();
    style.visuals = if dark {
        dark_visuals()
    } else {
        light_visuals()
    };
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 7.0);
    style.spacing.indent = 16.0;
    style.spacing.interact_size.y = 28.0;
    style.spacing.scroll.bar_width = 10.0;
    style.interaction.selectable_labels = false;
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(26.0, FontFamily::Proportional),
    );
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(16.0, FontFamily::Proportional));
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(15.5, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(13.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(14.5, FontFamily::Monospace),
    );
    ctx.set_style(style);
}

/// Secondary label color that follows the active theme.
#[must_use]
pub fn muted(ui: &egui::Ui) -> Color32 {
    ui.visuals().weak_text_color()
}

fn dark_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = Color32::from_rgb(18, 20, 24);
    visuals.panel_fill = Color32::from_rgb(22, 24, 30);
    visuals.extreme_bg_color = Color32::from_rgb(14, 15, 18);
    visuals.faint_bg_color = Color32::from_rgb(32, 35, 42);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(40, 44, 54);
    visuals.widgets.active.bg_fill = Color32::from_rgb(50, 54, 66);
    paint_widgets(&mut visuals, Color32::from_rgb(226, 228, 234), true);
    visuals
}

fn light_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::light();
    visuals.window_fill = Color32::from_rgb(244, 245, 248);
    visuals.panel_fill = Color32::from_rgb(252, 252, 254);
    visuals.extreme_bg_color = Color32::from_rgb(232, 234, 238);
    visuals.faint_bg_color = Color32::from_rgb(236, 238, 242);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(232, 226, 214);
    visuals.widgets.active.bg_fill = Color32::from_rgb(224, 214, 196);
    paint_widgets(&mut visuals, Color32::from_rgb(32, 36, 42), false);
    visuals
}

fn paint_widgets(visuals: &mut egui::Visuals, text: Color32, dark: bool) {
    visuals.override_text_color = Some(text);
    visuals.selection.bg_fill = if dark {
        Color32::from_rgb(58, 48, 36)
    } else {
        Color32::from_rgb(245, 232, 210)
    };
    visuals.selection.stroke = Stroke::new(1.0_f32, GOLD);
    visuals.hyperlink_color = GOLD;
    visuals.widgets.inactive.corner_radius = CornerRadius::same(8);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(8);
    visuals.widgets.active.corner_radius = CornerRadius::same(8);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, text);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, text);
    visuals.widgets.hovered.expansion = 1.0;
    visuals.widgets.active.expansion = 1.0;
    visuals.interact_cursor = Some(CursorIcon::PointingHand);
}

/// Mix two colors. Used for hover/selection motion.
#[must_use]
pub fn lerp(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let mix = |left: u8, right: u8| ((left as f32) * (1.0 - t) + (right as f32) * t) as u8;
    Color32::from_rgba_unmultiplied(
        mix(a.r(), b.r()),
        mix(a.g(), b.g()),
        mix(a.b(), b.b()),
        mix(a.a(), b.a()),
    )
}
