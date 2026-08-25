//! Browser process trees. Tab lastAccessed needs the companion.

use eframe::egui::RichText;
use sweeploom_browser::BrowserPressure;

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::widgets::page_title;

pub fn ui_browser(app: &SweepLoomApp, ui: &mut eframe::egui::Ui) {
    page_title(
        ui,
        "Browser",
        "Process trees and RAM are visible now. Tab lastAccessed / Discard need the companion.",
    );
    let processes = app
        .snapshot
        .as_ref()
        .map(|item| item.processes.as_slice())
        .unwrap_or(&[]);
    let pressure = BrowserPressure::from_live(&app.sessions, processes);
    if pressure.hosts.is_empty() {
        ui.label("No browser process trees in this sample.");
        ui.label(
            RichText::new("Cold-tab counts stay unknown until the companion connects.")
                .size(14.0)
                .weak(),
        );
        return;
    }
    for host in &pressure.hosts {
        ui.label(
            RichText::new(format!(
                "{}  {}  {} processes  {:.1}% CPU",
                host.family,
                format_bytes(host.rss_bytes),
                host.processes,
                host.cpu_percent
            ))
            .size(16.0),
        );
        ui.label(
            RichText::new("Do not kill the whole browser to reclaim RAM. Discard cold tabs later.")
                .size(14.0)
                .weak(),
        );
        ui.add_space(8.0);
    }
    ui.label(
        RichText::new("Companion: not connected. lastAccessed is not shown as zero.")
            .size(14.0)
            .weak(),
    );
}
