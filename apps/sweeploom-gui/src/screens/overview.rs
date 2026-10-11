//! Overview: KPI tiles, CPU cores, memory/disk breakdowns and top opportunities.

use eframe::egui::{self, RichText};
use sweeploom_browser::BrowserPressure;
use sweeploom_core::Recommendation;

use super::session_label;
use super::session_pressure;
use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::nav::Nav;
use crate::theme;
use crate::widgets::{self, PressureItem, cpu_cores, list_row_at};

#[path = "overview_board.rs"]
mod board;
#[path = "overview_tiles.rs"]
mod tiles;

pub fn ui_overview(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    widgets::page_title(
        ui,
        "Overview",
        "Live pressure across memory, CPU and disk. Click a tile or a session to open it.",
    );
    let cores = cpu_cores::history(ui.ctx());
    if let Some(nav) = widgets::metric_grid(ui, &tiles::overview_cards(app, &cores)) {
        app.nav = nav;
    }
    widgets::card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("CPU cores").size(15.5).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                widgets::caption(
                    ui,
                    format!("{} · last {} s", cores.summary(), cores_window(&cores)),
                );
            });
        });
        ui.add_space(theme::MD);
        cpu_cores::grid(ui, &cores);
    });
    ui.add_space(theme::MD);
    board::breakdowns(app, ui);
    draw_heaviest(app, ui);
    widgets::section_heading(ui, "Top opportunities");
    draw_opportunities(app, ui);
}

fn cores_window(cores: &cpu_cores::CoreHistory) -> usize {
    cores.total.len()
}

fn draw_heaviest(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let hogs = session_pressure::heaviest(app);
    if hogs.is_empty() {
        return;
    }
    ui.add_space(theme::SM);
    ui.horizontal(|ui| {
        ui.label(RichText::new("Heaviest sessions").size(15.5).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            widgets::caption(ui, session_pressure::heaviest_caption(app, &hogs));
        });
    });
    ui.add_space(theme::SM);
    let items: Vec<PressureItem> = hogs
        .iter()
        .map(|hog| PressureItem {
            title: hog.title.clone(),
            rss: format_bytes(hog.rss),
            cpu: hog.cpu,
            ram_share: hog.share,
        })
        .collect();
    if let Some(index) = widgets::pressure_grid(ui, &items)
        && let Some(hog) = hogs.get(index)
    {
        app.selected_session = Some(hog.id);
        app.nav = Nav::Sessions;
    }
}

fn draw_opportunities(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let mut shown = 0_usize;
    let sessions: Vec<_> = {
        let processes = processes_of(app);
        app.sessions
            .iter()
            .filter(|session| session.recommendation.recommendation != Recommendation::Keep)
            .take(6)
            .map(|session| {
                (
                    session.id,
                    session_label::title(session, processes),
                    format_bytes(session.rss_bytes),
                    session.recommendation.recommendation.label().to_owned(),
                )
            })
            .collect()
    };
    for (id, label, rss, rec) in sessions {
        shown += 1;
        if list_row_at(ui, &label, &rss, &rec).clicked() {
            app.selected_session = Some(id);
            app.nav = Nav::Sessions;
        }
    }
    let pressure = {
        let processes = processes_of(app);
        BrowserPressure::from_live(&app.sessions, processes)
    };
    if pressure.rss_bytes() > 0 && shown < 8 {
        shown += 1;
        if list_row_at(
            ui,
            "Browser",
            &format_bytes(pressure.rss_bytes()),
            "open Browser — companion needed for tab discard",
        )
        .clicked()
        {
            app.nav = Nav::Browser;
        }
    }
    let review: Vec<_> = app
        .review
        .iter()
        .take(4)
        .map(|row| {
            (
                crate::format::row_caption(&row.title),
                format_bytes(row.candidate.logical_bytes),
            )
        })
        .collect();
    for (name, size) in review {
        shown += 1;
        if list_row_at(ui, &name, &size, "open Review").clicked() {
            app.nav = Nav::Storage;
        }
    }
    if shown == 0 {
        widgets::caption(
            ui,
            "No idle sessions. Open Review for Cargo target / node_modules, or Browser for process trees.",
        );
    }
}

pub(super) fn processes_of(app: &SweepLoomApp) -> &[sweeploom_core::ProcessSnapshot] {
    app.snapshot
        .as_ref()
        .map(|item| item.processes.as_slice())
        .unwrap_or(&[])
}
