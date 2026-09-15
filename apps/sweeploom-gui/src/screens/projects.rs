//! Projects as a groupable table. Rebuild runs off the UI thread.

use eframe::egui::{self, RichText};
use egui_extras::{Column, TableBuilder};

use crate::app::SweepLoomApp;
use crate::format::{format_bytes, short_path};
use crate::nav::Nav;
use crate::sort::{Col, header_cell};
use crate::theme;
use crate::widgets::{self, page_title, pointer, table_scroll_height};

use super::project_rows::{Line, ProjectGroup, refresh_cards, sort_cards, table_lines};

pub fn ui_projects(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(
        ui,
        "Projects",
        "Cargo, npm, and Python artifacts from Review. Click a folder to collapse it. Click a project to open Explorer. Browser is for Chrome/Edge trees, not node_modules.",
    );
    toolbar(app, ui);
    refresh_cards(app);
    if app.project_cards.is_empty() {
        empty_hint(app, ui);
        return;
    }
    sort_cards(&mut app.project_cards, app.project_sort);
    draw_table(app, ui);
}

fn toolbar(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        let label = if app.scanning {
            "Working…"
        } else {
            "Rebuild"
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
    widgets::action_note(ui, app.action_message.as_deref());
}

fn empty_hint(app: &SweepLoomApp, ui: &mut egui::Ui) {
    if app.scanning {
        ui.label("Rebuilding review in the background. The window stays interactive.");
    } else {
        ui.label("No projects yet. Rebuild review to find Cargo.toml / package.json under GitHub and the current workspace.");
    }
}

fn draw_table(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let mut sort = app.project_sort;
    let nested = app.project_group != ProjectGroup::None;
    let lines = table_lines(
        &app.project_cards,
        app.project_group,
        &app.collapsed_project_groups,
        sort,
    );
    let row_count = lines.len();
    let height = table_scroll_height(ui);
    let mut toggle = None;
    let mut open = None;
    TableBuilder::new(ui)
        .id_salt("projects-grid")
        .striped(true)
        .resizable(true)
        .sense(egui::Sense::click())
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::remainder().at_least(160.0).clip(true))
        .column(Column::exact(80.0).clip(true))
        .column(Column::exact(72.0).clip(true))
        .column(Column::exact(140.0).clip(true))
        .header(32.0, |mut header| {
            header.col(|ui| header_cell(ui, &mut sort, Col::Name, "Name"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Status, "Kind"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Size, "Size"));
            header.col(|ui| {
                ui.strong("Artifacts");
            });
        })
        .body(|body| {
            body.rows(widgets::TABLE_ROW, row_count, |mut row| {
                fill_line(
                    &lines,
                    &app.project_cards,
                    nested,
                    &mut row,
                    &mut toggle,
                    &mut open,
                );
            });
        });
    app.project_sort = sort;
    apply_clicks(app, toggle, open);
}

fn fill_line(
    lines: &[Line],
    cards: &[super::project_rows::ProjectCard],
    nested: bool,
    row: &mut egui_extras::TableRow<'_, '_>,
    toggle: &mut Option<String>,
    open: &mut Option<std::path::PathBuf>,
) {
    match lines.get(row.index()) {
        Some(Line::Group {
            key,
            title,
            count,
            bytes,
            expanded,
        }) => fill_group(row, key, title, *count, *bytes, *expanded, toggle),
        Some(Line::Project(index)) => {
            if let Some(card) = cards.get(*index) {
                fill_project(row, card, nested, open);
            }
        }
        None => {}
    }
}

fn fill_group(
    row: &mut egui_extras::TableRow<'_, '_>,
    key: &str,
    title: &str,
    count: usize,
    bytes: u64,
    expanded: bool,
    toggle: &mut Option<String>,
) {
    row.col(|ui| {
        crate::icons::show(ui, crate::icons::Glyph::Explorer, 14.0, theme::accent());
        if crate::widgets::disclose(ui, expanded, theme::accent()) {
            *toggle = Some(key.to_owned());
        }
        ui.add(egui::Label::new(RichText::new(format!("{title}  ·  {count}")).strong()).truncate());
    });
    row.col(|ui| {
        ui.label("Folder");
    });
    row.col(|ui| {
        ui.label(if bytes > 0 {
            format_bytes(bytes)
        } else {
            "—".to_owned()
        });
    });
    row.col(|_ui| {});
    if row.response().clicked() {
        *toggle = Some(key.to_owned());
    }
}

fn fill_project(
    row: &mut egui_extras::TableRow<'_, '_>,
    card: &super::project_rows::ProjectCard,
    nested: bool,
    open: &mut Option<std::path::PathBuf>,
) {
    let name = card.name().to_owned();
    let path = short_path(&card.path);
    row.col(|ui| {
        if nested {
            ui.add_space(18.0);
        }
        crate::icons::show(ui, crate::icons::Glyph::Projects, 14.0, theme::muted(ui));
        ui.add(egui::Label::new(RichText::new(name).size(15.0)).truncate())
            .on_hover_text(&path);
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
        ui.add(egui::Label::new(&card.artifacts).truncate());
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
