//! About links, the MPL-2.0 source notice and the App Store edition's limits.

use eframe::egui::{self, RichText};

use crate::edition::{self, PRIVACY_URL, SITE_URL, SOURCE_URL, TERMS_URL};
use crate::{theme, widgets};

/// Website, terms, privacy and where the source code lives.
pub(super) fn links(ui: &mut egui::Ui) {
    ui.add_space(theme::MD);
    ui.horizontal_wrapped(|ui| {
        ui.hyperlink_to("sweeploom.com", SITE_URL);
        ui.label(RichText::new("·").color(theme::muted(ui)));
        ui.hyperlink_to("Terms", TERMS_URL);
        ui.label(RichText::new("·").color(theme::muted(ui)));
        ui.hyperlink_to("Privacy", PRIVACY_URL);
        ui.label(RichText::new("·").color(theme::muted(ui)));
        ui.hyperlink_to("Source code", SOURCE_URL);
    });
    ui.add_space(theme::SM);
    // MPL-2.0 §3.2: an executable distributed under other terms must say how to
    // obtain the Source Code Form.
    widgets::caption(
        ui,
        format!(
            "SweepLoom is open source under the Mozilla Public License 2.0. {} The complete source code of this version is available at {SOURCE_URL}.",
            if edition::STORE {
                "This App Store build is distributed under the App Store terms; the MPL-2.0 still covers its source."
            } else {
                "You may use, change and share it under that license."
            }
        ),
    );
}

/// App Store edition: what the sandbox leaves out, and where to get it.
pub(super) fn edition(ui: &mut egui::Ui) {
    if !edition::STORE {
        return;
    }
    widgets::section(
        ui,
        "Full open-source edition",
        "The Mac App Store requires the App Sandbox. These features need more access than the sandbox allows, so they are only in the free edition you can build from source:",
        |ui| {
            for (feature, why) in edition::FULL_ONLY {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(*feature).strong());
                    ui.label(RichText::new(format!("— {why}")).color(theme::muted(ui)));
                });
            }
            ui.add_space(theme::SM);
            ui.hyperlink_to("Get the open-source edition on GitHub", SOURCE_URL);
        },
    );
}
