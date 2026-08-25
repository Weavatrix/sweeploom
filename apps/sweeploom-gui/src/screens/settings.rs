//! Local-only settings. Telemetry stays off.

use eframe::egui::{self, RichText};

use crate::app::SweepLoomApp;
use crate::autostart;
use crate::prefs::{SCALE_CHOICES, ThemeMode};
use crate::tray;
use crate::widgets::{page_title, section};

pub fn ui_settings(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(ui, "Settings", "Local-only. Telemetry stays off.");
    appearance(app, ui);
    background(app, ui);
    about(app, ui);
}

fn appearance(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    section(
        ui,
        "Appearance",
        "Theme and size apply immediately and are stored in prefs.json.",
        |ui| {
            ui.horizontal(|ui| {
                ui.label("Theme");
                for mode in [ThemeMode::Auto, ThemeMode::Dark, ThemeMode::Light] {
                    if ui
                        .selectable_label(app.prefs.theme == mode, mode.label())
                        .clicked()
                    {
                        app.prefs.theme = mode;
                        app.persist_prefs();
                    }
                }
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label("Interface size");
                for (label, scale) in SCALE_CHOICES {
                    let selected = (app.prefs.ui_scale - *scale).abs() < 0.01;
                    if ui.selectable_label(selected, *label).clicked() {
                        app.prefs.ui_scale = *scale;
                        app.persist_prefs();
                    }
                }
            });
        },
    );
}

fn background(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    section(
        ui,
        "Background",
        "Quiet tray watch: no UI, no network scan, process tables dropped until you open the window.",
        |ui| {
            if !tray::is_supported() {
                ui.label("Tray is available on Windows and macOS. Closing the window quits.");
                return;
            }
            let mut tray_on = app.prefs.tray_enabled;
            if ui
                .checkbox(
                    &mut tray_on,
                    "Keep running in the tray when the window closes",
                )
                .changed()
            {
                app.prefs.tray_enabled = tray_on;
                app.sync_tray();
                app.persist_prefs();
                if !tray_on {
                    app.leave_background(ui.ctx());
                }
            }
            if app.prefs.tray_enabled && app.tray.is_none() {
                ui.label("Tray icon could not be created. Closing the window will still quit.");
            }
            ui.add_space(6.0);
            if autostart::is_supported() {
                let mut auto = autostart::is_enabled();
                if ui
                    .checkbox(&mut auto, "Start SweepLoom in the tray when I sign in")
                    .changed()
                    && let Err(error) = autostart::set_enabled(auto)
                {
                    app.action_message = Some(error);
                }
            } else {
                ui.label("Sign-in autostart is available on Windows. Launch with --tray to start hidden.");
            }
        },
    );
}

fn about(app: &SweepLoomApp, ui: &mut egui::Ui) {
    section(ui, "About", "", |ui| {
        ui.label("Telemetry: none.");
        ui.label("License: MPL-2.0 (SweepLoom). Weavatrix crates remain MIT.");
        ui.label(format!("Home: {}", app.locations.home.display()));
        ui.label(format!("Config: {}", app.locations.app_config.display()));
        if let Some(snapshot) = &app.snapshot {
            ui.label(format!("Observed processes: {}", snapshot.processes.len()));
        }
        ui.label(RichText::new("Hidden launch: sweeploom-gui --tray").italics());
    });
}
