//! Projects from Review discovery. Does not walk node_modules on the UI thread.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use eframe::egui::{self, CornerRadius, Margin, RichText};
use sweeploom_core::CandidateOwner;
use sweeploom_dev::classify_project;

use crate::app::SweepLoomApp;
use crate::format::{format_bytes, row_caption, short_path};
use crate::icons::{self, Glyph};
use crate::nav::Nav;
use crate::widgets::{list_row, list_row_at, page_title};

struct ProjectCard {
    path: PathBuf,
    bytes: u64,
    offers: Vec<(String, u64, &'static str)>,
}

pub fn ui_projects(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(
        ui,
        "Projects",
        "Cargo, npm, and Python artifacts from Review. Browser is for Chrome/Edge trees, not node_modules.",
    );
    ui.horizontal_wrapped(|ui| {
        if crate::widgets::pointer(ui.button("Rebuild review")).clicked() {
            app.rebuild_review();
        }
        if crate::widgets::pointer(ui.button("Open Review")).clicked() {
            app.nav = Nav::Storage;
        }
    });
    ui.add_space(8.0);
    let cards = collect_cards(app);
    if cards.is_empty() {
        ui.label("No projects yet. Rebuild review to find Cargo.toml / package.json under GitHub and the current workspace.");
        return;
    }
    ui.label(
        RichText::new(format!(
            "{} project(s). Offers reuse the Review scan — this page does not walk node_modules again.",
            cards.len()
        ))
        .color(crate::theme::muted(ui)),
    );
    ui.add_space(8.0);
    for card in cards.into_iter().take(48) {
        draw_card(app, ui, &card);
    }
}

fn collect_cards(app: &SweepLoomApp) -> Vec<ProjectCard> {
    let mut map: BTreeMap<PathBuf, ProjectCard> = BTreeMap::new();
    for path in &app.project_roots {
        map.entry(path.clone())
            .or_insert_with(|| ProjectCard::new(path));
    }
    if let Some(report) = &app.inventory {
        for path in &report.projects {
            map.entry(path.clone())
                .or_insert_with(|| ProjectCard::new(path));
        }
    }
    for row in &app.review {
        let CandidateOwner::Project(id) = &row.candidate.owner else {
            continue;
        };
        let card = map
            .entry(id.0.clone())
            .or_insert_with(|| ProjectCard::new(&id.0));
        card.bytes = card.bytes.saturating_add(row.candidate.logical_bytes);
        card.offers.push((
            row_caption(&row.title),
            row.candidate.logical_bytes,
            row.candidate.rebuild.cost.label(),
        ));
    }
    let mut cards: Vec<ProjectCard> = map.into_values().collect();
    cards.sort_by(|left, right| {
        right
            .bytes
            .cmp(&left.bytes)
            .then(left.path.cmp(&right.path))
    });
    cards
}

impl ProjectCard {
    fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            bytes: 0,
            offers: Vec::new(),
        }
    }
}

fn draw_card(app: &mut SweepLoomApp, ui: &mut egui::Ui, card: &ProjectCard) {
    let kinds = classify_project(&card.path)
        .into_iter()
        .map(|kind| kind.label())
        .collect::<Vec<_>>()
        .join(", ");
    let name = card
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project");
    let mut open_explorer = false;
    let mut open_review = false;
    egui::Frame::default()
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                icons::show(ui, Glyph::Projects, 16.0, crate::theme::accent());
                if crate::widgets::pointer(
                    ui.add(egui::Button::new(RichText::new(name).size(16.0)).frame(false)),
                )
                .clicked()
                {
                    open_explorer = true;
                }
                ui.label(RichText::new(&kinds).color(crate::theme::muted(ui)));
                if card.bytes > 0 {
                    ui.label(RichText::new(format_bytes(card.bytes)).strong());
                }
            });
            ui.label(
                RichText::new(short_path(&card.path))
                    .size(12.0)
                    .color(crate::theme::muted(ui)),
            );
            if card.offers.is_empty() {
                list_row(
                    ui,
                    "No generated artifacts in Review yet",
                    "",
                    "target / node_modules appear after a local build",
                );
            } else {
                for (title, bytes, rebuild) in card.offers.iter().take(6) {
                    if list_row_at(
                        ui,
                        title,
                        &format_bytes(*bytes),
                        &format!("rebuild={rebuild}"),
                    )
                    .clicked()
                    {
                        open_review = true;
                    }
                }
            }
        });
    if open_explorer {
        app.scan_root = card.path.display().to_string();
        app.nav = Nav::Explorer;
    }
    if open_review {
        app.nav = Nav::Storage;
    }
    ui.add_space(10.0);
}
