//! Projects with Source/Artifact Heat and Git safety from Weavatrix Git.

use eframe::egui;
use sweeploom_dev::{cargo_offers, classify_project, inspect};
use weavatrix_git::WorktreeSafetyLevel;

use crate::app::SweepLoomApp;
use crate::format::format_bytes;

pub fn ui_projects(app: &SweepLoomApp, ui: &mut egui::Ui) {
    ui.heading("Projects");
    ui.label(
        "Source Heat and Artifact Heat are independent. A fresh `target` does not make source HOT.",
    );
    let Some(report) = &app.inventory else {
        ui.label("Run a Storage scan to discover project markers.");
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
        let kinds = classify_project(project);
        let git = inspect(project);
        ui.separator();
        ui.label(project.display().to_string());
        ui.label(format!(
            "kind={:?}  source={:?}  artifact={:?}  git={}",
            kinds,
            source,
            artifact,
            git_label(&git)
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
    }
}

fn git_label(git: &sweeploom_dev::GitSafety) -> &'static str {
    match git {
        sweeploom_dev::GitSafety::NotARepository => "none",
        sweeploom_dev::GitSafety::Unknown => "unknown",
        sweeploom_dev::GitSafety::Known(safety) => match safety.level {
            WorktreeSafetyLevel::Clean => "clean",
            WorktreeSafetyLevel::IgnoredOnly => "ignored-only",
            WorktreeSafetyLevel::HasUntracked => "untracked",
            WorktreeSafetyLevel::DirtyTracked => "dirty",
            WorktreeSafetyLevel::Unknown => "unknown",
        },
    }
}
