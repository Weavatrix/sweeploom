//! Cool canvas, ink type, gold only as accent.

use eframe::egui::{
    self, Color32, CornerRadius, CursorIcon, FontFamily, FontId, Shadow, Stroke, TextStyle, Theme,
};

use crate::prefs::ThemeMode;

const GOLD: Color32 = Color32::from_rgb(196, 140, 48);
const GOLD_SOFT: Color32 = Color32::from_rgb(214, 168, 84);
const INK: Color32 = Color32::from_rgb(17, 18, 22);

/// SweepLoom gold. Use for marks and a single selected rail — not fills.
#[must_use]
pub const fn accent() -> Color32 {
    GOLD
}

/// Softer gold for chips on dark chrome.
#[must_use]
pub const fn accent_soft() -> Color32 {
    GOLD_SOFT
}

/// Ink used on gold fills and dark chrome.
#[must_use]
pub const fn ink() -> Color32 {
    INK
}

/// Warning / blocked / critical free space.
#[must_use]
pub const fn warn() -> Color32 {
    Color32::from_rgb(196, 72, 48)
}

/// Safe / quiet chip.
#[must_use]
pub const fn ok() -> Color32 {
    Color32::from_rgb(46, 128, 88)
}

/// Dark sidebar / header fill.
#[must_use]
pub const fn chrome() -> Color32 {
    Color32::from_rgb(18, 20, 24)
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
    style.spacing.item_spacing = egui::vec2(12.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.indent = 18.0;
    style.spacing.interact_size.y = 26.0;
    style.spacing.scroll.bar_width = 8.0;
    style.interaction.selectable_labels = false;
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(22.0, FontFamily::Proportional),
    );
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(15.0, FontFamily::Proportional));
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(14.5, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(12.5, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(13.5, FontFamily::Monospace),
    );
    ctx.set_style(style);
}

/// Secondary label color that follows the active theme.
#[must_use]
pub fn muted(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(148, 154, 164)
    } else {
        Color32::from_rgb(92, 98, 110)
    }
}

/// Card surface: white in light, lifted charcoal in dark.
#[must_use]
pub fn card_fill(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(24, 28, 34)
    } else {
        Color32::WHITE
    }
}

/// Hairline around cards.
#[must_use]
pub fn card_stroke(ui: &egui::Ui) -> Stroke {
    Stroke::new(
        1.0_f32,
        if ui.visuals().dark_mode {
            Color32::from_rgb(42, 48, 58)
        } else {
            Color32::from_rgb(226, 230, 236)
        },
    )
}

/// Soft card lift. egui has no real material, so keep this faint.
#[must_use]
pub fn card_shadow(ui: &egui::Ui) -> Shadow {
    if ui.visuals().dark_mode {
        Shadow::NONE
    } else {
        Shadow {
            offset: [0, 1],
            blur: 10,
            spread: 0,
            color: Color32::from_black_alpha(18),
        }
    }
}

fn dark_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = Color32::from_rgb(12, 14, 18);
    visuals.panel_fill = Color32::from_rgb(16, 18, 22);
    visuals.extreme_bg_color = Color32::from_rgb(10, 12, 16);
    visuals.faint_bg_color = Color32::from_rgb(24, 28, 34);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(28, 32, 40);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(38, 44, 54);
    visuals.widgets.active.bg_fill = Color32::from_rgb(48, 54, 66);
    paint_widgets(&mut visuals, Color32::from_rgb(232, 234, 238), true);
    visuals
}

fn light_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::light();
    visuals.window_fill = Color32::from_rgb(244, 246, 248);
    visuals.panel_fill = Color32::from_rgb(244, 246, 248);
    visuals.extreme_bg_color = Color32::from_rgb(232, 236, 240);
    visuals.faint_bg_color = Color32::WHITE;
    visuals.widgets.inactive.bg_fill = Color32::WHITE;
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(236, 240, 244);
    visuals.widgets.active.bg_fill = Color32::from_rgb(226, 232, 238);
    paint_widgets(&mut visuals, Color32::from_rgb(20, 22, 28), false);
    visuals
}

fn paint_widgets(visuals: &mut egui::Visuals, text: Color32, dark: bool) {
    visuals.override_text_color = Some(text);
    visuals.selection.bg_fill = if dark {
        Color32::from_rgb(48, 40, 28)
    } else {
        Color32::from_rgb(255, 244, 220)
    };
    visuals.selection.stroke = Stroke::new(1.0_f32, GOLD);
    visuals.hyperlink_color = GOLD;
    visuals.widgets.inactive.corner_radius = CornerRadius::same(8);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(8);
    visuals.widgets.active.corner_radius = CornerRadius::same(8);
    visuals.window_corner_radius = CornerRadius::same(10);
    visuals.widgets.inactive.bg_stroke = Stroke::new(
        1.0_f32,
        if dark {
            Color32::from_rgb(48, 54, 64)
        } else {
            Color32::from_rgb(214, 220, 228)
        },
    );
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, text);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, text);
    visuals.widgets.hovered.expansion = 0.0;
    visuals.widgets.active.expansion = 0.0;
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
