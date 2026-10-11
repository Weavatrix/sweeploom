//! Browser process trees, companion tabs, and the Later shelf.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::SweepLoomApp;
use crate::widgets::{self, page_title};

use super::browser_later;
use super::browser_state::BrowserPane;
use super::browser_tabs;
use super::browser_trees;

pub fn ui_browser(app: &mut SweepLoomApp, ui: &mut eframe::egui::Ui) {
    page_title(
        ui,
        "Browser",
        "Live browser processes, memory and CPU. Open a tree to inspect its helpers. Tabs and Later need the companion.",
    );
    let mut pane = app.browser.pane;
    if widgets::tabs(
        ui,
        &mut pane,
        &[
            (BrowserPane::Trees, "Process trees".to_owned()),
            (BrowserPane::Tabs, "Tabs".to_owned()),
            (BrowserPane::Later, "Later".to_owned()),
        ],
    ) {
        app.browser.pane = pane;
    }
    match app.browser.pane {
        BrowserPane::Trees => browser_trees::draw(app, ui),
        BrowserPane::Tabs => browser_tabs::draw(app, ui),
        BrowserPane::Later => browser_later::draw(app, ui),
    }
}

pub(super) fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|item| u64::try_from(item.as_millis()).unwrap_or(0))
        .unwrap_or(0)
}
