//! Session share of host RAM/CPU and the smallest set covering ~80%.

use sweeploom_core::{LiveSession, SessionId};

use crate::app::SweepLoomApp;
use crate::format::format_bytes;

use super::session_label;

/// One heavy session in a coverage list.
#[derive(Clone, Debug, PartialEq)]
pub struct Hog {
    /// Session to open on click.
    pub id: SessionId,
    /// Display title.
    pub title: String,
    /// Combined RSS.
    pub rss: u64,
    /// Combined CPU percent.
    pub cpu: f32,
    /// Share of the chosen denominator, 0–1.
    pub share: f32,
}

/// Input row for [`cover_rss`].
#[derive(Clone, Debug)]
pub struct PressureRow {
    /// Session id.
    pub id: SessionId,
    /// Display title.
    pub title: String,
    /// Combined RSS.
    pub rss: u64,
    /// Combined CPU percent.
    pub cpu: f32,
}

/// Live RAM/CPU vs the host totals.
#[must_use]
pub fn share_line(session: &LiveSession, used_rss: u64, total_rss: u64, host_cpu: f32) -> String {
    let ram_pct = percent(session.rss_bytes, used_rss.max(1));
    let cpu_note = if host_cpu > 0.05 {
        format!("machine CPU {host_cpu:.0}%")
    } else {
        "machine CPU settling".to_owned()
    };
    format!(
        "This session {} RAM ({ram_pct:.0}% of {} in use / {} total) · {:.1}% CPU · {cpu_note}",
        format_bytes(session.rss_bytes),
        format_bytes(used_rss),
        format_bytes(total_rss),
        session.cpu_percent
    )
}

/// Largest sessions until `share` of `denom` RSS is covered, at most 10.
#[must_use]
pub fn cover_rss(rows: &[PressureRow], denom: u64, share: f32) -> Vec<Hog> {
    let mut order: Vec<usize> = (0..rows.len()).collect();
    order.sort_by(|&left, &right| rows[right].rss.cmp(&rows[left].rss));
    let target = ((denom as f64) * f64::from(share.clamp(0.0, 1.0))) as u64;
    let mut covered = 0_u64;
    let mut out = Vec::new();
    for index in order {
        if out.len() >= 10 || (covered >= target && !out.is_empty()) {
            break;
        }
        let row = &rows[index];
        if row.rss == 0 {
            continue;
        }
        covered = covered.saturating_add(row.rss);
        out.push(Hog {
            id: row.id,
            title: row.title.clone(),
            rss: row.rss,
            cpu: row.cpu,
            share: percent(row.rss, denom.max(1)) / 100.0,
        });
    }
    out
}

/// Heaviest sessions covering ~80% of in-use RAM.
#[must_use]
pub fn heaviest(app: &SweepLoomApp) -> Vec<Hog> {
    let processes = app
        .snapshot
        .as_ref()
        .map(|item| item.processes.as_slice())
        .unwrap_or(&[]);
    let rows: Vec<PressureRow> = app
        .sessions
        .iter()
        .map(|session| PressureRow {
            id: session.id,
            title: session_label::title(session, processes),
            rss: session.rss_bytes,
            cpu: session.cpu_percent,
        })
        .collect();
    let denom = app
        .snapshot
        .as_ref()
        .map(|item| item.memory.used_bytes.max(item.total_rss_bytes))
        .unwrap_or(1)
        .max(1);
    cover_rss(&rows, denom, 0.8)
}

/// Caption above the Overview heaviest list.
#[must_use]
pub fn heaviest_caption(app: &SweepLoomApp, hogs: &[Hog]) -> String {
    if hogs.is_empty() {
        return String::new();
    }
    let denom = app
        .snapshot
        .as_ref()
        .map(|item| item.memory.used_bytes.max(item.total_rss_bytes))
        .unwrap_or(1)
        .max(1);
    let held: u64 = hogs.iter().map(|item| item.rss).sum();
    let total = app
        .snapshot
        .as_ref()
        .map(|item| format_bytes(item.memory.total_bytes))
        .unwrap_or_else(|| "—".to_owned());
    format!(
        "Top {} hold {:.0}% of RAM in use ({}) — {} of {total}",
        hogs.len(),
        percent(held, denom),
        format_bytes(held),
        format_bytes(denom),
    )
}

fn percent(part: u64, whole: u64) -> f32 {
    (part as f64 / whole as f64 * 100.0) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: u64, title: &str, rss: u64) -> PressureRow {
        PressureRow {
            id: SessionId(id),
            title: title.into(),
            rss,
            cpu: 1.0,
        }
    }

    #[test]
    fn cover_stops_near_eighty_percent() {
        let rows = [
            row(1, "a", 50),
            row(2, "b", 30),
            row(3, "c", 10),
            row(4, "d", 10),
        ];
        let hogs = cover_rss(&rows, 100, 0.8);
        let titles: Vec<_> = hogs.iter().map(|item| item.title.as_str()).collect();
        assert_eq!(titles, ["a", "b"]);
        assert_eq!(hogs[0].id, SessionId(1));
        assert_eq!(hogs[1].id, SessionId(2));
        assert!(hogs.iter().map(|item| item.rss).sum::<u64>() >= 80);
    }
}
