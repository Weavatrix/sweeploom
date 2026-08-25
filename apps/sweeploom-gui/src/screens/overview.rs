//! Overview pressure cards and top opportunities.

use eframe::egui::{self, RichText};
use sweeploom_browser::BrowserPressure;
use sweeploom_core::Recommendation;

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::nav::Nav;
use crate::widgets::{list_row_at, metric_card, page_title};

pub fn ui_overview(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    page_title(
        ui,
        "Overview",
        "Live pressure now. History starts the moment SweepLoom opens.",
    );
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
    ui.horizontal_wrapped(|ui| {
        metric_card(
            ui,
            "MEMORY",
            &format_bytes(memory.used_bytes),
            &format!("of {}", format_bytes(memory.total_bytes)),
        );
        metric_card(
            ui,
            "IDLE SESSIONS",
            &format_bytes(reclaimable),
            &format!("{stale} idle long enough to consider"),
        );
        metric_card(
            ui,
            "CPU",
            &format!("{cpu:.0}%"),
            &format!("{stale_cpu:.0}% in idle sessions"),
        );
        let disk = if app.review.is_empty() {
            "open Review".to_owned()
        } else {
            format_bytes(
                app.review
                    .iter()
                    .map(|row| row.candidate.logical_bytes)
                    .sum(),
            )
        };
        metric_card(ui, "REVIEW DISK", &disk, "Generated + temp");
        if let Some((mount, total, avail)) = app.volumes.first() {
            metric_card(
                ui,
                "VOLUME",
                &format_bytes(*avail),
                &format!("free of {} on {}", format_bytes(*total), mount.display()),
            );
        }
    });
    ui.add_space(16.0);
    ui.label(RichText::new("Top opportunities").size(18.0).strong());
    ui.add_space(6.0);
    draw_opportunities(app, ui);
}

fn draw_opportunities(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let mut shown = 0_usize;
    let sessions: Vec<_> = app
        .sessions
        .iter()
        .filter(|session| session.recommendation.recommendation != Recommendation::Keep)
        .take(6)
        .map(|session| {
            (
                session.label().to_owned(),
                format_bytes(session.rss_bytes),
                session.recommendation.recommendation.label().to_owned(),
            )
        })
        .collect();
    for (label, rss, rec) in sessions {
        shown += 1;
        if list_row_at(ui, &label, &rss, &rec).clicked() {
            app.nav = Nav::Sessions;
        }
    }
    let processes = app
        .snapshot
        .as_ref()
        .map(|item| item.processes.as_slice())
        .unwrap_or(&[]);
    let pressure = BrowserPressure::from_live(&app.sessions, processes);
    if pressure.rss_bytes() > 0 && shown < 8 {
        shown += 1;
        if list_row_at(
            ui,
            "Browser",
            &format_bytes(pressure.rss_bytes()),
            "companion needed for tab discard",
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
        if list_row_at(ui, &name, &size, "disk").clicked() {
            app.nav = Nav::Storage;
        }
    }
    if shown == 0 {
        ui.label("No idle sessions. Open Review to list Cargo target / node_modules without an Explorer scan.");
    }
}
