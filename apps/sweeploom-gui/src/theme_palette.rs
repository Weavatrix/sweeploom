//! Design tokens: spacing, radii, semantic tones and chart colors for both themes.

use eframe::egui::{self, Color32};

/// 4 px — tight pairs (icon + label).
pub const XS: f32 = 4.0;
/// 8 px — default gap between related controls.
pub const SM: f32 = 8.0;
/// 12 px — gap between cards in a grid.
pub const MD: f32 = 12.0;
/// 16 px — card padding, gap after a toolbar.
pub const LG: f32 = 16.0;
/// 24 px — page side padding, gap between sections.
pub const XL: f32 = 24.0;

/// Small radius: checkboxes, bars, pills.
pub const RADIUS_SM: u8 = 4;
/// Control radius: buttons, inputs, segments.
pub const RADIUS: u8 = 6;
/// Card radius.
pub const RADIUS_LG: u8 = 10;

/// Semantic color role for values, pills and bars.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    /// Default ink.
    Neutral,
    /// Critical / blocked / high load.
    Warn,
    /// Quiet / healthy / active.
    Ok,
    /// Reclaimable or worth a look.
    Caution,
    /// Informational.
    Info,
    /// De-emphasised state.
    Muted,
}

fn dark(ui: &egui::Ui) -> bool {
    ui.visuals().dark_mode
}

fn pick(ui: &egui::Ui, dark_rgb: [u8; 3], light_rgb: [u8; 3]) -> Color32 {
    let [r, g, b] = if dark(ui) { dark_rgb } else { light_rgb };
    Color32::from_rgb(r, g, b)
}

/// Track behind meters and bars; inset wells.
#[must_use]
pub fn inset(ui: &egui::Ui) -> Color32 {
    pick(ui, [34, 39, 48], [232, 236, 241])
}

/// Stronger border for checkboxes and inputs.
#[must_use]
pub fn border_strong(ui: &egui::Ui) -> Color32 {
    pick(ui, [84, 92, 106], [168, 176, 188])
}

/// Text / mark color for a tone. Meets 4.5:1 on cards in both themes.
#[must_use]
pub fn tone(ui: &egui::Ui, tone: Tone) -> Color32 {
    match tone {
        Tone::Neutral => ui.visuals().text_color(),
        Tone::Muted => super::muted(ui),
        Tone::Ok => pick(ui, [84, 200, 128], [22, 124, 62]),
        Tone::Warn => pick(ui, [240, 124, 92], [190, 62, 40]),
        Tone::Caution => pick(ui, [230, 180, 76], [150, 100, 14]),
        Tone::Info => pick(ui, [104, 164, 240], [36, 100, 190]),
    }
}

/// Subtle pill background for a tone.
#[must_use]
pub fn tone_bg(ui: &egui::Ui, tone: Tone) -> Color32 {
    let base = match tone {
        Tone::Neutral | Tone::Muted => return inset(ui),
        _ => self::tone(ui, tone),
    };
    let alpha = if dark(ui) { 0.16 } else { 0.12 };
    super::lerp(super::card_fill(ui), base, alpha)
}

/// Categorical series color, fixed order. Folds past eight.
#[must_use]
pub fn series(ui: &egui::Ui, index: usize) -> Color32 {
    const LIGHT: [[u8; 3]; 8] = [
        [42, 120, 214],
        [235, 104, 52],
        [27, 175, 122],
        [237, 161, 0],
        [232, 123, 164],
        [0, 131, 0],
        [74, 58, 167],
        [227, 73, 72],
    ];
    const DARK: [[u8; 3]; 8] = [
        [57, 135, 229],
        [217, 89, 38],
        [25, 158, 112],
        [201, 133, 0],
        [213, 81, 129],
        [0, 131, 0],
        [144, 133, 233],
        [230, 103, 103],
    ];
    let [r, g, b] = if dark(ui) { DARK } else { LIGHT }[index % 8];
    Color32::from_rgb(r, g, b)
}

/// Neutral gray for "Other" / "Free" segments.
#[must_use]
pub fn series_rest(ui: &egui::Ui) -> Color32 {
    pick(ui, [86, 94, 108], [176, 183, 194])
}

/// Sequential blue ramp for magnitude (0 = near surface, 1 = densest).
#[must_use]
pub fn seq(ui: &egui::Ui, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let (low, high) = if dark(ui) {
        (
            Color32::from_rgb(30, 44, 66),
            Color32::from_rgb(110, 172, 245),
        )
    } else {
        (
            Color32::from_rgb(214, 228, 248),
            Color32::from_rgb(24, 79, 149),
        )
    };
    super::lerp(low, high, t.powf(0.8))
}

/// Load color: sequential up to 85%, then the warn tone.
#[must_use]
pub fn load(ui: &egui::Ui, percent: f32) -> Color32 {
    if percent >= 85.0 {
        tone(ui, Tone::Warn)
    } else {
        seq(ui, 0.2 + percent / 100.0)
    }
}

/// Map a session / process status label to a tone.
#[must_use]
pub fn status_tone(label: &str) -> Tone {
    match label {
        "High CPU" => Tone::Warn,
        "Possibly stale" | "Forgotten" | "Orphan helper" | "Idle, heavy RAM" => Tone::Caution,
        "Active now" | "Network active" => Tone::Ok,
        _ => Tone::Muted,
    }
}
