//! Project list derived from the last inventory scan.

use eframe::egui;

use crate::app::SweepLoomApp;

pub fn ui_projects(app: &SweepLoomApp, ui: &mut egui::Ui) {
    ui.heading("Projects");
    ui.label(
        "Source Heat and Artifact Heat are independent. A fresh `target` does not make source HOT.",
    );
    if let Some(report) = &app.inventory {
        for project in &report.projects {
            ui.label(project.display().to_string());
        }
    } else {
        ui.label(
            "Run a Storage scan to discover project markers (Cargo.toml, package.json, pyproject.toml, …).",
        );
    }
}
