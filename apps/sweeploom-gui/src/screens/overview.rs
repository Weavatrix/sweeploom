//! Overview pressure cards and top opportunities.

use eframe::egui::{self, RichText};
use sweeploom_browser::BrowserPressure;
use sweeploom_core::{Recommendation, reclaim_set};

use super::session_label;
use super::session_pressure;
use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::icons::Glyph;
use crate::nav::Nav;
use crate::widgets::{self, Metric, PressureItem, Tone, list_row_at, page_title};

pub fn ui_overview(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(
        ui,
        "Overview",
        "Live pressure. Click a card or a session to open the matching screen.",
    );
    if let Some(nav) = widgets::metric_grid(ui, &overview_cards(app)) {
        app.nav = nav;
    }
    ui.add_space(4.0);
    draw_heaviest(app, ui);
    ui.add_space(8.0);
    widgets::section_heading(ui, "Top opportunities");
    draw_opportunities(app, ui);
}

fn draw_heaviest(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let hogs = session_pressure::heaviest(app);
    if hogs.is_empty() {
        return;
    }
    let caption = session_pressure::heaviest_caption(app, &hogs);
    widgets::section_heading(ui, "Heaviest sessions");
    ui.label(
        RichText::new(&caption)
            .size(13.0)
            .color(crate::theme::muted(ui)),
    );
    ui.add_space(8.0);
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

fn overview_cards(app: &SweepLoomApp) -> Vec<Metric> {
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
    let stale = app
        .sessions
        .iter()
        .filter(|session| {
            matches!(
                session.recommendation.recommendation,
                Recommendation::Recommended | Recommendation::StronglyRecommended
            )
        })
        .count();
    let reclaimable = app
        .sessions
        .iter()
        .filter(|session| session.recommendation.recommendation != Recommendation::Keep)
        .map(|session| session.recommendation.estimated_reclaimable_rss)
        .sum::<u64>();
    let stale_cpu: f32 = app
        .sessions
        .iter()
        .filter(|session| session.recommendation.recommendation != Recommendation::Keep)
        .map(|session| session.cpu_percent)
        .sum();
    let (disk, disk_ready) = if app.review.is_empty() {
        ("—".to_owned(), false)
    } else {
        let unique = reclaim_set(
            &app.review
                .iter()
                .map(|row| row.candidate.clone())
                .collect::<Vec<_>>(),
        );
        (
            crate::format::format_bytes_bound(
                unique.unique_estimate.bytes,
                unique.unique_estimate.complete,
            ),
            true,
        )
    };
    let mem_total = memory.total_bytes.max(1);
    let mem_avail = memory
        .available_bytes
        .max(memory.total_bytes.saturating_sub(memory.used_bytes));
    let mem_used = 1.0 - (mem_avail as f32 / mem_total as f32);
    let mut cards = vec![
        Metric {
            icon: Glyph::Memory,
            title: "MEMORY".into(),
            value: format_bytes(mem_avail),
            sub: format!("available of {}", format_bytes(memory.total_bytes)),
            open: Nav::Sessions,
            fill: Some(mem_used.clamp(0.0, 1.0)),
            tone: if mem_used >= 0.9 {
                Tone::Warn
            } else {
                Tone::Neutral
            },
        },
        Metric {
            icon: Glyph::Sessions,
            title: "IDLE SESSIONS".into(),
            value: if stale == 0 {
                "None".into()
            } else {
                format_bytes(reclaimable)
            },
            sub: if stale == 0 {
                "nothing idle yet".into()
            } else {
                format!("{stale} idle to review")
            },
            open: Nav::Sessions,
            fill: None,
            tone: if stale == 0 { Tone::Ok } else { Tone::Neutral },
        },
        Metric {
            icon: Glyph::Cpu,
            title: "CPU".into(),
            value: format!("{cpu:.0}%"),
            sub: format!("{stale_cpu:.0}% in idle sessions"),
            open: Nav::Sessions,
            fill: Some((cpu / 100.0).clamp(0.0, 1.0)),
            tone: if cpu >= 85.0 {
                Tone::Warn
            } else {
                Tone::Neutral
            },
        },
        Metric {
            icon: Glyph::Disk,
            title: "REVIEW DISK".into(),
            value: disk,
            sub: if disk_ready {
                "unique generated + temp".into()
            } else {
                "open Review to measure".into()
            },
            open: Nav::Storage,
            fill: None,
            tone: Tone::Neutral,
        },
    ];
    if let Some((mount, total, avail)) = app.volumes.first() {
        let used = 1.0 - (*avail as f32 / (*total).max(1) as f32);
        let tight = *avail < 8 * 1024 * 1024 * 1024;
        cards.push(Metric {
            icon: Glyph::Volume,
            title: "VOLUME".into(),
            value: format_bytes(*avail),
            sub: format!("free of {} on {}", format_bytes(*total), mount.display()),
            open: Nav::Explorer,
            fill: Some(used.clamp(0.0, 1.0)),
            tone: if tight { Tone::Warn } else { Tone::Neutral },
        });
    }
    cards
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
        ui.label("No idle sessions. Open Review for Cargo target / node_modules, or Browser for process trees.");
    }
}

fn processes_of(app: &SweepLoomApp) -> &[sweeploom_core::ProcessSnapshot] {
    app.snapshot
        .as_ref()
        .map(|item| item.processes.as_slice())
        .unwrap_or(&[])
}
