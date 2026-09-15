//! History table grouping. Does not walk the disk.

use std::collections::BTreeMap;

use sweeploom_core::{LiveSession, ProcessSnapshot, SessionId, SessionKind};

use crate::sort::{Col, Sort};

use super::session_label;

/// One tracked process in History.
#[derive(Clone, Debug)]
pub struct HistRow {
    pub name: String,
    pub pid: u32,
    pub rss: u64,
    pub peak: u64,
    pub cpu: f32,
    pub avg_5m: String,
    pub samples: usize,
    pub spark: Vec<f32>,
    pub session: Option<SessionId>,
}

/// Flattened history line.
#[derive(Clone, Debug)]
pub enum Line {
    /// Session or same-name process group.
    Group {
        /// Collapse key.
        key: String,
        /// Title.
        title: String,
        /// Session kind when the bucket is a live session.
        kind: Option<SessionKind>,
        /// Rolled RSS.
        rss: u64,
        /// Max member peak RSS.
        peak: u64,
        /// Rolled CPU.
        cpu: f32,
        /// Sum of member 5m averages, or unavailable.
        avg_5m: String,
        /// CPU series folded from the newest sample.
        spark: Vec<f32>,
        /// Longest member sample ring.
        samples: usize,
        /// Member count.
        count: usize,
        /// Children currently shown.
        expanded: bool,
    },
    /// One tracked process.
    Item(HistRow),
}

/// Group live processes by session, then by executable name.
#[must_use]
pub fn table_lines(
    rows: &[HistRow],
    sessions: &[LiveSession],
    processes: &[ProcessSnapshot],
    sort: Sort,
    expanded: &std::collections::HashSet<String>,
) -> Vec<Line> {
    let mut buckets: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, row) in rows.iter().enumerate() {
        buckets
            .entry(group_key(row, sessions))
            .or_default()
            .push(index);
    }
    let mut keys: Vec<String> = buckets.keys().cloned().collect();
    keys.sort_by(|left, right| {
        let left_rss: u64 = rss_of(&buckets[left], rows);
        let right_rss: u64 = rss_of(&buckets[right], rows);
        match sort.col {
            Col::Name => left.cmp(right),
            _ => left_rss.cmp(&right_rss),
        }
    });
    if sort.desc && sort.col != Col::Name {
        keys.reverse();
    }
    let mut lines = Vec::new();
    for key in keys {
        let indexes = buckets.remove(&key).unwrap_or_default();
        let title = title_of(&key, sessions, processes);
        let kind = kind_of(&key, sessions);
        let open = expanded.contains(&key);
        lines.push(Line::Group {
            key: key.clone(),
            title,
            kind,
            rss: rss_of(&indexes, rows),
            peak: peak_of(&indexes, rows),
            cpu: indexes.iter().map(|&i| rows[i].cpu).sum(),
            avg_5m: avg_5m_of(&indexes, rows),
            spark: fold_spark(&indexes, rows),
            samples: indexes.iter().map(|&i| rows[i].samples).max().unwrap_or(0),
            count: indexes.len(),
            expanded: open,
        });
        if open {
            lines.extend(
                indexes
                    .into_iter()
                    .map(|index| Line::Item(rows[index].clone())),
            );
        }
    }
    lines
}

fn rss_of(indexes: &[usize], rows: &[HistRow]) -> u64 {
    indexes.iter().map(|&i| rows[i].rss).sum()
}

fn peak_of(indexes: &[usize], rows: &[HistRow]) -> u64 {
    indexes.iter().map(|&i| rows[i].peak).max().unwrap_or(0)
}

fn avg_5m_of(indexes: &[usize], rows: &[HistRow]) -> String {
    let mut sum = 0.0_f32;
    let mut any = false;
    for &index in indexes {
        if let Some(cpu) = parse_avg(&rows[index].avg_5m) {
            sum += cpu;
            any = true;
        }
    }
    if any {
        format!("{sum:.1}%")
    } else {
        "unavailable".to_owned()
    }
}

fn parse_avg(text: &str) -> Option<f32> {
    text.strip_suffix('%')?.parse().ok()
}

fn fold_spark(indexes: &[usize], rows: &[HistRow]) -> Vec<f32> {
    let len = indexes
        .iter()
        .map(|&index| rows[index].spark.len())
        .max()
        .unwrap_or(0);
    (0..len)
        .map(|offset| {
            let from_end = len - 1 - offset;
            indexes
                .iter()
                .map(|&index| {
                    let spark = &rows[index].spark;
                    spark
                        .len()
                        .checked_sub(from_end + 1)
                        .and_then(|at| spark.get(at).copied())
                        .unwrap_or(0.0)
                })
                .sum()
        })
        .collect()
}

fn group_key(row: &HistRow, sessions: &[LiveSession]) -> String {
    if let Some(id) = row.session
        && sessions.iter().any(|item| item.id == id)
    {
        return format!("s:{}", id.0);
    }
    format!("n:{}", stem(&row.name))
}

fn title_of(key: &str, sessions: &[LiveSession], processes: &[ProcessSnapshot]) -> String {
    if let Some(session) = session_of(key, sessions) {
        return session_label::title(session, processes);
    }
    key.strip_prefix("n:").unwrap_or(key).to_owned()
}

fn kind_of(key: &str, sessions: &[LiveSession]) -> Option<SessionKind> {
    session_of(key, sessions).map(|session| session.kind)
}

fn session_of<'a>(key: &str, sessions: &'a [LiveSession]) -> Option<&'a LiveSession> {
    let rest = key.strip_prefix("s:")?;
    let id = rest.parse::<u64>().ok()?;
    sessions.iter().find(|item| item.id == SessionId(id))
}

fn stem(name: &str) -> String {
    name.strip_suffix(".exe")
        .or_else(|| name.strip_suffix(".EXE"))
        .unwrap_or(name)
        .to_owned()
}
