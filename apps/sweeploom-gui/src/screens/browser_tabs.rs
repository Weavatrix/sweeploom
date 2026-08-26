//! Companion tab table: save to Later, discard, focus current.

use sweeploom_browser::{
    TabAction, TabCommand, TabSnapshot, add_later, load_later, load_snapshot, save_apply,
    save_later,
};

use crate::app::SweepLoomApp;
use crate::sort::{Col, Sort, header_cell};
use crate::widgets::{pointer, table_scroll_height};
use eframe::egui::RichText;
use egui_extras::{Column, TableBuilder};

use super::browser::unix_ms;

struct TabRow {
    id: i64,
    title: String,
    url: String,
    heat: String,
    mark: String,
}

pub fn draw(app: &mut SweepLoomApp, ui: &mut eframe::egui::Ui) {
    let now = unix_ms();
    let stored = load_snapshot(&app.locations.app_data).ok().flatten();
    let Some(stored) = stored.filter(|item| item.is_fresh(now)) else {
        app.browser.confirm_discard = false;
        ui.label("Companion is not fresh. Install the native host, then the extension.");
        ui.label("1. sweeploom companion-install");
        ui.label("2. Host binary: sweeploom-companion-host");
        ui.label("3. Load the SweepLoom companion extension in Chrome or Edge");
        ui.label("Until then, Process trees still work. Later can reopen saved URLs.");
        return;
    };
    let discard = stored.tabs.discard_count(now);
    ui.label(
        RichText::new(format!(
            "{} tabs · {} Discard suggestion(s). Close is never sent.",
            stored.tabs.tabs.len(),
            discard
        ))
        .strong(),
    );
    if let Some(message) = &app.action_message {
        ui.label(message.clone());
    }
    draw_actions(
        app,
        ui,
        &stored.tabs.tabs,
        stored.tabs.active_tab_id,
        now,
        discard,
    );
    let mut rows = collect_rows(&stored.tabs.tabs, stored.tabs.active_tab_id, now);
    let mut sort = app.browser.tab_sort;
    sort_rows(&mut rows, sort);
    draw_table(app, ui, &rows, &mut sort);
    app.browser.tab_sort = sort;
}

fn collect_rows(tabs: &[TabSnapshot], active: Option<i64>, now: u64) -> Vec<TabRow> {
    tabs.iter()
        .map(|tab| {
            let heat = tab.heat(now, active);
            let action = tab.suggested_action(now, active);
            TabRow {
                id: tab.tab_id,
                title: if tab.title.is_empty() {
                    tab.url.clone()
                } else {
                    tab.title.clone()
                },
                url: tab.url.clone(),
                heat: heat.label().to_owned(),
                mark: if action == TabAction::Discard {
                    "discard".into()
                } else {
                    "keep".into()
                },
            }
        })
        .collect()
}

fn sort_rows(rows: &mut [TabRow], sort: Sort) {
    rows.sort_by(|left, right| match sort.col {
        Col::Name => left.title.cmp(&right.title),
        Col::Status => left.heat.cmp(&right.heat),
        _ => left.url.cmp(&right.url),
    });
    if sort.desc {
        rows.reverse();
    }
}

fn draw_actions(
    app: &mut SweepLoomApp,
    ui: &mut eframe::egui::Ui,
    tabs: &[TabSnapshot],
    active: Option<i64>,
    now: u64,
    discard: usize,
) {
    ui.horizontal_wrapped(|ui| {
        if pointer(ui.button("Go to current tab")).clicked() {
            focus_current(app, active);
        }
        if pointer(ui.button("Save selected to Later")).clicked() {
            save_selected(app, tabs, now);
        }
        if discard > 0
            && !app.browser.confirm_discard
            && pointer(ui.button("Discard suggestions…")).clicked()
        {
            app.browser.confirm_discard = true;
        }
    });
    if app.browser.confirm_discard {
        ui.label("Queue Discard only. Tabs stay in the strip. Bookmark+Close stays off.");
        ui.horizontal(|ui| {
            if pointer(ui.button("Cancel")).clicked() {
                app.browser.confirm_discard = false;
            }
            if pointer(ui.button("Queue Discard")).clicked() {
                queue_discard(app, tabs, active, now);
            }
        });
    }
}

fn draw_table(app: &mut SweepLoomApp, ui: &mut eframe::egui::Ui, rows: &[TabRow], sort: &mut Sort) {
    let height = table_scroll_height(ui);
    let count = rows.len();
    let mut selected = std::mem::take(&mut app.browser.tab_ids);
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .cell_layout(eframe::egui::Layout::left_to_right(
            eframe::egui::Align::Center,
        ))
        .column(Column::auto().at_least(36.0))
        .column(Column::remainder().at_least(160.0))
        .column(Column::auto().at_least(80.0))
        .column(Column::auto().at_least(80.0))
        .column(Column::remainder().at_least(160.0))
        .header(32.0, |mut header| {
            header.col(|ui| {
                ui.strong("");
            });
            header.col(|ui| header_cell(ui, sort, Col::Name, "Title"));
            header.col(|ui| header_cell(ui, sort, Col::Status, "Heat"));
            header.col(|ui| {
                ui.strong("Policy");
            });
            header.col(|ui| {
                ui.strong("URL");
            });
        })
        .body(|body| {
            body.rows(28.0, count, |mut row| {
                let Some(item) = rows.get(row.index()) else {
                    return;
                };
                fill_row(&mut row, item, &mut selected);
            });
        });
    app.browser.tab_ids = selected;
}

fn fill_row(
    row: &mut egui_extras::TableRow<'_, '_>,
    item: &TabRow,
    selected: &mut std::collections::HashSet<i64>,
) {
    let mut on = selected.contains(&item.id);
    row.col(|ui| {
        if ui.checkbox(&mut on, "").changed() {
            if on {
                selected.insert(item.id);
            } else {
                selected.remove(&item.id);
            }
        }
    });
    row.col(|ui| {
        ui.label(&item.title);
    });
    row.col(|ui| {
        ui.label(&item.heat);
    });
    row.col(|ui| {
        ui.label(&item.mark);
    });
    row.col(|ui| {
        ui.monospace(&item.url);
    });
}

fn focus_current(app: &mut SweepLoomApp, active: Option<i64>) {
    let Some(tab_id) = active else {
        app.action_message = Some("No current tab in the companion snapshot.".into());
        return;
    };
    queue(
        app,
        vec![TabCommand {
            tab_id,
            action: TabAction::Focus,
        }],
    );
}

fn save_selected(app: &mut SweepLoomApp, tabs: &[TabSnapshot], now: u64) {
    let ids = &app.browser.tab_ids;
    let picked: Vec<&TabSnapshot> = tabs
        .iter()
        .filter(|tab| ids.contains(&tab.tab_id))
        .collect();
    if picked.is_empty() {
        app.action_message = Some("Check tabs to save. Saving does not close them.".into());
        return;
    }
    let mut shelf = load_later(&app.locations.app_data).unwrap_or_default();
    let mut added = 0_usize;
    for tab in picked {
        if add_later(&mut shelf, &tab.title, &tab.url, now) {
            added += 1;
        }
    }
    app.action_message = Some(match save_later(&app.locations.app_data, &shelf) {
        Ok(()) => format!("Saved {added} tab(s) to Later. They stay open."),
        Err(error) => format!("Could not save Later: {error}"),
    });
}

fn queue_discard(app: &mut SweepLoomApp, tabs: &[TabSnapshot], active: Option<i64>, now: u64) {
    let actions: Vec<TabCommand> = tabs
        .iter()
        .filter(|tab| tab.suggested_action(now, active) == TabAction::Discard)
        .map(|tab| TabCommand {
            tab_id: tab.tab_id,
            action: TabAction::Discard,
        })
        .collect();
    app.browser.confirm_discard = false;
    queue(app, actions);
}

fn queue(app: &mut SweepLoomApp, actions: Vec<TabCommand>) {
    let n = actions.len();
    app.action_message = Some(match save_apply(&app.locations.app_data, actions) {
        Ok(()) => {
            format!("Queued {n} action(s). The companion applies them on the next tabs ping.")
        }
        Err(error) => format!("Could not queue companion action: {error}"),
    });
}
