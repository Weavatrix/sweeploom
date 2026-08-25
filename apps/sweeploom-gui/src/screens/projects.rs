//! Projects with Source/Artifact Heat and Git safety from Weavatrix Git.

use eframe::egui;
use sweeploom_dev::{cargo_offers, classify_project, inspect, node_offers, python_offers};

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::widgets::page_title;

pub fn ui_projects(app: &SweepLoomApp, ui: &mut egui::Ui) {
    page_title(
        ui,
        "Projects",
        "Source Heat and Artifact Heat are independent. A fresh target does not make source HOT.",
    );
    let Some(report) = &app.inventory else {
        ui.label("Run Explorer scan to discover project markers.");
        return;
    };
    let now = std::time::SystemTime::now();
    let processes = app
        .snapshot
        .as_ref()
        .map(|item| item.processes.as_slice())
        .unwrap_or(&[]);
    for project in &report.projects {
        let (source, artifact) = report.project_heat(project, now);
        ui.separator();
        ui.label(project.display().to_string());
        ui.label(format!(
            "kind={:?}  source={:?}  artifact={:?}  git={}",
            classify_project(project),
            source,
            artifact,
            inspect(project).label()
        ));
        for offer in cargo_offers(project, processes) {
            ui.label(format!(
                "  cargo {:?}  {}  rebuild={:?}{}",
                offer.mode,
                format_bytes(offer.logical_bytes),
                offer.rebuild,
                if offer.blocked { "  BLOCKED" } else { "" }
            ));
        }
        for offer in node_offers(project, processes) {
            ui.label(format!(
                "  node_modules  {}  rebuild={:?}{}",
                format_bytes(offer.logical_bytes),
                offer.rebuild,
                if offer.blocked { "  BLOCKED" } else { "" }
            ));
        }
        for offer in python_offers(project, processes) {
            ui.label(format!(
                "  python {}  {}  rebuild={:?}{}",
                offer.label,
                format_bytes(offer.logical_bytes),
                offer.rebuild,
                if offer.blocked { "  BLOCKED" } else { "" }
            ));
        }
    }
}
