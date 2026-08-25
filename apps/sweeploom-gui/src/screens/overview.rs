//! Overview pressure cards and top opportunities.

use eframe::egui::{self, RichText};
use sweeploom_browser::BrowserPressure;
use sweeploom_core::Recommendation;

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::widgets::{metric_card, page_title};

pub fn ui_overview(app: &SweepLoomApp, ui: &mut egui::Ui) {
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
    ui.horizontal(|ui| {
        metric_card(
            ui,
            "MEMORY",
            &format_bytes(memory.used_bytes),
            &format!("of {}", format_bytes(memory.total_bytes)),
        );
        metric_card(
            ui,
            "RECLAIMABLE SESSIONS",
            &format_bytes(reclaimable),
            &format!("{stale} stale candidates"),
        );
        metric_card(
            ui,
            "CPU",
            &format!("{cpu:.0}%"),
            &format!("{stale_cpu:.0}% in forgotten sessions"),
        );
        let disk = app.inventory.as_ref().map_or_else(
            || "scan Explorer".to_owned(),
            |item| format_bytes(item.tree.logical_bytes),
        );
        metric_card(ui, "SCANNED DISK", &disk, "Folder Inspector");
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

fn draw_opportunities(app: &SweepLoomApp, ui: &mut egui::Ui) {
    let mut shown = 0_usize;
    for session in &app.sessions {
        if session.recommendation.recommendation == Recommendation::Keep {
            continue;
        }
        shown += 1;
        if shown > 8 {
            break;
        }
        ui.label(format!(
            "• {}  {}  {}  {:?}",
            session.label(),
            format_bytes(session.rss_bytes),
            session
                .project
                .as_ref()
                .map(|item| item.0.display().to_string())
                .unwrap_or_default(),
            session.recommendation.recommendation
        ));
    }
    let processes = app
        .snapshot
        .as_ref()
        .map(|item| item.processes.as_slice())
        .unwrap_or(&[]);
    let pressure = BrowserPressure::from_live(&app.sessions, processes);
    if pressure.rss_bytes() > 0 && shown < 8 {
        shown += 1;
        ui.label(format!(
            "• Browser  {}  companion needed for tab discard",
            format_bytes(pressure.rss_bytes())
        ));
    }
    if shown == 0 {
        ui.label("No forgotten-session candidates in this sample. Keep watching.");
    }
}
