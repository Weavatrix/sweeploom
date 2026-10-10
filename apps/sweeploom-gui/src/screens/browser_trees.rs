//! Browser OS process table. Helpers can be stopped; the browser process cannot.

use sweeploom_browser::{
    BrowserPressure, can_stop_helper, family_from_name, process_caption, process_role,
};
use sweeploom_core::{
    BrowserPart, LiveSession, ProcessSnapshot, SessionId, SessionKind, browser_identity,
};

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::sort::{Col, Sort, header_cell};
use crate::widgets::pointer;
use eframe::egui::RichText;
use egui_extras::Column;

pub(super) struct TreeRow {
    pub(super) id: SessionId,
    family: &'static str,
    role: String,
    pid: u32,
    procs: usize,
    rss: u64,
    cpu: f32,
    status: String,
    pub(super) stoppable: bool,
}

pub fn draw(app: &mut SweepLoomApp, ui: &mut eframe::egui::Ui) {
    let Some(snapshot) = app.snapshot.as_ref() else {
        ui.spinner();
        ui.label("Waiting for the first process sample…");
        return;
    };
    let processes = snapshot.processes.as_slice();
    let pressure = BrowserPressure::from_live(&app.sessions, processes);
    if pressure.hosts.is_empty() {
        ui.label("No supported browser executables or browser-specific services were found in the latest process sample.");
        return;
    }
    summary(ui, &pressure);
    let mut rows = collect_rows(&app.sessions, processes);
    let mut sort = app.browser.tree_sort;
    sort_rows(&mut rows, sort);
    draw_table(app, ui, &rows, &mut sort);
    app.browser.tree_sort = sort;
    super::browser_tree_detail::draw(app, ui, &rows);
}

fn collect_rows(sessions: &[LiveSession], processes: &[ProcessSnapshot]) -> Vec<TreeRow> {
    sessions
        .iter()
        .filter(|session| session.kind == SessionKind::Browser)
        .map(|session| {
            let root = session
                .processes
                .first()
                .and_then(|key| processes.iter().find(|process| process.key == *key));
            let identity = root.and_then(browser_identity);
            let role = root
                .map(|item| process_role(&item.command))
                .unwrap_or("Browser");
            let caption = if identity.is_some_and(|identity| identity.part == BrowserPart::Service)
            {
                format!(
                    "Background service · {}",
                    root.map(|item| item.name.as_str()).unwrap_or("Unknown")
                )
            } else if identity.is_some_and(|identity| identity.part == BrowserPart::Helper)
                && role == "Browser"
            {
                "Helper".to_owned()
            } else {
                root.map(|item| process_caption(&item.command))
                    .unwrap_or_else(|| "Browser".to_owned())
            };
            TreeRow {
                id: session.id,
                family: identity
                    .map(|identity| identity.family)
                    .or_else(|| root.map(|item| family_from_name(&item.name)))
                    .unwrap_or("Browser"),
                role: caption,
                pid: root.map(|item| item.pid).unwrap_or(0),
                procs: session.processes.len(),
                rss: session.rss_bytes,
                cpu: session.cpu_percent,
                status: session.activity.label().to_owned(),
                stoppable: can_stop_helper(role)
                    && identity.is_none_or(|identity| identity.part != BrowserPart::Service),
            }
        })
        .collect()
}

fn sort_rows(rows: &mut [TreeRow], sort: Sort) {
    rows.sort_by(|left, right| match sort.col {
        Col::Name => left
            .role
            .cmp(&right.role)
            .then(left.family.cmp(right.family)),
        Col::Procs => left.procs.cmp(&right.procs),
        Col::Cpu => left
            .cpu
            .partial_cmp(&right.cpu)
            .unwrap_or(std::cmp::Ordering::Equal),
        Col::Status | Col::Safety => left.status.cmp(&right.status),
        Col::Size => left.rss.cmp(&right.rss),
    });
    if sort.desc {
        rows.reverse();
    }
}

fn draw_table(
    app: &mut SweepLoomApp,
    ui: &mut eframe::egui::Ui,
    rows: &[TreeRow],
    sort: &mut Sort,
) {
    let reserve = if app.browser.selected_tree.is_some() {
        (ui.available_height() * 0.45).clamp(180.0, 380.0)
    } else {
        40.0
    };
    let height = (ui.available_height() - reserve).max(140.0);
    let count = rows.len();
    let mut selected = std::mem::take(&mut app.browser.tree_ids);
    let mut opened = app.browser.selected_tree;
    crate::widgets::table(ui, "browser-trees-grid")
        .striped(true)
        .resizable(true)
        .sense(eframe::egui::Sense::click())
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .cell_layout(eframe::egui::Layout::left_to_right(
            eframe::egui::Align::Center,
        ))
        .column(Column::exact(34.0).clip(true).resizable(false))
        .column(Column::remainder().at_least(160.0).clip(true))
        .column(Column::exact(96.0).clip(true))
        .column(Column::exact(72.0).clip(true))
        .column(Column::exact(64.0).clip(true))
        .column(Column::exact(88.0).clip(true))
        .column(Column::exact(64.0).clip(true))
        .column(Column::exact(120.0).clip(true))
        .header(30.0, |mut header| {
            header.col(|ui| {
                ui.strong("");
            });
            header.col(|ui| header_cell(ui, sort, Col::Name, "Role"));
            header.col(|ui| {
                ui.strong("Family");
            });
            header.col(|ui| {
                ui.strong("PID");
            });
            header.col(|ui| header_cell(ui, sort, Col::Procs, "Procs"));
            header.col(|ui| header_cell(ui, sort, Col::Size, "RSS"));
            header.col(|ui| header_cell(ui, sort, Col::Cpu, "CPU"));
            header.col(|ui| header_cell(ui, sort, Col::Status, "Status"));
        })
        .body(|body| {
            body.rows(crate::widgets::TABLE_ROW, count, |mut row| {
                let Some(item) = rows.get(row.index()) else {
                    return;
                };
                fill_row(&mut row, item, &mut selected, &mut opened);
            });
        });
    app.browser.tree_ids = selected;
    app.browser.selected_tree = opened;
}

fn fill_row(
    row: &mut egui_extras::TableRow<'_, '_>,
    item: &TreeRow,
    selected: &mut std::collections::HashSet<SessionId>,
    opened: &mut Option<SessionId>,
) {
    let mut on = selected.contains(&item.id);
    row.set_selected(*opened == Some(item.id));
    row.col(|ui| {
        if crate::widgets::check_enabled(ui, &mut on, item.stoppable).changed() {
            if on {
                selected.insert(item.id);
            } else {
                selected.remove(&item.id);
            }
        }
    });
    row.col(|ui| {
        if pointer(ui.add(
            eframe::egui::Label::new(RichText::new(&item.role))
                .truncate()
                .sense(eframe::egui::Sense::click()),
        ))
        .clicked()
        {
            *opened = Some(item.id);
        }
    });
    row.col(|ui| {
        ui.label(item.family);
    });
    row.col(|ui| {
        ui.label(RichText::new(item.pid.to_string()).color(crate::theme::muted(ui)));
    });
    row.col(|ui| {
        ui.label(item.procs.to_string());
    });
    row.col(|ui| {
        ui.label(format_bytes(item.rss));
    });
    row.col(|ui| {
        ui.label(format!("{:.1}%", item.cpu));
    });
    row.col(|ui| {
        crate::widgets::status_pill(ui, &item.status);
    });
    if row.response().clicked() {
        *opened = Some(item.id);
    }
}

/// Memory by browser family, with per-family notes.
fn summary(ui: &mut eframe::egui::Ui, pressure: &BrowserPressure) {
    let segments: Vec<crate::widgets::Segment> = pressure
        .hosts
        .iter()
        .enumerate()
        .map(|(index, host)| crate::widgets::Segment {
            label: host.family.to_string(),
            value: host.rss_bytes as f64,
            detail: format!(
                "{} · {} procs · {:.1}% CPU",
                format_bytes(host.rss_bytes),
                host.processes,
                host.cpu_percent
            ),
            color: crate::theme::series(ui, index),
        })
        .collect();
    crate::widgets::card(ui, |ui| {
        let total = format!(
            "{} across {} {}",
            format_bytes(pressure.rss_bytes()),
            pressure.hosts.len(),
            if pressure.hosts.len() == 1 { "family" } else { "families" }
        );
        crate::widgets::breakdown(ui, "Browser memory", &total, &segments);
        ui.add_space(crate::theme::SM);
        let mut notes = vec!["Main browser processes stay Keep.".to_owned()];
        for host in &pressure.hosts {
            if host.main_processes == 0 && host.background_services > 0 {
                notes.push(format!("{}: background services only; main browser not observed.", host.family));
            } else if host.main_processes == 0 {
                notes.push(format!("{}: helpers only; main browser not observed.", host.family));
            }
        }
        crate::widgets::caption(ui, notes.join(" "));
    });
    ui.add_space(crate::theme::MD);
}

#[cfg(test)]
#[path = "browser_trees_tests.rs"]
mod tests;
