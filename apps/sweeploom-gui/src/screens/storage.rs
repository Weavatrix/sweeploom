//! Storage scan and Folder Inspector tree.

use eframe::egui::{self, Color32, RichText};
use sweeploom_storage::DirectoryNode;

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::sort::{Col, Sort, header_button};
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
        ui.colored_label(Color32::from_rgb(240, 120, 120), error);
    }
    if let Some(report) = &app.inventory {
        ui.label(format!(
            "entries {} · projects {} · logical {}{}",
            report.entries,
            report.projects.len(),
            format_bytes(report.tree.logical_bytes),
            if report.capped { " · capped" } else { "" }
        ));
        ui.horizontal(|ui| {
            ui.label("Sort children");
            header_button(ui, &mut app.explorer_sort, Col::Size, "Size");
            header_button(ui, &mut app.explorer_sort, Col::Name, "Name");
        });
        ui.add_space(8.0);
        folder_tree(ui, &report.tree, 0, app.explorer_sort);
    } else {
        ui.label("Scan a folder to open the inspector. Symlinks are not followed.");
    }
}

fn folder_tree(ui: &mut egui::Ui, node: &DirectoryNode, depth: usize, sort: Sort) {
    if depth > 6 {
        return;
    }
    let name = node
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(".");
    let label = format!(
        "{}  {name}  {:?}",
        format_bytes(node.logical_bytes),
        node.category
    );
    if node.children.is_empty() {
        ui.label(RichText::new(label).size(16.0));
        return;
    }
    egui::CollapsingHeader::new(RichText::new(label).size(16.0))
        .id_salt(&node.path)
        .default_open(depth < 1)
        .show(ui, |ui| {
            let mut children: Vec<&DirectoryNode> = node.children.iter().collect();
            children.sort_by(|left, right| match sort.col {
                Col::Name => left.path.file_name().cmp(&right.path.file_name()),
                _ => left.logical_bytes.cmp(&right.logical_bytes),
            });
            if sort.desc {
                children.reverse();
            }
            for child in children {
                folder_tree(ui, child, depth + 1, sort);
            }
        });
}
