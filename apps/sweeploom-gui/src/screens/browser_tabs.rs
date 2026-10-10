//! Companion tab table: save to Later, discard, focus current.

use sweeploom_browser::{
    TabAction, TabCommand, TabSnapshot, add_later, load_later, load_snapshot, save_apply,
    save_later, snapshot_path,
};

use crate::app::SweepLoomApp;
use crate::sort::{Col, Sort, header_cell};
use crate::widgets::{pointer, table_scroll_height};
use eframe::egui::RichText;
use egui_extras::Column;

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
    let path = snapshot_path(&app.locations.app_data);
    let stored = match load_snapshot(&app.locations.app_data) {
        Ok(Some(stored)) => stored,
        Ok(None) => {
            app.browser.confirm_discard = false;
            super::browser_tabs_setup::draw_missing(app, ui, &path);
            return;
        }
        Err(error) => {
            app.browser.confirm_discard = false;
            super::browser_tabs_setup::draw_error(app, ui, &path, &error);
            return;
        }
    };
    if !stored.is_fresh(now) {
        let mins = now.saturating_sub(stored.written_unix_ms) / 60_000;
        ui.label(
            RichText::new(format!(
                "Companion last pinged {mins} min ago. Reload SweepLoom Companion if this stays stale."
            ))
            .color(crate::theme::muted(ui)),
        );
    }
    let discard = stored.tabs.discard_count(now);
    ui.label(
        RichText::new(format!(
            "{} tabs · {} Discard suggestion(s). Close is never sent.",
            stored.tabs.tabs.len(),
            discard
        ))
        .strong(),
    );
    if stored.tabs.tabs.is_empty() {
        ui.label("Companion is connected but sent 0 tabs. Keep the extension enabled.");
    }
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
        (&stored.instance_id, stored.epoch),
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
    route: (&str, u64),
) {
    let (instance_id, epoch) = route;
    ui.horizontal_wrapped(|ui| {
        if pointer(ui.button("Go to current tab")).clicked() {
            focus_current(app, active, instance_id, epoch);
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
                queue_discard(app, tabs, active, now, instance_id, epoch);
            }
        });
    }
}

fn draw_table(app: &mut SweepLoomApp, ui: &mut eframe::egui::Ui, rows: &[TabRow], sort: &mut Sort) {
    let height = table_scroll_height(ui);
    let count = rows.len();
    let mut selected = std::mem::take(&mut app.browser.tab_ids);
    crate::widgets::table(ui, "browser-tabs-grid")
        .striped(true)
        .resizable(true)
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .cell_layout(eframe::egui::Layout::left_to_right(
            eframe::egui::Align::Center,
        ))
        .column(Column::exact(36.0).clip(true).resizable(false))
        .column(Column::remainder().at_least(140.0).clip(true))
        .column(Column::exact(72.0).clip(true))
        .column(Column::exact(80.0).clip(true))
        .column(Column::remainder().at_least(120.0).clip(true))
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
            body.rows(crate::widgets::TABLE_ROW, count, |mut row| {
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
        if crate::widgets::check(ui, &mut on).changed() {
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

fn focus_current(app: &mut SweepLoomApp, active: Option<i64>, instance_id: &str, epoch: u64) {
    let Some(tab_id) = active else {
        app.action_message = Some("No current tab in the companion snapshot.".into());
        return;
    };
    queue(
        app,
        vec![TabCommand::new(
            tab_id,
            TabAction::Focus,
            instance_id,
            epoch,
        )],
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

fn queue_discard(
    app: &mut SweepLoomApp,
    tabs: &[TabSnapshot],
    active: Option<i64>,
    now: u64,
    instance_id: &str,
    epoch: u64,
) {
    let actions: Vec<TabCommand> = tabs
        .iter()
        .filter(|tab| tab.suggested_action(now, active) == TabAction::Discard)
        .map(|tab| {
            let mut command = TabCommand::new(tab.tab_id, TabAction::Discard, instance_id, epoch);
            command.expected_url = tab.url.clone();
            command
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
