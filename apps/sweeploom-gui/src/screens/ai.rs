//! Inspect-first AI stores. Nothing here is auto-deleted.

use eframe::egui::RichText;
use sweeploom_ai::{AiOffer, inspect_offers};

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::widgets::{list_row, page_title};

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
    let cap = if offer.capped {
        "walk capped"
    } else {
        "inspect-only"
    };
    let samples = if offer.samples.is_empty() {
        String::new()
    } else {
        format!("samples: {}", offer.samples.join(", "))
    };
    list_row(
        ui,
        &offer.title,
        &format!(
            "{} · {} files · {cap} · not selected",
            format_bytes(offer.candidate.logical_bytes),
            offer.candidate.file_count
        ),
        &samples,
    );
}
