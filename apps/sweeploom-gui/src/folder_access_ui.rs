//! Folder access UI for the App Store edition: grant prompts, the first-run
//! card and the Settings list.

use std::path::{Path, PathBuf};

use eframe::egui::{self, RichText};

use crate::app::SweepLoomApp;
use crate::folder_access::{self, short};
use crate::{edition, theme, widgets};

const WHY: &str = "The App Store edition runs in the macOS App Sandbox, so it only sees folders you grant. Grant your project folders to find projects and their build output, ~/Library/Developer for Xcode data, and package caches such as ~/.cargo or ~/.npm. Remove access any time in Settings.";

impl SweepLoomApp {
    /// Open the folder panel at `start`; returns the folders the user granted.
    pub(crate) fn grant_folder(&mut self, start: Option<&Path>) -> Vec<PathBuf> {
        match self.folder_access.request(start) {
            Ok(added) => {
                if !added.is_empty() {
                    self.action_message = Some(format!(
                        "SweepLoom can now read {} more folder(s). Rescanning…",
                        added.len()
                    ));
                }
                added
            }
            Err(error) => {
                self.action_message = Some(error);
                Vec::new()
            }
        }
    }

    /// Forget a grant and drop what was listed from it on the next rescan.
    pub(crate) fn revoke_folder(&mut self, path: &Path) {
        self.folder_access.remove(path);
        self.action_message = Some(format!(
            "Removed access to {}.",
            short(&self.locations.home, path)
        ));
    }

    /// After grants change: relist sources, rediscover projects, re-size AI stores.
    pub(crate) fn poll_folder_access(&mut self) {
        #[cfg(debug_assertions)]
        if edition::STORE
            && let Some(start) = folder_access::debug::panel_request()
        {
            self.grant_folder(Some(&start));
        }
        if !edition::STORE || self.scanning || !self.folder_access.take_changed() {
            return;
        }
        self.native_cleanup.access_changed();
        if !self.ai_listing {
            self.ai_offers = None;
        }
        if self.scan_rx.is_none() && !folder_access::allowed(Path::new(self.scan_root.trim())) {
            self.scan_root = self
                .folder_access
                .active()
                .first()
                .map_or_else(String::new, |path| path.display().to_string());
        }
        self.rebuild_review();
    }
}

/// Card with a grant button per suggested folder, plus Add folder….
pub(crate) fn prompt(
    app: &mut SweepLoomApp,
    ui: &mut egui::Ui,
    title: &str,
    body: &str,
    suggestions: &[PathBuf],
) {
    let home = app.locations.home.clone();
    let mut pick: Option<Option<PathBuf>> = None;
    widgets::card(ui, |ui| {
        ui.label(RichText::new(title).size(15.5).strong());
        ui.add_space(theme::XS);
        widgets::caption(ui, body);
        ui.add_space(theme::SM);
        ui.horizontal_wrapped(|ui| {
            for (index, path) in suggestions.iter().enumerate() {
                let text = format!("Grant {}", short(&home, path));
                let clicked = if index == 0 {
                    widgets::primary_button(ui, &text).clicked()
                } else {
                    widgets::button(ui, &text, true).clicked()
                };
                if clicked {
                    pick = Some(Some(path.clone()));
                }
            }
            if widgets::button(ui, "Add folder…", true).clicked() {
                pick = Some(None);
            }
        });
    });
    ui.add_space(theme::MD);
    if let Some(start) = pick {
        app.grant_folder(start.as_deref());
    }
}

/// Explorer header controls: scan a granted folder, or grant and scan a new one.
pub(crate) fn explorer_picker(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    if !edition::STORE {
        return;
    }
    let home = app.locations.home.clone();
    let mut chosen = None;
    egui::ComboBox::from_id_salt("explorer-granted")
        .selected_text("Granted folders…")
        .show_ui(ui, |ui| {
            for path in app.folder_access.active() {
                if ui.button(short(&home, &path)).clicked() {
                    chosen = Some(path);
                }
            }
        });
    if widgets::button(ui, "Choose folder…", app.scan_rx.is_none()).clicked() {
        chosen = app.grant_folder(None).into_iter().next();
    }
    if let Some(path) = chosen {
        app.scan_root = path.display().to_string();
        app.run_scan();
    }
}

/// Developer, Xcode and cache folders that exist and are not granted yet.
pub(crate) fn starter_folders(home: &Path) -> Vec<PathBuf> {
    [
        folder_access::DEVELOPER,
        folder_access::XCODE,
        folder_access::CACHES,
    ]
    .into_iter()
    .flat_map(|group| folder_access::missing(home, group))
    .collect()
}

/// First-run card, shown while no folder is granted.
pub(crate) fn onboarding(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    if !edition::STORE || !app.folder_access.active().is_empty() {
        return;
    }
    let suggestions = starter_folders(&app.locations.home);
    prompt(
        app,
        ui,
        "Choose the folders SweepLoom may measure and clean",
        WHY,
        &suggestions,
    );
}

/// Settings → Folder access: granted folders, suggestions and Add folder….
pub(crate) fn settings(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    if !edition::STORE {
        return;
    }
    let home = app.locations.home.clone();
    let suggestions = starter_folders(&home);
    let mut revoke = None;
    let mut pick: Option<Option<PathBuf>> = None;
    widgets::section(ui, "Folder access", WHY, |ui| {
        if app.folder_access.grants().is_empty() {
            widgets::caption(ui, "No folders granted yet.");
        }
        for grant in app.folder_access.grants() {
            ui.horizontal(|ui| {
                ui.label(RichText::new(short(&home, &grant.path)).strong())
                    .on_hover_text(grant.path.display().to_string());
                match &grant.error {
                    None => widgets::pill(ui, "Granted", theme::Tone::Ok),
                    Some(error) => widgets::pill(ui, "Access lost", theme::Tone::Warn)
                        .on_hover_text(error.as_str()),
                };
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if widgets::button(ui, "Remove", true).clicked() {
                        revoke = Some(grant.path.clone());
                    }
                    if grant.error.is_some() && widgets::button(ui, "Grant again", true).clicked() {
                        pick = Some(Some(grant.path.clone()));
                    }
                });
            });
        }
        ui.add_space(theme::SM);
        ui.horizontal_wrapped(|ui| {
            for path in &suggestions {
                if widgets::button(ui, &format!("Grant {}", short(&home, path)), true).clicked() {
                    pick = Some(Some(path.clone()));
                }
            }
            if widgets::primary_button(ui, "Add folder…").clicked() {
                pick = Some(None);
            }
        });
    });
    if let Some(path) = revoke {
        app.revoke_folder(&path);
    }
    if let Some(start) = pick {
        app.grant_folder(start.as_deref());
    }
}
