//! Local-only settings. Telemetry stays off.

use eframe::egui::{self, RichText};

use crate::app::SweepLoomApp;
use crate::autostart;
use crate::prefs::{SCALE_CHOICES, ThemeMode};
use crate::tray;
use crate::widgets::{self, page_title, section};

pub fn ui_settings(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(ui, "Settings", "Local-only. Telemetry stays off.");
    ui.scope(|ui| {
        ui.set_max_width(760.0);
        appearance(app, ui);
        background(app, ui);
        about(app, ui);
    });
}

fn row(ui: &mut egui::Ui, label: &str, add: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(132.0, 28.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.set_width(132.0);
                ui.label(RichText::new(label).color(crate::theme::muted(ui)));
            },
        );
        add(ui);
    });
}

fn appearance(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    section(
        ui,
        "Appearance",
        "Theme and size apply immediately and are stored in prefs.json.",
        |ui| {
            row(ui, "Theme", |ui| {
                let mut mode = app.prefs.theme;
                if widgets::segmented(
                    ui,
                    &mut mode,
                    &[
                        (ThemeMode::Auto, "Auto"),
                        (ThemeMode::Dark, "Dark"),
                        (ThemeMode::Light, "Light"),
                    ],
                ) {
                    app.prefs.theme = mode;
                    app.persist_prefs();
                }
            });
            ui.add_space(crate::theme::SM);
            row(ui, "Interface size", |ui| {
                let mut scale = SCALE_CHOICES
                    .iter()
                    .position(|(_, scale)| (app.prefs.ui_scale - *scale).abs() < 0.01)
                    .unwrap_or(usize::MAX);
                let options: Vec<(usize, &str)> = SCALE_CHOICES
                    .iter()
                    .enumerate()
                    .map(|(index, (label, _))| (index, *label))
                    .collect();
                if widgets::segmented(ui, &mut scale, &options)
                    && let Some((_, value)) = SCALE_CHOICES.get(scale)
                {
                    app.prefs.ui_scale = *value;
                    app.persist_prefs();
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
            if widgets::check_label(
                ui,
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
                if widgets::check_label(ui, &mut auto, "Start SweepLoom in the tray when I sign in")
                    .changed()
                    && let Err(error) = autostart::set_enabled(auto)
                {
                    app.action_message = Some(error);
                }
            } else {
                widgets::caption(
                    ui,
                    "Sign-in autostart is available on Windows. Launch with --tray to start hidden.",
                );
            }
        },
    );
}

fn about(app: &SweepLoomApp, ui: &mut egui::Ui) {
    section(ui, "About", "", |ui| {
        egui::Grid::new("settings-about")
            .num_columns(2)
            .spacing([crate::theme::LG, 6.0])
            .show(ui, |ui| {
                let line = |ui: &mut egui::Ui, key: &str, value: String| {
                    ui.label(RichText::new(key).color(crate::theme::muted(ui)));
                    ui.label(value);
                    ui.end_row();
                };
                line(ui, "Telemetry", "None".into());
                line(
                    ui,
                    "License",
                    "MPL-2.0 (SweepLoom). Weavatrix crates remain MIT.".into(),
                );
                line(ui, "Home", app.locations.home.display().to_string());
                line(ui, "Config", app.locations.app_config.display().to_string());
                if let Some(snapshot) = &app.snapshot {
                    line(
                        ui,
                        "Observed processes",
                        snapshot.processes.len().to_string(),
                    );
                }
                line(ui, "Hidden launch", "sweeploom-gui --tray".into());
            });
    });
}
