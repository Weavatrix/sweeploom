//! Display helpers for saved measurements: timestamps, deltas and trend cells.

use super::*;

pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

pub(crate) fn signed_bytes(delta: i128) -> String {
    if delta == 0 {
        return "Unchanged".into();
    }
    let bytes = delta.unsigned_abs().min(u128::from(u64::MAX)) as u64;
    let size = if bytes < 1000 {
        format!("{bytes} B")
    } else {
        crate::format::format_bytes(bytes)
    };
    format!("{}{size}", if delta > 0 { "+" } else { "-" })
}

pub(crate) fn timestamp(ms: u64) -> String {
    // Gregorian calendar conversion, UTC. No locale or time-zone ambiguity in
    // comparisons after travel or a daylight-saving change.
    let seconds = ms / 1000;
    let days = (seconds / 86400) as i64 + 719468;
    let era = days.div_euclid(146097);
    let doe = days - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02} UTC",
        seconds / 3600 % 24,
        seconds / 60 % 60,
        seconds % 60
    )
}

/// Consistent visible trend in every disk table; full dates in the tooltip.
pub(crate) fn trend(ui: &mut eframe::egui::Ui, history: &ScanHistory, source: Source, path: &Path) {
    trend_at(ui, history, source, path, None);
}

pub(crate) fn trend_at(
    ui: &mut eframe::egui::Ui,
    history: &ScanHistory,
    source: Source,
    path: &Path,
    at: Option<u64>,
) {
    let Some(series) = history.get(source, path) else {
        ui.label(
            eframe::egui::RichText::new("No history")
                .small()
                .color(crate::theme::muted(ui)),
        );
        return;
    };
    let count = at.map_or(series.points.len(), |at| {
        series.points.partition_point(|point| point.at <= at)
    });
    if count == 0 {
        ui.label("No history");
        return;
    }
    let latest = &series.points[count - 1];
    let previous = series.points[..count - 1]
        .iter()
        .rev()
        .find(|point| point.usage.complete && point.basis == latest.basis);
    let delta = latest
        .usage
        .complete
        .then_some(previous)
        .flatten()
        .map(|before| {
            (
                i128::from(latest.usage.bytes) - i128::from(before.usage.bytes),
                before,
            )
        });
    let (caption, note) = match delta {
        Some((delta, before)) => {
            let percent = if before.usage.bytes > 0 {
                format!(
                    " ({:+.1}%)",
                    delta as f64 / before.usage.bytes as f64 * 100.0
                )
            } else {
                String::new()
            };
            (
                signed_bytes(delta),
                format!(
                    "{}{} since {}\nPrevious: {} bytes\nLatest: {} bytes at {}\nExact change: {delta:+} bytes\n{} saved measurements\n{}",
                    signed_bytes(delta),
                    percent,
                    timestamp(before.at),
                    before.usage.bytes,
                    latest.usage.bytes,
                    timestamp(latest.at),
                    count,
                    source.label()
                ),
            )
        }
        None if !latest.usage.complete => (
            "Partial".into(),
            format!(
                "Incomplete scan at {}. Growth/shrinkage cannot be determined.",
                timestamp(latest.at)
            ),
        ),
        None => (
            "First scan".into(),
            format!(
                "Measured at {}. Scan again to compare.\n{}",
                timestamp(latest.at),
                source.label()
            ),
        ),
    };
    let color = match delta.map(|(value, _)| value) {
        Some(value) if value > 0 => crate::theme::accent(),
        Some(value) if value < 0 => crate::theme::ok(),
        _ => crate::theme::muted(ui),
    };
    ui.label(eframe::egui::RichText::new(caption).small().color(color))
        .on_hover_text(note);
}

pub(crate) fn candidate_value(
    candidate: &sweeploom_core::Candidate,
    complete: bool,
) -> (PathBuf, DiskUsage, Basis) {
    (
        candidate.path.clone(),
        DiskUsage {
            bytes: candidate.estimated_reclaimable_bytes(),
            logical_bytes: candidate.logical_bytes,
            files: candidate.file_count,
            errors: 0,
            complete,
        },
        if candidate.allocated_bytes.is_some() {
            Basis::Allocated
        } else {
            Basis::Logical
        },
    )
}
