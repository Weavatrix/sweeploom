//! Projects with Source/Artifact Heat and Git safety from Weavatrix Git.

use std::path::PathBuf;

use eframe::egui;
use sweeploom_dev::{cargo_offers, classify_project, inspect, node_offers, python_offers};

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::widgets::{list_row, page_title};

pub fn ui_projects(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(
        ui,
        "Projects",
        "Source Heat and Artifact Heat are independent. npm/Cargo live here and on Review, not under Browser.",
    );
    if ui.button("Rebuild review").clicked() {
        app.rebuild_review();
    }
    if app.inventory.is_some() {
        draw_inventory(app, ui);
        return;
    }
    if app.project_roots.is_empty() {
        ui.label(
            "No projects yet. Rebuild review to find Cargo.toml / package.json, or scan Explorer.",
        );
        return;
    }
    let projects = app.project_roots.clone();
    draw_offers(app, ui, &projects);
}

fn draw_inventory(app: &SweepLoomApp, ui: &mut egui::Ui) {
    let Some(report) = &app.inventory else {
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
        let kinds = classify_project(project)
            .into_iter()
            .map(|kind| kind.label())
            .collect::<Vec<_>>()
            .join(", ");
        list_row(
            ui,
            &project.display().to_string(),
            &format!(
                "{kinds} · source {} · artifact {}",
                source.label(),
                artifact.label()
            ),
            inspect(project).label(),
        );
        draw_project_offers(ui, project, processes);
    }
}

fn draw_offers(app: &SweepLoomApp, ui: &mut egui::Ui, projects: &[PathBuf]) {
    let processes = app
        .snapshot
        .as_ref()
        .map(|item| item.processes.as_slice())
        .unwrap_or(&[]);
    ui.label("Offers from Review discovery. Explorer scan adds source/artifact heat.");
    for project in projects {
        let kinds = classify_project(project)
            .into_iter()
            .map(|kind| kind.label())
            .collect::<Vec<_>>()
            .join(", ");
        list_row(
            ui,
            &project.display().to_string(),
            &kinds,
            inspect(project).label(),
        );
        draw_project_offers(ui, project, processes);
    }
}

fn draw_project_offers(
    ui: &mut egui::Ui,
    project: &std::path::Path,
    processes: &[sweeploom_core::ProcessSnapshot],
) {
    for offer in cargo_offers(project, processes) {
        list_row(
            ui,
            &format!("cargo {}", offer.mode.label()),
            &format_bytes(offer.logical_bytes),
            &format!(
                "rebuild={}{}",
                offer.rebuild.label(),
                if offer.blocked { " · blocked" } else { "" }
            ),
        );
    }
    for offer in node_offers(project, processes) {
        list_row(
            ui,
            "node_modules",
            &format_bytes(offer.logical_bytes),
            &format!(
                "rebuild={}{}",
                offer.rebuild.label(),
                if offer.blocked { " · blocked" } else { "" }
            ),
        );
    }
    for offer in python_offers(project, processes) {
        list_row(
            ui,
            &format!("python {}", offer.label),
            &format_bytes(offer.logical_bytes),
            &format!(
                "rebuild={}{}",
                offer.rebuild.label(),
                if offer.blocked { " · blocked" } else { "" }
            ),
        );
    }
}
