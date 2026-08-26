//! Declarative TOML cleaners. Packs are data; nothing here is executed.

use std::path::{Path, PathBuf};

use sweeploom_rules::load_packs;

use crate::app::SweepLoomApp;
use crate::widgets::{list_row, page_title, section};

pub fn ui_rules(app: &SweepLoomApp, ui: &mut eframe::egui::Ui) {
    page_title(
        ui,
        "Rules",
        "Declarative TOML cleaners. No shell. Downloaded packs are never executed.",
    );
    let files = load_visible_packs(app);
    if files.is_empty() {
        ui.label("No rule packs found under ./rules or the SweepLoom config rules directory.");
        return;
    }
    for file in files {
        let title = pack_title(&file.path);
        match file.pack {
            Ok(pack) => section(
                ui,
                &title,
                &format!(
                    "schema {} · {} cleaner(s) · {}",
                    pack.schema,
                    pack.cleaner.len(),
                    file.path.display()
                ),
                |ui| {
                    for rule in pack.cleaner {
                        let label = rule.label.as_deref().unwrap_or(&rule.id);
                        let detail = format!(
                            "{} · {}{}",
                            rule.safety_level().label(),
                            rule.deletion_strategy().label(),
                            markers_suffix(&rule.markers, &rule.paths)
                        );
                        list_row(ui, &rule.id, label, &detail);
                    }
                },
            ),
            Err(error) => {
                ui.colored_label(
                    ui.visuals().warn_fg_color,
                    format!("{title}: not loaded: {error}"),
                );
            }
        }
    }
}

fn pack_title(path: &Path) -> String {
    path.file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("pack")
        .to_owned()
}

fn markers_suffix(markers: &[String], paths: &[String]) -> String {
    let mut bits = Vec::new();
    if let Some(marker) = markers.first() {
        bits.push(format!("marker {marker}"));
    }
    if let Some(rel) = paths.first() {
        bits.push(rel.clone());
    }
    if bits.is_empty() {
        String::new()
    } else {
        format!(" · {}", bits.join(" · "))
    }
}

fn load_visible_packs(app: &SweepLoomApp) -> Vec<sweeploom_rules::LoadedRuleFile> {
    let mut files = Vec::new();
    for root in rule_roots(app) {
        if let Ok(mut loaded) = load_packs(&root) {
            files.append(&mut loaded);
        }
    }
    files
}

fn rule_roots(app: &SweepLoomApp) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd.join("rules"));
    }
    roots.push(app.locations.app_config.join("rules"));
    roots
}
