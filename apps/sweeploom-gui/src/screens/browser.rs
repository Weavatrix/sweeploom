//! Browser process trees, companion tabs, and the Later shelf.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::SweepLoomApp;
use crate::widgets::{page_title, pointer};

use super::browser_later;
use super::browser_state::BrowserPane;
use super::browser_tabs;
use super::browser_trees;

pub fn ui_browser(app: &mut SweepLoomApp, ui: &mut eframe::egui::Ui) {
    page_title(
        ui,
        "Browser",
        "Renderer helpers can be stopped without ending Edge. Tabs and Later need URLs from the companion. Close is never sent.",
    );
    ui.horizontal(|ui| {
        pane_button(ui, app, BrowserPane::Trees, "Process trees");
        pane_button(ui, app, BrowserPane::Tabs, "Tabs");
        pane_button(ui, app, BrowserPane::Later, "Later");
    });
    ui.add_space(8.0);
    match app.browser.pane {
        BrowserPane::Trees => browser_trees::draw(app, ui),
        BrowserPane::Tabs => browser_tabs::draw(app, ui),
        BrowserPane::Later => browser_later::draw(app, ui),
    }
}

fn pane_button(ui: &mut eframe::egui::Ui, app: &mut SweepLoomApp, pane: BrowserPane, label: &str) {
    if pointer(ui.selectable_label(app.browser.pane == pane, label)).clicked() {
        app.browser.pane = pane;
    }
}

pub(super) fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|item| u64::try_from(item.as_millis()).unwrap_or(0))
        .unwrap_or(0)
}
