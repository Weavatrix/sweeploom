//! Inspect-first AI stores. Nothing here is auto-deleted.

use eframe::egui::{self, CornerRadius, Margin, RichText};
use sweeploom_ai::{AiOffer, inspect_offers};
use sweeploom_core::CandidateOwner;

use crate::app::SweepLoomApp;
use crate::format::{format_bytes, short_path};
use crate::icons::{self, Glyph};
use crate::widgets::{list_row, page_title, pointer};

pub fn ui_ai(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(
        ui,
        "AI",
        "Inspect-first. Internal Claude/Codex/Cursor DBs are never auto-selected.",
    );
    ui.label(
        RichText::new("Bounded listing only. File contents and sqlite internals are not opened.")
            .color(crate::theme::muted(ui)),
    );
    if pointer(ui.button("Refresh listing")).clicked() {
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
    ui.label(
        RichText::new(format!(
            "{} store(s). Review never pre-selects these.",
            offers.len()
        ))
        .color(crate::theme::muted(ui)),
    );
    ui.add_space(8.0);
    for offer in offers {
        draw_offer(ui, offer);
    }
}

fn draw_offer(ui: &mut egui::Ui, offer: &AiOffer) {
    let tool = match &offer.candidate.owner {
        CandidateOwner::Application(name) => name.as_str(),
        _ => "AI",
    };
    let cap = if offer.capped {
        "walk capped — listing is incomplete"
    } else {
        "inspect only · never selected"
    };
    egui::Frame::default()
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                icons::show(ui, Glyph::Ai, 16.0, crate::theme::accent());
                ui.label(RichText::new(tool).size(16.0).strong());
                ui.label(RichText::new(format_bytes(offer.candidate.logical_bytes)).strong());
                ui.label(
                    RichText::new(format!("{} files", offer.candidate.file_count))
                        .color(crate::theme::muted(ui)),
                );
            });
            ui.label(
                RichText::new(short_path(&offer.candidate.path))
                    .size(12.0)
                    .color(crate::theme::muted(ui)),
            );
            list_row(ui, cap, "", "");
            for sample in offer.samples.iter().take(6) {
                list_row(ui, sample, "", "sample path · contents not opened");
            }
        });
    ui.add_space(10.0);
}
