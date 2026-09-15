//! Browser OS process table. Helpers can be stopped; the browser process cannot.

use sweeploom_browser::{
    BrowserPressure, can_stop_helper, family_from_name, process_caption, process_role,
};
use sweeploom_core::{LiveSession, ProcessSnapshot, SessionId, SessionKind};

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::sort::{Col, Sort, header_cell};
use crate::widgets::pointer;
use eframe::egui::RichText;
use egui_extras::{Column, TableBuilder};

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
    let processes = app
        .snapshot
        .as_ref()
        .map(|item| item.processes.as_slice())
        .unwrap_or(&[]);
    let pressure = BrowserPressure::from_live(&app.sessions, processes);
    if pressure.hosts.is_empty() {
        ui.label("No browser process trees in this sample.");
        return;
    }
    ui.label(
        RichText::new(format!(
            "Combined RSS {} across {} family(ies). The Browser process stays Keep.",
            format_bytes(pressure.rss_bytes()),
            pressure.hosts.len()
        ))
        .strong(),
    );
    for host in &pressure.hosts {
        ui.label(format!(
            "{} · {} · {} session(s) · {} processes · {:.1}% CPU",
            host.family,
            format_bytes(host.rss_bytes),
            host.sessions,
            host.processes,
            host.cpu_percent
        ));
    }
    ui.add_space(6.0);
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
            let role = root
                .map(|item| process_role(&item.command))
                .unwrap_or("Browser");
            let caption = root
                .map(|item| process_caption(&item.command))
                .unwrap_or_else(|| "Browser".to_owned());
            TreeRow {
                id: session.id,
                family: root
                    .map(|item| family_from_name(&item.name))
                    .unwrap_or("Browser"),
                role: caption,
                pid: root.map(|item| item.pid).unwrap_or(0),
                procs: session.processes.len(),
                rss: session.rss_bytes,
                cpu: session.cpu_percent,
                status: session.activity.label().to_owned(),
                stoppable: can_stop_helper(role),
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
        Col::Status => left.status.cmp(&right.status),
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
    TableBuilder::new(ui)
        .id_salt("browser-trees-grid")
        .striped(true)
        .resizable(true)
        .sense(eframe::egui::Sense::click())
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .cell_layout(eframe::egui::Layout::left_to_right(
            eframe::egui::Align::Center,
        ))
        .column(Column::exact(36.0).clip(true).resizable(false))
        .column(Column::remainder().at_least(88.0).clip(true))
        .column(Column::exact(72.0).clip(true))
        .column(Column::exact(56.0).clip(true))
        .column(Column::exact(56.0).clip(true))
        .column(Column::exact(72.0).clip(true))
        .column(Column::exact(56.0).clip(true))
        .column(Column::exact(88.0).clip(true))
        .header(32.0, |mut header| {
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
        if ui
            .add_enabled(item.stoppable, eframe::egui::Checkbox::new(&mut on, ""))
            .changed()
        {
            if on {
                selected.insert(item.id);
            } else {
                selected.remove(&item.id);
            }
        }
    });
    row.col(|ui| {
        if pointer(
            ui.add(eframe::egui::Button::new(RichText::new(&item.role).size(16.0)).frame(false)),
        )
        .clicked()
        {
            *opened = Some(item.id);
        }
    });
    row.col(|ui| {
        ui.label(item.family);
    });
    row.col(|ui| {
        ui.label(item.pid.to_string());
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
        ui.label(&item.status);
    });
    if row.response().clicked() {
        *opened = Some(item.id);
    }
}
