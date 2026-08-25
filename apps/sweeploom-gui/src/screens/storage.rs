//! Storage scan and Folder Inspector tree.

use eframe::egui::{self, RichText};
use egui_extras::{Column, TableBuilder};
use sweeploom_storage::DirectoryNode;

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::sort::{Col, Sort, header_cell};
use crate::widgets::page_title;

pub fn ui_storage(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(
        ui,
        "Explorer",
        "Folder inspector. Symlinks are not followed. Scan runs in the background.",
    );
    ui.horizontal(|ui| {
        ui.label("Root");
        ui.add(egui::TextEdit::singleline(&mut app.scan_root).desired_width(480.0));
        let scan = if app.scanning { "Scanning…" } else { "Scan" };
        if ui
            .add_enabled(!app.scanning, egui::Button::new(scan))
            .clicked()
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
    if let Some(report) = &app.inventory {
        ui.label(format!(
            "entries {} · projects {} · logical {}{}",
            report.entries,
            report.projects.len(),
            format_bytes(report.tree.logical_bytes),
            if report.capped { " · capped" } else { "" }
        ));
        ui.add_space(8.0);
        folder_table(ui, &report.tree, &mut app.explorer_sort);
    } else {
        ui.label("Scan a folder to open the inspector. Symlinks are not followed.");
    }
}

fn folder_table(ui: &mut egui::Ui, root: &DirectoryNode, sort: &mut Sort) {
    let mut rows = Vec::new();
    collect_rows(root, 0, *sort, &mut rows);
    let row_count = rows.len();
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(110.0))
        .column(Column::remainder().at_least(220.0))
        .column(Column::auto().at_least(120.0))
        .column(Column::auto().at_least(80.0))
        .header(32.0, |mut header| {
            header.col(|ui| header_cell(ui, sort, Col::Size, "Size"));
            header.col(|ui| header_cell(ui, sort, Col::Name, "Name"));
            header.col(|ui| {
                ui.strong("Category");
            });
            header.col(|ui| {
                ui.strong("Files");
            });
        })
        .body(|body| {
            body.rows(26.0, row_count, |mut row| {
                let Some((depth, node)) = rows.get(row.index()) else {
                    return;
                };
                let name = node
                    .path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(".");
                let indent = "    ".repeat(*depth);
                row.col(|ui| {
                    ui.label(format_bytes(node.logical_bytes));
                });
                row.col(|ui| {
                    ui.label(RichText::new(format!("{indent}{name}")).size(16.0));
                });
                row.col(|ui| {
                    ui.label(format!("{:?}", node.category));
                });
                row.col(|ui| {
                    ui.label(node.files.to_string());
                });
            });
        });
}

fn collect_rows<'a>(
    node: &'a DirectoryNode,
    depth: usize,
    sort: Sort,
    out: &mut Vec<(usize, &'a DirectoryNode)>,
) {
    if depth > 5 {
        return;
    }
    let mut children: Vec<&DirectoryNode> = node.children.iter().collect();
    children.sort_by(|left, right| match sort.col {
        Col::Name => left.path.file_name().cmp(&right.path.file_name()),
        _ => left.logical_bytes.cmp(&right.logical_bytes),
    });
    if sort.desc {
        children.reverse();
    }
    for child in children {
        out.push((depth, child));
        collect_rows(child, depth + 1, sort, out);
    }
}
