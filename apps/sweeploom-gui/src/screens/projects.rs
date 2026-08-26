//! Projects as a groupable table. Rebuild runs off the UI thread.

use eframe::egui::{self, RichText};
use egui_extras::{Column, TableBuilder};

use crate::app::SweepLoomApp;
use crate::format::{format_bytes, short_path};
use crate::nav::Nav;
use crate::sort::{Col, header_cell};
use crate::theme;
use crate::widgets::{page_title, pointer, table_scroll_height};

use super::project_rows::{
    Line, ProjectCard, ProjectGroup, collect_cards, sort_cards, table_lines,
};

pub fn ui_projects(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(
        ui,
        "Projects",
        "Cargo, npm, and Python artifacts from Review. Browser is for Chrome/Edge trees, not node_modules.",
    );
    toolbar(app, ui);
    ui.add_space(8.0);
    let mut cards = collect_cards(app);
    if cards.is_empty() {
        empty_hint(app, ui);
        return;
    }
    sort_cards(&mut cards, app.project_sort);
    ui.label(
        RichText::new(format!(
            "{} project(s). Click a group to collapse it. Click a row to open Explorer.",
            cards.len()
        ))
        .color(theme::muted(ui)),
    );
    draw_table(app, ui, &cards);
}

fn toolbar(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        let label = if app.scanning {
            "Working…"
        } else {
            "Rebuild review"
        };
        if pointer(ui.add_enabled(!app.scanning, egui::Button::new(label))).clicked() {
            app.rebuild_review();
        }
        if pointer(ui.button("Open Review")).clicked() {
            app.nav = Nav::Storage;
        }
        ui.label("Group by");
        for mode in [ProjectGroup::Kind, ProjectGroup::Parent, ProjectGroup::None] {
            if pointer(ui.selectable_label(app.project_group == mode, mode.label())).clicked() {
                app.project_group = mode;
            }
        }
    });
    if let Some(message) = &app.action_message {
        ui.label(message);
    }
}

fn empty_hint(app: &SweepLoomApp, ui: &mut egui::Ui) {
    if app.scanning {
        ui.label("Rebuilding review in the background. The window stays interactive.");
    } else {
        ui.label("No projects yet. Rebuild review to find Cargo.toml / package.json under GitHub and the current workspace.");
    }
}

fn draw_table(app: &mut SweepLoomApp, ui: &mut egui::Ui, cards: &[ProjectCard]) {
    let mut sort = app.project_sort;
    let lines = table_lines(cards, app.project_group, &app.collapsed_project_groups);
    let row_count = lines.len();
    let height = table_scroll_height(ui);
    let mut toggle = None;
    let mut open = None;
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .sense(egui::Sense::click())
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::remainder().at_least(220.0))
        .column(Column::auto().at_least(90.0))
        .column(Column::auto().at_least(90.0))
        .column(Column::auto().at_least(140.0))
        .header(32.0, |mut header| {
            header.col(|ui| header_cell(ui, &mut sort, Col::Name, "Name"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Status, "Kind"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Size, "Size"));
            header.col(|ui| {
                ui.strong("Artifacts");
            });
        })
        .body(|body| {
            body.rows(40.0, row_count, |mut row| {
                fill_line(&lines, cards, &mut row, &mut toggle, &mut open);
            });
        });
    app.project_sort = sort;
    apply_clicks(app, toggle, open);
}

fn fill_line(
    lines: &[Line],
    cards: &[ProjectCard],
    row: &mut egui_extras::TableRow<'_, '_>,
    toggle: &mut Option<String>,
    open: &mut Option<std::path::PathBuf>,
) {
    match lines.get(row.index()) {
        Some(Line::Group { key, title }) => fill_group(row, key, title, toggle),
        Some(Line::Project(index)) => {
            if let Some(card) = cards.get(*index) {
                fill_project(row, card, open);
            }
        }
        None => {}
    }
}

fn fill_group(
    row: &mut egui_extras::TableRow<'_, '_>,
    key: &str,
    title: &str,
    toggle: &mut Option<String>,
) {
    row.col(|ui| {
        ui.label(RichText::new(title).strong());
    });
    row.col(|_ui| {});
    row.col(|_ui| {});
    row.col(|_ui| {});
    if row.response().clicked() {
        *toggle = Some(key.to_owned());
    }
}

fn fill_project(
    row: &mut egui_extras::TableRow<'_, '_>,
    card: &ProjectCard,
    open: &mut Option<std::path::PathBuf>,
) {
    let name = card.name().to_owned();
    let path = short_path(&card.path);
    row.col(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(name).size(15.0).strong());
            ui.label(RichText::new(path).size(12.0).color(theme::muted(ui)));
        });
    });
    row.col(|ui| {
        ui.label(&card.kinds);
    });
    row.col(|ui| {
        ui.label(if card.bytes > 0 {
            format_bytes(card.bytes)
        } else {
            "—".to_owned()
        });
    });
    row.col(|ui| {
        ui.label(&card.artifacts);
    });
    if row.response().clicked() {
        *open = Some(card.path.clone());
    }
}

fn apply_clicks(app: &mut SweepLoomApp, toggle: Option<String>, open: Option<std::path::PathBuf>) {
    if let Some(key) = toggle
        && !app.collapsed_project_groups.remove(&key)
    {
        app.collapsed_project_groups.insert(key);
    }
    if let Some(path) = open {
        app.scan_root = path.display().to_string();
        app.nav = Nav::Explorer;
    }
}
