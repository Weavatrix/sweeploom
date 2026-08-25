//! Inspect-first AI stores. Nothing here is auto-deleted.

use eframe::egui::RichText;
use sweeploom_ai::{AiOffer, inspect_offers};

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::widgets::page_title;

pub fn ui_ai(app: &mut SweepLoomApp, ui: &mut eframe::egui::Ui) {
    page_title(
        ui,
        "AI",
        "Inspect-first. Internal Claude/Codex/Cursor DBs are never auto-selected.",
    );
    ui.label(
        RichText::new("Bounded listing only. File contents and sqlite internals are not opened.")
            .weak(),
    );
    if ui.button("Refresh listing").clicked() {
        app.ai_offers = None;
    }
    ui.add_space(8.0);
    if app.ai_offers.is_none() {
        app.ai_offers = Some(inspect_offers(&app.locations));
    }
    let Some(offers) = &app.ai_offers else {
        return;
    };
    if offers.is_empty() {
        ui.label("No local AI session stores were found under the home directory.");
        return;
    }
    for offer in offers {
        draw_offer(ui, offer);
    }
}

fn draw_offer(ui: &mut eframe::egui::Ui, offer: &AiOffer) {
    ui.label(RichText::new(&offer.title).size(16.0));
    let cap = if offer.capped { " · walk capped" } else { "" };
    ui.label(
        RichText::new(format!(
            "{} · {} files{cap} · inspect-only · not selected",
            format_bytes(offer.candidate.logical_bytes),
            offer.candidate.file_count
        ))
        .size(14.0)
        .weak(),
    );
    if !offer.samples.is_empty() {
        ui.label(
            RichText::new(format!("samples: {}", offer.samples.join(", ")))
                .size(13.0)
                .weak(),
        );
    }
    ui.add_space(8.0);
}
