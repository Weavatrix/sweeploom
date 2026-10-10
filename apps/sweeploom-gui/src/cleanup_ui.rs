use super::*;
use crate::format::format_bytes;
use crate::theme;
use crate::widgets::{self, Segment};
use eframe::egui;
#[path = "cleanup_table.rs"]
mod table;
#[path = "cleanup_toolbar.rs"]
mod toolbar;

pub(crate) fn ui(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    widgets::page_title(
        ui,
        "Cleanup",
        "Review storage by source. Choose individual objects, caches, models or archives. Each cleanup shows its exact targets before confirmation.",
    );
    #[cfg(debug_assertions)]
    if let Some(pane) = crate::chrome::shots::pane(ui.ctx()) {
        app.native_cleanup.pane = Pane::ALL[pane % Pane::ALL.len()];
    }
    app.poll_native();
    crate::disk_history::history_link(app, ui);
    ui.add_space(theme::SM);
    let processes = app
        .snapshot
        .as_ref()
        .map(|snapshot| snapshot.processes.clone());
    let tabs: Vec<(Pane, String)> = Pane::ALL
        .into_iter()
        .map(|pane| (pane, tab_label(&app.native_cleanup, pane)))
        .collect();
    let mut pane = app.native_cleanup.pane;
    if widgets::tabs(ui, &mut pane, &tabs) {
        app.native_cleanup.pane = pane;
    }
    summary(app, ui);
    let refresh = toolbar::draw(app, ui, processes.as_deref());
    widgets::action_note(ui, app.action_message.as_deref());
    if refresh || app.native_cleanup.active().needs_scan() {
        app.refresh_native();
    }
    if let Some(listing) = &app.native_cleanup.active().listing
        && !listing.note.is_empty()
    {
        widgets::caption(ui, listing.note.trim());
        ui.add_space(theme::SM);
    }
    table::draw(app, ui, processes.as_deref());
}

fn tab_label(cleanup: &NativeCleanup, pane: Pane) -> String {
    let total = cleanup.panes[&pane].listing.as_ref().map(|listing| {
        listing
            .items
            .iter()
            .filter_map(|item| item.bytes)
            .fold(0_u64, u64::saturating_add)
    });
    match total {
        Some(bytes) if bytes > 0 => format!("{} · {}", pane.label(), format_bytes(bytes)),
        _ => pane.label().to_owned(),
    }
}

/// Size breakdown card: by kind, or by largest items when one kind fills the pane.
fn summary(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let state = app.native_cleanup.active();
    let scanning = state.rx.is_some();
    let label = app.native_cleanup.pane.label();
    widgets::card(ui, |ui| {
        let Some(listing) = &state.listing else {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(format!("Discovering {label} storage and measuring sizes…"));
            });
            return;
        };
        let segments = segments(ui, &listing.items);
        let total: u64 = listing
            .items
            .iter()
            .filter_map(|item| item.bytes)
            .fold(0, u64::saturating_add);
        let measured = listing.items.iter().filter(|i| i.bytes.is_some()).count();
        let caption = format!(
            "{} across {} items · {measured} measured",
            format_bytes(total),
            listing.items.len()
        );
        widgets::breakdown(ui, &format!("{label} storage"), &caption, &segments);
        if scanning {
            ui.add_space(theme::SM);
            ui.horizontal(|ui| {
                ui.spinner();
                widgets::caption(ui, "Measuring sizes… totals grow as items finish.");
            });
        }
    });
    ui.add_space(theme::MD);
}

const KIND_ORDER: [&str; 12] = [
    "Runtime",
    "Device",
    "Sim files",
    "Unavailable",
    "Image",
    "Container",
    "Volume",
    "Build cache",
    "Model",
    "Cache",
    "User data",
    "Toolchain",
];

fn segments(ui: &egui::Ui, items: &[Item]) -> Vec<Segment> {
    let mut kinds: Vec<(&'static str, u64, usize)> = Vec::new();
    for item in items {
        let bytes = item.bytes.unwrap_or(0);
        match kinds
            .iter_mut()
            .find(|(kind, ..)| *kind == item.kind_label())
        {
            Some(entry) => {
                entry.1 = entry.1.saturating_add(bytes);
                entry.2 += 1;
            }
            None => kinds.push((item.kind_label(), bytes, 1)),
        }
    }
    if kinds.len() > 1 {
        kinds.sort_by_key(|(kind, ..)| KIND_ORDER.iter().position(|k| k == kind).unwrap_or(99));
        return kinds
            .iter()
            .enumerate()
            .map(|(index, (kind, bytes, count))| Segment {
                label: format!("{kind} · {count}"),
                value: *bytes as f64,
                detail: format_bytes(*bytes),
                color: kind_color(ui, kind, index),
            })
            .collect();
    }
    largest(ui, items)
}

fn largest(ui: &egui::Ui, items: &[Item]) -> Vec<Segment> {
    let mut sized: Vec<&Item> = items.iter().filter(|i| i.bytes.unwrap_or(0) > 0).collect();
    sized.sort_by_key(|item| std::cmp::Reverse(item.bytes));
    let mut out: Vec<Segment> = sized
        .iter()
        .take(5)
        .enumerate()
        .map(|(index, item)| Segment {
            label: crate::format::row_caption(&item.name),
            value: item.bytes.unwrap_or(0) as f64,
            detail: format_bytes(item.bytes.unwrap_or(0)),
            color: theme::series(ui, index),
        })
        .collect();
    let rest: u64 = sized.iter().skip(5).filter_map(|i| i.bytes).sum();
    if rest > 0 {
        out.push(Segment {
            label: format!("{} more", sized.len() - 5),
            value: rest as f64,
            detail: format_bytes(rest),
            color: theme::series_rest(ui),
        });
    }
    out
}

/// Stable color for a kind label, shared by the bar and the table.
pub(super) fn kind_color(ui: &egui::Ui, kind: &str, fallback: usize) -> egui::Color32 {
    if kind == "Inspect" {
        return theme::series_rest(ui);
    }
    // Each pane shows one family of four kinds, so slots repeat per family.
    let slot = KIND_ORDER
        .iter()
        .position(|k| *k == kind)
        .unwrap_or(fallback);
    theme::series(ui, slot % 4)
}
