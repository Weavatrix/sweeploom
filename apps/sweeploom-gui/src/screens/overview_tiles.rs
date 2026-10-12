//! Overview KPI tiles: memory, CPU, idle sessions, disk.

use sweeploom_core::Recommendation;

use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::icons::Glyph;
use crate::nav::{Dest, Nav};
use crate::widgets::{Metric, StatTile, TileViz, Tone, cpu_cores};

fn tile(
    icon: Glyph,
    label: &str,
    value: String,
    sub: String,
    tone: Tone,
    viz: TileViz,
) -> StatTile {
    StatTile {
        icon: Some(icon),
        label: label.into(),
        value,
        sub,
        tone,
        viz,
    }
}

pub(super) fn overview_cards(app: &SweepLoomApp, cores: &cpu_cores::CoreHistory) -> Vec<Metric> {
    vec![
        memory_card(app),
        cpu_card(app, cores),
        if crate::edition::STORE {
            access_card(app)
        } else {
            idle_card(app)
        },
        disk_card(app),
    ]
}

/// Where the live tiles lead: Sessions, which the App Store edition cannot show.
fn live_dest() -> Dest {
    Dest::Screen(if crate::edition::STORE {
        Nav::Overview
    } else {
        Nav::Sessions
    })
}

/// App Store edition: how many folders SweepLoom may read.
fn access_card(app: &SweepLoomApp) -> Metric {
    let count = app.folder_access.active().len();
    let (value, sub, tone) = if count == 0 {
        (
            "None".into(),
            "grant folders to start".into(),
            Tone::Caution,
        )
    } else {
        (count.to_string(), "folders granted".into(), Tone::Ok)
    };
    Metric {
        tile: tile(
            Glyph::Explorer,
            "Folder access",
            value,
            sub,
            tone,
            TileViz::None,
        ),
        open: Dest::Screen(Nav::Settings),
    }
}

fn memory_card(app: &SweepLoomApp) -> Metric {
    let memory = app
        .snapshot
        .as_ref()
        .map(|item| item.memory)
        .unwrap_or_default();
    let mem_total = memory.total_bytes.max(1);
    let mem_avail = memory
        .available_bytes
        .max(memory.total_bytes.saturating_sub(memory.used_bytes));
    let mem_used = 1.0 - (mem_avail as f32 / mem_total as f32);
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
        open: live_dest(),
    }
}

fn idle(app: &SweepLoomApp) -> impl Iterator<Item = &sweeploom_core::LiveSession> + Clone {
    app.sessions
        .iter()
        .filter(|session| session.recommendation.recommendation != Recommendation::Keep)
}

fn cpu_card(app: &SweepLoomApp, cores: &cpu_cores::CoreHistory) -> Metric {
    let cpu = app
        .snapshot
        .as_ref()
        .map(|item| item.cpu.usage_percent)
        .unwrap_or(0.0);
    // Float sums start at -0.0; keep the caption from reading "-0%".
    let stale_cpu = idle(app)
        .map(|session| session.cpu_percent.max(0.0))
        .sum::<f32>()
        .abs();
    Metric {
        tile: tile(
            Glyph::Cpu,
            "CPU load",
            format!("{cpu:.0}%"),
            if crate::edition::STORE {
                cores.summary()
            } else {
                format!("{} · {stale_cpu:.0}% in idle sessions", cores.summary())
            },
            if cpu >= 85.0 {
                Tone::Warn
            } else {
                Tone::Neutral
            },
            TileViz::Trend(cores.total.iter().copied().collect(), Some(100.0)),
        ),
        open: live_dest(),
    }
}

fn idle_card(app: &SweepLoomApp) -> Metric {
    let stale = idle(app)
        .filter(|session| {
            matches!(
                session.recommendation.recommendation,
                Recommendation::Recommended | Recommendation::StronglyRecommended
            )
        })
        .count();
    let reclaimable = idle(app)
        .map(|session| session.recommendation.estimated_reclaimable_rss)
        .sum::<u64>();
    let (value, sub, tone) = if stale == 0 {
        ("None".into(), "nothing idle yet".into(), Tone::Ok)
    } else {
        (
            format_bytes(reclaimable),
            format!("{stale} idle to review"),
            Tone::Caution,
        )
    };
    Metric {
        tile: tile(
            Glyph::Sessions,
            "Idle sessions",
            value,
            sub,
            tone,
            TileViz::None,
        ),
        open: Dest::Screen(Nav::Sessions),
    }
}

fn disk_card(app: &SweepLoomApp) -> Metric {
    let review = crate::screens::review_bytes(app);
    let sub = review.map_or_else(
        || "open Cleanup to measure".to_owned(),
        |(bytes, complete)| {
            if bytes == 0 {
                return "no project output to reclaim".to_owned();
            }
            format!(
                "{} in project output",
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
        open: Dest::ProjectOutput,
    }
}
