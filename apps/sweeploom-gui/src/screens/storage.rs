//! Storage scan and Folder Inspector tree.

use eframe::egui::{self, RichText};
use egui_extras::{Column, TableBuilder};

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::sort::{Col, header_cell};
use crate::widgets::{page_title, table_scroll_height};

use super::explorer_rows::{self, Line};

pub fn ui_storage(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(
        ui,
        "Explorer",
        "Folder inspector. Symlinks are not followed. Scan runs in the background.",
    );
    ui.horizontal(|ui| {
        ui.label("Root");
        let width = (ui.available_width() - 88.0).max(160.0);
        ui.add(egui::TextEdit::singleline(&mut app.scan_root).desired_width(width));
        let scan = if app.scanning { "Scanning…" } else { "Scan" };
        if crate::widgets::pointer(ui.add_enabled(!app.scanning, egui::Button::new(scan))).clicked()
        {
            app.run_scan();
        }
    });
    if app.scanning {
        ui.label("Walk is running in the background. The window stays interactive.");
    }
    if let Some(error) = &app.inventory_error {
        ui.colored_label(ui.visuals().error_fg_color, error);
    }
    let summary = app.inventory.as_ref().map(|report| {
        format!(
            "entries {} · projects {} · logical {}{}",
            report.entries,
            report.projects.len(),
            format_bytes(report.tree.logical_bytes),
            if report.capped { " · capped" } else { "" }
        )
    });
    if let Some(summary) = summary {
        ui.label(summary);
        ui.add_space(8.0);
        ui.label(
            RichText::new("Click a folder to expand it. Double-click to scan it as Root.")
                .size(13.0)
                .color(crate::theme::muted(ui)),
        );
        folder_table(app, ui);
    } else {
        ui.label("Scan a folder to open the inspector. Symlinks are not followed.");
    }
}

fn folder_table(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let mut sort = app.explorer_sort;
    let expanded = app.expanded_explorer.clone();
    let lines = {
        let Some(report) = &app.inventory else {
            return;
        };
        explorer_rows::visible_lines(&report.tree, sort, &expanded)
    };
    let row_count = lines.len();
    let height = table_scroll_height(ui);
    let mut toggle = None;
    let mut picked = None;
    TableBuilder::new(ui)
        .id_salt("explorer-tree")
        .striped(true)
        .resizable(true)
        .sense(egui::Sense::click())
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::remainder().at_least(140.0).clip(true))
        .column(Column::auto().at_least(56.0).at_most(88.0).clip(true))
        .column(Column::auto().at_least(64.0).at_most(110.0).clip(true))
        .column(Column::auto().at_least(48.0).at_most(72.0).clip(true))
        .header(32.0, |mut header| {
            header.col(|ui| header_cell(ui, &mut sort, Col::Name, "Name"));
            header.col(|ui| header_cell(ui, &mut sort, Col::Size, "Size"));
            header.col(|ui| {
                ui.strong("Category");
            });
            header.col(|ui| {
                ui.strong("Files");
            });
        })
        .body(|body| {
            body.rows(26.0, row_count, |mut row| {
                let Some(line) = lines.get(row.index()) else {
                    return;
                };
                fill_line(&mut row, line);
                if row.response().double_clicked() {
                    picked = Some(line.path.display().to_string());
                } else if row.response().clicked() && line.has_children {
                    toggle = Some(explorer_rows::path_key(&line.path));
                } else if row.response().clicked() {
                    picked = Some(line.path.display().to_string());
                }
            });
        });
    app.explorer_sort = sort;
    if let Some(key) = toggle
        && !app.expanded_explorer.remove(&key)
    {
        app.expanded_explorer.insert(key);
    }
    if let Some(path) = picked {
        app.scan_root = path;
    }
}

fn fill_line(row: &mut egui_extras::TableRow<'_, '_>, line: &Line) {
    let mark = if !line.has_children {
        "  "
    } else if line.expanded {
        "▾ "
    } else {
        "▸ "
    };
    let name = format!("{mark}{}", line.name);
    let size = format_bytes(line.bytes);
    let files = line.files.to_string();
    let category = line.category.label();
    let depth = line.depth;
    let glyph = category_glyph(line.category);
    row.col(|ui| {
        ui.add_space(depth as f32 * 12.0);
        crate::icons::show(ui, glyph, 14.0, crate::theme::accent());
        ui.add(egui::Label::new(RichText::new(name).size(16.0)).truncate());
    });
    row.col(|ui| {
        ui.label(size);
    });
    row.col(|ui| {
        ui.label(category);
    });
    row.col(|ui| {
        ui.label(files);
    });
}

fn category_glyph(category: sweeploom_storage::PathCategory) -> crate::icons::Glyph {
    use crate::icons::Glyph;
    use sweeploom_storage::PathCategory;
    match category {
        PathCategory::Generated | PathCategory::Cache => Glyph::Disk,
        PathCategory::Dependencies => Glyph::Projects,
        PathCategory::UserData => Glyph::Volume,
        PathCategory::Source | PathCategory::Unknown => Glyph::Explorer,
    }
}
