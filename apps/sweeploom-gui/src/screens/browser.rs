//! Browser process trees plus companion tab heat when the host is fresh.

use std::time::{SystemTime, UNIX_EPOCH};

use eframe::egui::RichText;
use sweeploom_browser::{BrowserPressure, TabAction, TabCommand, load_snapshot, save_apply};
use sweeploom_core::SessionKind;

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::icons::{self, Glyph};
use crate::nav::Nav;
use crate::widgets::{list_row, list_row_at, page_title, pointer, section};

pub fn ui_browser(app: &mut SweepLoomApp, ui: &mut eframe::egui::Ui) {
    page_title(
        ui,
        "Browser",
        "Chrome/Edge/Firefox process trees. Tab lastAccessed needs the companion. npm lives on Projects and Review.",
    );
    let processes = app
        .snapshot
        .as_ref()
        .map(|item| item.processes.as_slice())
        .unwrap_or(&[]);
    let pressure = BrowserPressure::from_live(&app.sessions, processes);
    draw_hosts(app, ui, &pressure);
    draw_companion(app, ui);
}

fn draw_hosts(app: &mut SweepLoomApp, ui: &mut eframe::egui::Ui, pressure: &BrowserPressure) {
    let browser: Vec<_> = app
        .sessions
        .iter()
        .filter(|item| item.kind == SessionKind::Browser)
        .take(12)
        .map(|session| {
            (
                session.id,
                session.label().to_owned(),
                format_bytes(session.rss_bytes),
                format!(
                    "{:.1}% CPU · {} process(es) · {}",
                    session.cpu_percent,
                    session.processes.len(),
                    session.recommendation.recommendation.label()
                ),
            )
        })
        .collect();
    let mut open = None;
    section(
        ui,
        "Process trees",
        "Do not kill the whole browser to reclaim RAM. Discard stale tabs through the companion.",
        |ui| {
            if pressure.hosts.is_empty() {
                ui.label("No browser process trees in this sample.");
                return;
            }
            ui.label(
                RichText::new(format!(
                    "Combined RSS {} across {} family(ies)",
                    format_bytes(pressure.rss_bytes()),
                    pressure.hosts.len()
                ))
                .strong(),
            );
            ui.add_space(6.0);
            for host in &pressure.hosts {
                ui.horizontal(|ui| {
                    icons::show(ui, Glyph::Browser, 16.0, crate::theme::accent());
                    ui.label(RichText::new(host.family).size(16.0).strong());
                });
                list_row(
                    ui,
                    &format_bytes(host.rss_bytes),
                    &format!(
                        "{} session(s) · {} processes · {:.1}% CPU",
                        host.sessions, host.processes, host.cpu_percent
                    ),
                    "Keep the tree — terminate is never the browser action",
                );
            }
            ui.add_space(8.0);
            ui.label(RichText::new("Live sessions — click to open members").strong());
            for (id, label, rss, detail) in &browser {
                if list_row_at(ui, label, rss, detail).clicked() {
                    open = Some(*id);
                }
            }
        },
    );
    if let Some(id) = open {
        app.selected_session = Some(id);
        app.nav = Nav::Sessions;
    }
}

fn draw_companion(app: &mut SweepLoomApp, ui: &mut eframe::egui::Ui) {
    let now = unix_ms();
    let stored = load_snapshot(&app.locations.app_data).ok().flatten();
    let Some(stored) = stored.filter(|item| item.is_fresh(now)) else {
        app.confirm_browser_discard = false;
        section(
            ui,
            "Companion",
            "lastAccessed is not invented. Install the native host, then the extension.",
            |ui| {
                ui.label("1. sweeploom companion-install");
                ui.label("2. Host binary: sweeploom-companion-host");
                ui.label("3. Load the SweepLoom companion extension in Chrome or Edge");
                ui.label(
                    RichText::new(
                        "Until the companion is fresh, only process trees above are shown.",
                    )
                    .color(crate::theme::muted(ui)),
                );
            },
        );
        return;
    };
    let discard = stored.tabs.discard_count(now);
    section(
        ui,
        "Companion",
        "Connected. Discard is queued; Close is never sent.",
        |ui| {
            ui.label(
                RichText::new(format!(
                    "{} tabs · {} Discard suggestion(s)",
                    stored.tabs.tabs.len(),
                    discard
                ))
                .size(16.0)
                .strong(),
            );
            if let Some(message) = &app.action_message {
                ui.label(message);
            }
            draw_discard_confirm(app, ui, &stored, now, discard);
            for tab in stored.tabs.tabs.iter().take(16) {
                let heat = tab.heat(now, stored.tabs.active_tab_id);
                let action = tab.suggested_action(now, stored.tabs.active_tab_id);
                let mark = if action == TabAction::Discard {
                    "discard"
                } else {
                    "keep"
                };
                list_row(
                    ui,
                    &tab.title,
                    &format!("{} · {mark}", heat.label()),
                    &tab.url,
                );
            }
        },
    );
}

fn draw_discard_confirm(
    app: &mut SweepLoomApp,
    ui: &mut eframe::egui::Ui,
    stored: &sweeploom_browser::StoredCompanion,
    now: u64,
    discard: usize,
) {
    if discard == 0 {
        app.confirm_browser_discard = false;
        return;
    }
    if !app.confirm_browser_discard {
        if pointer(ui.button("Ask companion to Discard suggestions…")).clicked() {
            app.confirm_browser_discard = true;
        }
        return;
    }
    ui.label("Queue Discard only. Close is never sent. Bookmark+Close stays off by default.");
    ui.horizontal(|ui| {
        if pointer(ui.button("Cancel")).clicked() {
            app.confirm_browser_discard = false;
        }
        if pointer(ui.button("Queue Discard")).clicked() {
            queue_discard(app, stored, now);
        }
    });
}

fn queue_discard(app: &mut SweepLoomApp, stored: &sweeploom_browser::StoredCompanion, now: u64) {
    let actions: Vec<TabCommand> = stored
        .tabs
        .tabs
        .iter()
        .filter(|tab| tab.suggested_action(now, stored.tabs.active_tab_id) == TabAction::Discard)
        .map(|tab| TabCommand {
            tab_id: tab.tab_id,
            action: TabAction::Discard,
        })
        .collect();
    let n = actions.len();
    app.confirm_browser_discard = false;
    app.action_message = Some(match save_apply(&app.locations.app_data, actions) {
        Ok(()) => format!(
            "Queued {n} Discard action(s). The companion applies them on the next tabs ping."
        ),
        Err(error) => format!("Could not queue Discard: {error}"),
    });
}

fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|item| u64::try_from(item.as_millis()).unwrap_or(0))
        .unwrap_or(0)
}
