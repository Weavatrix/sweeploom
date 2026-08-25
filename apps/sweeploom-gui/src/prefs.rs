//! Persisted appearance and tray preferences.

use std::fs;
use std::path::Path;

/// Color theme selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    /// Follow the OS light/dark setting.
    Auto,
    /// Dark palette.
    Dark,
    /// Light palette.
    Light,
}

impl ThemeMode {
    /// Short label for settings.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Dark => "Dark",
            Self::Light => "Light",
        }
    }
}

/// User-tunable GUI preferences.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Prefs {
    /// Theme.
    pub theme: ThemeMode,
    /// `egui` pixels-per-point. `1.0` is the default size.
    pub ui_scale: f32,
    /// Close hides to the tray instead of quitting.
    pub tray_enabled: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Auto,
            ui_scale: 1.0,
            tray_enabled: true,
        }
    }
}

impl Prefs {
    /// Clamp scale to a supported range.
    pub fn sanitize(&mut self) {
        if !self.ui_scale.is_finite() {
            self.ui_scale = 1.0;
        }
        self.ui_scale = self.ui_scale.clamp(0.8, 1.6);
        self.ui_scale = (self.ui_scale * 20.0).round() / 20.0;
    }

    /// Load JSON. Missing or invalid files become defaults.
    #[must_use]
    pub fn load(path: &Path) -> Self {
        let Ok(text) = fs::read_to_string(path) else {
            return Self::default();
        };
        let mut prefs: Prefs = serde_json::from_str(&text).unwrap_or_default();
        prefs.sanitize();
        prefs
    }

    /// Write JSON, creating parent directories.
    pub fn save(&self, path: &Path) {
        let mut prefs = self.clone();
        prefs.sanitize();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(text) = serde_json::to_string_pretty(&prefs) {
            let _ = fs::write(path, text);
        }
    }
}

/// Supported UI scale choices.
pub const SCALE_CHOICES: &[(&str, f32)] = &[
    ("Small", 0.85),
    ("Default", 1.0),
    ("Large", 1.15),
    ("Larger", 1.3),
    ("Huge", 1.5),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_json() {
        let root = std::env::temp_dir().join(format!("sweeploom-prefs-{}", std::process::id()));
        let _ = fs::create_dir_all(&root);
        let path = root.join("prefs.json");
        let prefs = Prefs {
            theme: ThemeMode::Light,
            ui_scale: 1.15,
            tray_enabled: false,
        };
        prefs.save(&path);
        let loaded = Prefs::load(&path);
        let _ = fs::remove_dir_all(&root);
        assert_eq!(loaded.theme, ThemeMode::Light);
        assert!((loaded.ui_scale - 1.15).abs() < f32::EPSILON);
        assert!(!loaded.tray_enabled);
    }
}
