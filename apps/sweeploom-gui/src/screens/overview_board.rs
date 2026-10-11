//! Memory and disk breakdown bars for the Overview.

use eframe::egui;

use super::super::session_pressure;
use crate::app::SweepLoomApp;
use crate::format::format_bytes;
use crate::nav::Nav;
use crate::theme;
use crate::widgets::{self, Segment};

/// Two equal cards side by side, stacked on narrow windows.
pub(super) fn breakdowns(app: &mut SweepLoomApp, ui: &mut egui::Ui) {
    let memory = memory_segments(app, ui);
    let disk = disk_segments(app, ui);
    let mut open = None;
    widgets::pair(
        ui,
        "overview-breakdowns",
        widgets::card_frame,
        |ui| {
            widgets::breakdown(ui, "Memory", &memory.0, &memory.1);
            ui.add_space(theme::LG);
            let (total, workloads) = workload_segments(app, ui);
            widgets::breakdown(ui, "Resident memory by workload", &total, &workloads);
            ui.add_space(theme::MD);
            widgets::caption(
                ui,
                "Session slices are scaled from resident memory; shared pages make raw RSS add up past RAM in use.",
            );
        },
        |ui| {
            widgets::breakdown(ui, "Disk", &disk.0, &disk.1);
            ui.add_space(theme::MD);
            if largest(app, ui) {
                open = Some(Nav::DiskHistory);
            }
            if let Some(scan) = app.scan_history.scans().last() {
                let name = scan.report.root.file_name().map_or_else(
                    || scan.report.root.display().to_string(),
                    |n| n.to_string_lossy().into_owned(),
                );
                let caption = format!(
                    "Last Explorer scan: {name} · {}",
                    crate::format::ago(crate::scan_history::now_ms(), scan.at)
                );
                let text = egui::RichText::new(caption)
                    .size(12.5)
                    .color(theme::muted(ui));
                if widgets::pointer(ui.add(egui::Label::new(text).sense(egui::Sense::click())))
                    .on_hover_text(scan.report.root.display().to_string())
                    .clicked()
                {
                    open = Some(Nav::DiskHistory);
                }
            }
        },
    );
    if let Some(nav) = open {
        app.nav = nav;
    }
}

fn memory_segments(app: &SweepLoomApp, ui: &egui::Ui) -> (String, Vec<Segment>) {
    let Some(snapshot) = &app.snapshot else {
        return ("—".into(), Vec::new());
    };
    let memory = snapshot.memory;
    let available = memory
        .available_bytes
        .max(memory.total_bytes.saturating_sub(memory.used_bytes));
    let in_use = memory.total_bytes.saturating_sub(available);
    let hogs = session_pressure::heaviest(app);
    let mut segments = Vec::new();
    let mut attributed = 0_u64;
    for (index, hog) in hogs.iter().take(4).enumerate() {
        let scaled = (hog.share.clamp(0.0, 1.0) as f64 * in_use as f64) as u64;
        attributed = attributed.saturating_add(scaled);
        segments.push(Segment {
            label: hog.title.clone(),
            value: scaled as f64,
            detail: format!("{} RSS", format_bytes(hog.rss)),
            // Ranked sessions: one hue, darker = larger (ordinal, not identity).
            color: theme::seq(ui, 1.0 - index as f32 * 0.22),
        });
    }
    segments.push(Segment {
        label: "Other in use".into(),
        value: in_use.saturating_sub(attributed) as f64,
        detail: format_bytes(in_use.saturating_sub(attributed)),
        color: theme::series_rest(ui),
    });
    segments.push(Segment {
        label: "Available".into(),
        value: available as f64,
        detail: format_bytes(available),
        color: theme::inset(ui),
    });
    (
        format!(
            "{} in use of {}",
            format_bytes(in_use),
            format_bytes(memory.total_bytes)
        ),
        segments,
    )
}

fn disk_segments(app: &SweepLoomApp, ui: &egui::Ui) -> (String, Vec<Segment>) {
    let Some((mount, total, avail)) = app.volumes.first() else {
        return ("No volume information".into(), Vec::new());
    };
    let used = total.saturating_sub(*avail);
    let review = super::tiles::review_bytes(app);
    let reclaim = review.map_or(0, |(bytes, _)| bytes.min(used));
    let review = review.filter(|(bytes, _)| *bytes > 0);
    let mut segments = vec![Segment {
        label: "Used".into(),
        value: used.saturating_sub(reclaim) as f64,
        detail: format_bytes(used.saturating_sub(reclaim)),
        color: theme::series(ui, 0),
    }];
    if let Some((_, complete)) = review {
        segments.push(Segment {
            label: "Reclaimable (Review)".into(),
            value: reclaim as f64,
            detail: crate::format::format_bytes_bound(reclaim, complete),
            color: theme::series(ui, 1),
        });
    }
    segments.push(Segment {
        label: "Free".into(),
        value: *avail as f64,
        detail: format_bytes(*avail),
        color: theme::inset(ui),
    });
    (
        format!(
            "{} free of {} on {}",
            format_bytes(*avail),
            format_bytes(*total),
            mount.display()
        ),
        segments,
    )
}

/// Largest measured locations from saved scans. Nested paths count once.
fn largest(app: &SweepLoomApp, ui: &mut egui::Ui) -> bool {
    let mut top: Vec<(&std::path::Path, u64, Option<i128>)> = Vec::new();
    for series in app.scan_history.series() {
        let bytes = series.latest().usage.bytes;
        if top.len() == 4 && top.last().is_some_and(|(_, b, _)| *b >= bytes) {
            continue;
        }
        let path = series.path.as_path();
        if top
            .iter()
            .any(|(p, ..)| p.starts_with(path) || path.starts_with(p))
        {
            continue;
        }
        top.push((path, bytes, series.delta().map(|(d, _)| d)));
        top.sort_by_key(|(_, b, _)| std::cmp::Reverse(*b));
        top.truncate(4);
    }
    if top.is_empty() {
        return false;
    }
    ui.label(
        egui::RichText::new("Largest measured locations")
            .size(12.5)
            .color(theme::muted(ui)),
    );
    ui.add_space(theme::XS);
    let max = top.first().map_or(1, |(_, b, _)| *b).max(1);
    let mut clicked = false;
    for (path, bytes, delta) in &top {
        let row = ui.horizontal(|ui| {
            let name = crate::format::tilde(path, &app.locations.home);
            let width = (ui.available_width() - 210.0).max(80.0);
            ui.allocate_ui_with_layout(
                egui::vec2(width, 20.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.set_width(width);
                    ui.add(egui::Label::new(egui::RichText::new(name).size(13.0)).truncate());
                },
            );
            widgets::meter(
                ui,
                *bytes as f32 / max as f32,
                theme::series(ui, 0),
                80.0,
                6.0,
            );
            ui.label(egui::RichText::new(format_bytes(*bytes)).size(13.0));
            if let Some(delta) = delta.filter(|d| *d != 0) {
                let tone = if delta > 0 {
                    theme::Tone::Caution
                } else {
                    theme::Tone::Ok
                };
                ui.label(
                    egui::RichText::new(crate::scan_history::signed_bytes(delta))
                        .size(12.0)
                        .color(theme::tone(ui, tone)),
                );
            }
        });
        clicked |= widgets::pointer(row.response.interact(egui::Sense::click())).clicked();
    }
    ui.add_space(theme::SM);
    clicked
}

const WORKLOADS: [&str; 5] = [
    "AI agents",
    "Dev servers & builds",
    "MCP servers",
    "Browsers",
    "Apps & other",
];

fn workload(kind: sweeploom_core::SessionKind) -> usize {
    use sweeploom_core::SessionKind as K;
    match kind {
        K::DevServer | K::Build | K::TestRunner | K::LanguageServer => 1,
        K::Mcp => 2,
        K::Browser => 3,
        _ if kind.is_agent() => 0,
        _ => 4,
    }
}

/// RSS summed per workload family. Fixed order and colors.
fn workload_segments(app: &SweepLoomApp, ui: &egui::Ui) -> (String, Vec<Segment>) {
    let mut sums = [0_u64; WORKLOADS.len()];
    for session in &app.sessions {
        let slot = &mut sums[workload(session.kind)];
        *slot = slot.saturating_add(session.rss_bytes);
    }
    let total: u64 = sums.iter().sum();
    let segments = WORKLOADS
        .iter()
        .zip(sums)
        .enumerate()
        .filter(|(_, (_, bytes))| *bytes > 0)
        .map(|(index, (label, bytes))| Segment {
            label: (*label).to_owned(),
            value: bytes as f64,
            detail: format_bytes(bytes),
            // Categorical slots 2–5 keep blue free for the ranked bar above.
            color: if index == 4 {
                theme::series_rest(ui)
            } else {
                theme::series(ui, index + 1)
            },
        })
        .collect();
    (
        format!("{} RSS across sessions", format_bytes(total)),
        segments,
    )
}
