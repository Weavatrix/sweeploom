//! Saved tab URLs. Reopen without closing anything in the live browser.

use sweeploom_browser::{LaterEntry, load_later, open_http_urls, save_later};

use crate::app::SweepLoomApp;
use crate::widgets::{pointer, table_scroll_height};
use eframe::egui::RichText;
use egui_extras::{Column, TableBuilder};

pub fn draw(app: &mut SweepLoomApp, ui: &mut eframe::egui::Ui) {
    let mut shelf = load_later(&app.locations.app_data).unwrap_or_default();
    ui.label(
        RichText::new(
            "Later is a local URL shelf. Saving does not close tabs. Reopen uses the default browser.",
        )
        .color(crate::theme::muted(ui)),
    );
    if let Some(message) = &app.action_message {
        ui.label(message.clone());
    }
    if shelf.entries.is_empty() {
        ui.label("Nothing saved yet. On Tabs, check rows and Save selected to Later.");
        return;
    }
    ui.horizontal_wrapped(|ui| {
        if pointer(ui.button("Reopen selected")).clicked() {
            reopen(app, &shelf.entries, false);
        }
        if pointer(ui.button("Reopen all")).clicked() {
            reopen(app, &shelf.entries, true);
        }
        if pointer(ui.button("Remove selected")).clicked() {
            remove_selected(app, &mut shelf);
        }
    });
    draw_table(app, ui, &shelf.entries);
}

fn draw_table(app: &mut SweepLoomApp, ui: &mut eframe::egui::Ui, entries: &[LaterEntry]) {
    let height = table_scroll_height(ui);
    let count = entries.len();
    let mut selected = std::mem::take(&mut app.browser.later_urls);
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .cell_layout(eframe::egui::Layout::left_to_right(
            eframe::egui::Align::Center,
        ))
        .column(Column::auto().at_least(36.0))
        .column(Column::remainder().at_least(180.0))
        .column(Column::remainder().at_least(180.0))
        .header(32.0, |mut header| {
            header.col(|ui| {
                ui.strong("");
            });
            header.col(|ui| {
                ui.strong("Title");
            });
            header.col(|ui| {
                ui.strong("URL");
            });
        })
        .body(|body| {
            body.rows(28.0, count, |mut row| {
                let Some(item) = entries.get(row.index()) else {
                    return;
                };
                let mut on = selected.contains(&item.url);
                row.col(|ui| {
                    if ui.checkbox(&mut on, "").changed() {
                        if on {
                            selected.insert(item.url.clone());
                        } else {
                            selected.remove(&item.url);
                        }
                    }
                });
                row.col(|ui| {
                    ui.label(&item.title);
                });
                row.col(|ui| {
                    ui.monospace(&item.url);
                });
            });
        });
    app.browser.later_urls = selected;
}

fn reopen(app: &mut SweepLoomApp, entries: &[LaterEntry], all: bool) {
    let urls: Vec<String> = entries
        .iter()
        .filter(|item| all || app.browser.later_urls.contains(&item.url))
        .map(|item| item.url.clone())
        .collect();
    if urls.is_empty() {
        app.action_message = Some("Check rows to reopen, or use Reopen all.".into());
        return;
    }
    app.action_message = Some(match open_http_urls(&urls) {
        Ok(n) => format!("Opened {n} URL(s) in the default browser."),
        Err(error) => format!("Could not open: {error}"),
    });
}

fn remove_selected(app: &mut SweepLoomApp, shelf: &mut sweeploom_browser::LaterShelf) {
    let before = shelf.entries.len();
    shelf
        .entries
        .retain(|item| !app.browser.later_urls.contains(&item.url));
    let removed = before.saturating_sub(shelf.entries.len());
    app.browser.later_urls.clear();
    app.action_message = Some(match save_later(&app.locations.app_data, shelf) {
        Ok(()) => format!("Removed {removed} saved tab(s)."),
        Err(error) => format!("Could not update Later: {error}"),
    });
}
