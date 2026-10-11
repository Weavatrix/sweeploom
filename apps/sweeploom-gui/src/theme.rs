//! Cool canvas, ink type, gold only as accent.

use eframe::egui::{
    self, Color32, CornerRadius, CursorIcon, FontFamily, FontId, Shadow, Stroke, TextStyle, Theme,
};

use crate::prefs::ThemeMode;

#[path = "theme_fonts.rs"]
mod fonts;
#[path = "theme_palette.rs"]
mod palette;
pub use palette::*;

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
    ctx.set_zoom_factor(scale.clamp(0.8, 1.6));
    fonts::install(ctx);
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
    style.spacing.item_spacing = egui::vec2(SM, SM);
    style.spacing.button_padding = egui::vec2(12.0, 5.0);
    style.spacing.indent = 18.0;
    style.spacing.interact_size.y = 28.0;
    style.spacing.icon_width = 16.0;
    style.spacing.icon_width_inner = 9.0;
    style.spacing.icon_spacing = 6.0;
    style.spacing.scroll.bar_width = 8.0;
    style.spacing.menu_margin = egui::Margin::same(6);
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
    visuals.window_fill = Color32::from_rgb(22, 26, 32);
    visuals.panel_fill = Color32::from_rgb(15, 17, 21);
    visuals.extreme_bg_color = Color32::from_rgb(11, 13, 17);
    visuals.faint_bg_color = Color32::from_rgb(22, 26, 32);
    let fills = [
        Color32::from_rgb(32, 37, 46),
        Color32::from_rgb(42, 48, 59),
        Color32::from_rgb(52, 59, 72),
    ];
    paint_widgets(&mut visuals, Color32::from_rgb(232, 234, 238), fills, true);
    visuals
}

fn light_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::light();
    visuals.window_fill = Color32::WHITE;
    visuals.panel_fill = Color32::from_rgb(244, 246, 248);
    visuals.extreme_bg_color = Color32::WHITE;
    visuals.faint_bg_color = Color32::from_rgb(249, 250, 252);
    let fills = [
        Color32::WHITE,
        Color32::from_rgb(241, 244, 247),
        Color32::from_rgb(229, 234, 240),
    ];
    paint_widgets(&mut visuals, Color32::from_rgb(20, 22, 28), fills, false);
    visuals
}

fn paint_widgets(visuals: &mut egui::Visuals, text: Color32, fills: [Color32; 3], dark: bool) {
    let rgb = |d: [u8; 3], l: [u8; 3]| {
        let [r, g, b] = if dark { d } else { l };
        Color32::from_rgb(r, g, b)
    };
    visuals.override_text_color = Some(text);
    visuals.weak_text_color = Some(rgb([120, 127, 140], [120, 126, 138]));
    visuals.selection.bg_fill = rgb([52, 42, 26], [255, 241, 212]);
    visuals.selection.stroke = Stroke::new(1.0_f32, GOLD);
    visuals.hyperlink_color = if dark { GOLD_SOFT } else { GOLD };
    visuals.disabled_alpha = 0.42;
    visuals.window_corner_radius = CornerRadius::same(RADIUS_LG);
    visuals.menu_corner_radius = CornerRadius::same(8);
    visuals.window_stroke = Stroke::new(1.0_f32, rgb([48, 54, 64], [214, 220, 228]));
    let border = rgb([70, 78, 92], [196, 204, 214]);
    let widgets = &mut visuals.widgets;
    widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, rgb([38, 44, 54], [226, 230, 236]));
    widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, text);
    widgets.noninteractive.corner_radius = CornerRadius::same(RADIUS);
    for (state, fill) in [
        &mut widgets.inactive,
        &mut widgets.hovered,
        &mut widgets.active,
    ]
    .into_iter()
    .zip(fills)
    {
        state.weak_bg_fill = fill;
        state.bg_fill = fill;
        state.corner_radius = CornerRadius::same(RADIUS);
        state.expansion = 0.0;
        state.fg_stroke = Stroke::new(1.5_f32, text);
    }
    widgets.inactive.bg_stroke = Stroke::new(1.0_f32, border);
    widgets.hovered.bg_stroke = Stroke::new(1.0_f32, rgb([104, 112, 128], [150, 160, 174]));
    widgets.active.bg_stroke = Stroke::new(1.0_f32, GOLD);
    widgets.open = widgets.active;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interface_scale_preserves_monitor_density() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            for scale in [1.0, 1.3] {
                let ctx = egui::Context::default();
                for native in [1.0, 2.0, 1.5, 1.0] {
                    let mut input = egui::RawInput::default();
                    input
                        .viewports
                        .get_mut(&egui::ViewportId::ROOT)
                        .unwrap()
                        .native_pixels_per_point = Some(native);
                    // Zoom changes take effect on the next pass.
                    for _ in 0..2 {
                        let _ = ctx.run(input.clone(), |ctx| apply(ctx, mode, scale));
                    }
                    assert!(
                        (ctx.pixels_per_point() - native * scale).abs() < 0.001,
                        "{mode:?}: native={native}, scale={scale}, actual={}",
                        ctx.pixels_per_point()
                    );
                }
            }
        }
    }
}
