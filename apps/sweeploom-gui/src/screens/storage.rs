//! Storage scan and Folder Inspector tree.

use eframe::egui::{self, Color32};
use sweeploom_storage::DirectoryNode;

use crate::app::SweepLoomApp;
use crate::format::format_bytes;

pub fn ui_storage(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    ui.heading("Explorer");
    ui.horizontal(|ui| {
        ui.label("Root");
        ui.add(egui::TextEdit::singleline(&mut app.scan_root).desired_width(480.0));
        if ui.button("Scan").clicked() {
            app.run_scan();
        }
    });
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
        ui.add_space(8.0);
        folder_tree(ui, &report.tree, 0);
    } else {
        ui.label("Scan a folder to open the inspector. Symlinks are not followed.");
    }
}

fn folder_tree(ui: &mut egui::Ui, node: &DirectoryNode, depth: usize) {
    if depth > 6 {
        return;
    }
    let label = format!(
        "{}  {}  {:?}",
        format_bytes(node.logical_bytes),
        node.path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("."),
        node.category
    );
    if node.children.is_empty() {
        ui.label(label);
        return;
    }
    egui::CollapsingHeader::new(label)
        .default_open(depth < 1)
        .show(ui, |ui| {
            for child in &node.children {
                folder_tree(ui, child, depth + 1);
            }
        });
}
