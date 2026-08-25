//! Local-only settings. Telemetry stays off.

use eframe::egui;

use crate::app::SweepLoomApp;
use crate::widgets::page_title;

pub fn ui_settings(app: &SweepLoomApp, ui: &mut egui::Ui) {
    page_title(ui, "Settings", "Local-only. Telemetry stays off.");
    ui.label("Telemetry: none.");
    ui.label("License: MPL-2.0 (SweepLoom). Weavatrix crates remain MIT.");
    ui.label(format!("Home: {}", app.locations.home.display()));
    ui.label(format!("Temp: {}", app.locations.temp.display()));
    if let Some(snapshot) = &app.snapshot {
        ui.label(format!("Observed processes: {}", snapshot.processes.len()));
    }
}
