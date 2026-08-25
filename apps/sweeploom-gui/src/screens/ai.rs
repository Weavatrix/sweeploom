//! Inspect-first AI stores. Nothing here is auto-deleted.

use eframe::egui::RichText;
use sweeploom_ai::discover_stores;

use crate::app::SweepLoomApp;
use crate::widgets::page_title;

pub fn ui_ai(app: &SweepLoomApp, ui: &mut eframe::egui::Ui) {
    page_title(
        ui,
        "AI",
        "Inspect-first. Internal Claude/Codex/Cursor DBs are never auto-selected.",
    );
    let stores = discover_stores(&app.locations);
    if stores.is_empty() {
        ui.label("No local AI session stores were found under the home directory.");
        return;
    }
    for store in stores {
        ui.label(RichText::new(format!("{}  {}", store.tool, store.path.display())).size(16.0));
        ui.label(
            RichText::new("REVIEW · inspect-only · search-before-delete is later")
                .size(14.0)
                .weak(),
        );
        ui.add_space(8.0);
    }
}
