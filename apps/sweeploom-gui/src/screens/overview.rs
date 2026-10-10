//! Overview: KPI tiles, CPU cores, memory/disk breakdowns and top opportunities.

use eframe::egui::{self, RichText};
use sweeploom_browser::BrowserPressure;
use sweeploom_core::{Recommendation, reclaim_set};

use super::session_label;
use super::session_pressure;
use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::icons::Glyph;
use crate::nav::Nav;
use crate::theme;
use crate::widgets::{self, Metric, PressureItem, StatTile, TileViz, Tone, cpu_cores, list_row_at};

#[path = "overview_board.rs"]
mod board;

pub fn ui_overview(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    widgets::page_title(
        ui,
        "Overview",
        "Live pressure across memory, CPU and disk. Click a tile or a session to open it.",
    );
    let cores = cpu_cores::history(ui.ctx());
    if let Some(nav) = widgets::metric_grid(ui, &overview_cards(app, &cores)) {
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

fn overview_cards(app: &SweepLoomApp, cores: &cpu_cores::CoreHistory) -> Vec<Metric> {
    let memory = app
        .snapshot
        .as_ref()
        .map(|item| item.memory)
        .unwrap_or_default();
    let cpu = app
        .snapshot
        .as_ref()
        .map(|item| item.cpu.usage_percent)
        .unwrap_or(0.0);
    let idle = app
        .sessions
        .iter()
        .filter(|session| session.recommendation.recommendation != Recommendation::Keep);
    let stale = idle
        .clone()
        .filter(|session| {
            matches!(
                session.recommendation.recommendation,
                Recommendation::Recommended | Recommendation::StronglyRecommended
            )
        })
        .count();
    let reclaimable = idle
        .clone()
        .map(|session| session.recommendation.estimated_reclaimable_rss)
        .sum::<u64>();
    let stale_cpu: f32 = idle
        .map(|session| session.cpu_percent.max(0.0))
        .sum::<f32>()
        .abs();
    let mem_total = memory.total_bytes.max(1);
    let mem_avail = memory
        .available_bytes
        .max(memory.total_bytes.saturating_sub(memory.used_bytes));
    let mem_used = 1.0 - (mem_avail as f32 / mem_total as f32);
    let tile = |icon, label: &str, value: String, sub: String, tone, viz| StatTile {
        icon: Some(icon),
        label: label.into(),
        value,
        sub,
        tone,
        viz,
    };
    let mut cards = vec![
        Metric {
            tile: tile(
                Glyph::Memory,
                "Memory available",
                format_bytes(mem_avail),
                format!(
                    "{:.0}% of {} in use",
                    mem_used * 100.0,
                    format_bytes(memory.total_bytes)
                ),
                if mem_used >= 0.9 {
                    Tone::Warn
                } else {
                    Tone::Neutral
                },
                TileViz::Meter(mem_used.clamp(0.0, 1.0)),
            ),
            open: Nav::Sessions,
        },
        Metric {
            tile: tile(
                Glyph::Cpu,
                "CPU load",
                format!("{cpu:.0}%"),
                format!("{} · {stale_cpu:.0}% in idle sessions", cores.summary()),
                if cpu >= 85.0 {
                    Tone::Warn
                } else {
                    Tone::Neutral
                },
                TileViz::Trend(cores.total.iter().copied().collect(), Some(100.0)),
            ),
            open: Nav::Sessions,
        },
        Metric {
            tile: tile(
                Glyph::Sessions,
                "Idle sessions",
                if stale == 0 {
                    "None".into()
                } else {
                    format_bytes(reclaimable)
                },
                if stale == 0 {
                    "nothing idle yet".into()
                } else {
                    format!("{stale} idle to review")
                },
                if stale == 0 { Tone::Ok } else { Tone::Caution },
                TileViz::None,
            ),
            open: Nav::Sessions,
        },
    ];
    cards.push(disk_card(app));
    cards
}

fn disk_card(app: &SweepLoomApp) -> Metric {
    let review = review_bytes(app);
    let sub = review.map_or_else(
        || "open Review to measure".to_owned(),
        |(bytes, complete)| {
            if bytes == 0 {
                return "nothing reclaimable in Review".to_owned();
            }
            format!(
                "{} reclaimable in Review",
                crate::format::format_bytes_bound(bytes, complete)
            )
        },
    );
    let (value, fill, tone) = match app.volumes.first() {
        Some((_, total, avail)) => (
            format_bytes(*avail),
            1.0 - (*avail as f32 / (*total).max(1) as f32),
            if *avail < 8 * 1024 * 1024 * 1024 {
                Tone::Warn
            } else {
                Tone::Neutral
            },
        ),
        None => ("—".to_owned(), 0.0, Tone::Neutral),
    };
    Metric {
        tile: StatTile {
            icon: Some(Glyph::Volume),
            label: "Disk free".into(),
            value,
            sub,
            tone,
            viz: TileViz::Meter(fill.clamp(0.0, 1.0)),
        },
        open: Nav::Storage,
    }
}

/// Unique Review estimate, if Review has rows.
pub(super) fn review_bytes(app: &SweepLoomApp) -> Option<(u64, bool)> {
    if app.review.is_empty() {
        return None;
    }
    let unique = reclaim_set(
        &app.review
            .iter()
            .map(|row| row.candidate.clone())
            .collect::<Vec<_>>(),
    );
    Some((
        unique.unique_estimate.bytes,
        unique.unique_estimate.complete,
    ))
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
