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
        "Folder inspector. Symlinks are not followed. Scan runs in the background. Click a folder to expand it. Double-click to scan it as Root.",
    );
    ui.horizontal(|ui| {
        ui.label("Root");
        ui.add(
            egui::TextEdit::singleline(&mut app.scan_root)
                .desired_width(280.0)
                .clip_text(true),
        );
        let scan = if app.scanning { "Scanning…" } else { "Scan" };
        if crate::widgets::pointer(ui.add_enabled(!app.scanning, egui::Button::new(scan))).clicked()
        {
            app.run_scan();
        }
    });
    if app.scanning {
        ui.horizontal(|ui| {
            ui.allocate_ui(egui::vec2(18.0, 18.0), |ui| {
                ui.spinner();
            });
            ui.label(
                RichText::new(format!(
                    "{} entries · {}",
                    app.scan_entries,
                    format_bytes(app.scan_bytes)
                ))
                .color(crate::theme::muted(ui)),
            );
            let hint = if app.scan_hint.is_empty() {
                "walking…".to_owned()
            } else {
                std::path::Path::new(&app.scan_hint)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(&app.scan_hint)
                    .to_owned()
            };
            ui.add(egui::Label::new(RichText::new(hint).color(crate::theme::muted(ui))).truncate());
        });
    } else if let Some(report) = &app.inventory {
        ui.label(
            RichText::new(format!(
                "{} entries · {} projects · {}",
                report.entries,
                report.projects.len(),
                format_bytes(report.tree.logical_bytes),
            ))
            .color(crate::theme::muted(ui)),
        );
    }
    if let Some(error) = &app.inventory_error {
        ui.colored_label(ui.visuals().error_fg_color, error);
    }
    if app.inventory.is_some() {
        folder_table(app, ui);
    } else if !app.scanning {
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
        .id_salt("explorer-grid")
        .striped(true)
        .resizable(true)
        .sense(egui::Sense::click())
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::remainder().at_least(160.0).clip(true))
        .column(Column::exact(72.0).clip(true))
        .column(Column::exact(100.0).clip(true))
        .column(Column::exact(64.0).clip(true))
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
            body.rows(crate::widgets::TABLE_ROW, row_count, |mut row| {
                let Some(line) = lines.get(row.index()) else {
                    return;
                };
                let disclosed = fill_line(&mut row, line);
                if row.response().double_clicked() {
                    picked = Some(line.path.display().to_string());
                } else if (disclosed || row.response().clicked()) && line.has_children {
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

fn fill_line(row: &mut egui_extras::TableRow<'_, '_>, line: &Line) -> bool {
    let name = line.name.clone();
    let size = format_bytes(line.bytes);
    let files = line.files.to_string();
    let category = line.category.label();
    let depth = line.depth;
    let glyph = category_glyph(line.category);
    let open = line.expanded;
    let can_open = line.has_children;
    let mut disclosed = false;
    row.col(|ui| {
        ui.add_space(depth as f32 * 12.0);
        if can_open {
            disclosed = crate::widgets::disclose(ui, open, crate::theme::accent());
        } else {
            ui.add_space(11.0);
        }
        crate::icons::show(ui, glyph, 14.0, crate::theme::accent());
        ui.add(egui::Label::new(RichText::new(name).size(15.0)).truncate());
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
    disclosed
}

fn category_glyph(_category: sweeploom_storage::PathCategory) -> crate::icons::Glyph {
    crate::icons::Glyph::Explorer
}
