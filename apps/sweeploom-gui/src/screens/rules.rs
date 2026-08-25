//! Declarative TOML cleaners. Packs are data; nothing here is executed.

use std::path::PathBuf;

use sweeploom_rules::load_packs;

use crate::app::SweepLoomApp;
use crate::widgets::{list_row, page_title};

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
        match file.pack {
            Ok(pack) => {
                list_row(
                    ui,
                    &file.path.display().to_string(),
                    &format!("schema {} · {} cleaner(s)", pack.schema, pack.cleaner.len()),
                    "",
                );
                for rule in pack.cleaner {
                    let label = rule.label.as_deref().unwrap_or(&rule.id);
                    list_row(
                        ui,
                        &rule.id,
                        label,
                        &format!("{:?} · {:?}", rule.safety_level(), rule.deletion_strategy()),
                    );
                }
            }
            Err(error) => {
                ui.colored_label(ui.visuals().warn_fg_color, format!("not loaded: {error}"));
            }
        }
        ui.add_space(10.0);
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
