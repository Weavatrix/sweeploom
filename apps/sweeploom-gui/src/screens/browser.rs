//! Browser process trees plus companion tab heat when the host is fresh.

use std::time::{SystemTime, UNIX_EPOCH};

use eframe::egui::RichText;
use sweeploom_browser::{BrowserPressure, TabAction, load_snapshot};

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::widgets::page_title;

pub fn ui_browser(app: &SweepLoomApp, ui: &mut eframe::egui::Ui) {
    page_title(
        ui,
        "Browser",
        "Process trees are always visible. Tab lastAccessed appears only while the companion is fresh.",
    );
    let processes = app
        .snapshot
        .as_ref()
        .map(|item| item.processes.as_slice())
        .unwrap_or(&[]);
    let pressure = BrowserPressure::from_live(&app.sessions, processes);
    draw_hosts(ui, &pressure);
    draw_companion(app, ui);
}

fn draw_hosts(ui: &mut eframe::egui::Ui, pressure: &BrowserPressure) {
    if pressure.hosts.is_empty() {
        ui.label("No browser process trees in this sample.");
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
            RichText::new("Do not kill the whole browser to reclaim RAM.")
                .size(14.0)
                .weak(),
        );
        ui.add_space(8.0);
    }
}

fn draw_companion(app: &SweepLoomApp, ui: &mut eframe::egui::Ui) {
    let now = unix_ms();
    let stored = load_snapshot(&app.locations.app_data).ok().flatten();
    let Some(stored) = stored.filter(|item| item.is_fresh(now)) else {
        ui.label(
            RichText::new(
                "Companion: not connected. lastAccessed is not shown as zero. Host: sweeploom companion-host",
            )
            .size(14.0)
            .weak(),
        );
        return;
    };
    let discard = stored.tabs.discard_count(now);
    ui.label(
        RichText::new(format!(
            "Companion: connected  {} tabs  {} Discard suggestions",
            stored.tabs.tabs.len(),
            discard
        ))
        .size(16.0),
    );
    for tab in stored.tabs.tabs.iter().take(12) {
        let heat = tab.heat(now, stored.tabs.active_tab_id);
        let action = tab.suggested_action(now, stored.tabs.active_tab_id);
        let mark = if action == TabAction::Discard {
            "[x]"
        } else {
            "[ ]"
        };
        ui.label(format!("{mark} {:?}  {}  {}", heat, tab.title, tab.url));
    }
}

fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|item| u64::try_from(item.as_millis()).unwrap_or(0))
        .unwrap_or(0)
}
