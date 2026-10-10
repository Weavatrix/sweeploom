//! Sessions table grouping by app title. Does not merge process trees.

use std::collections::{BTreeMap, HashSet};

use sweeploom_core::SessionKind;

use crate::sort::{Col, Sort};

/// One visible session used to build the table.
#[derive(Clone, Debug)]
pub struct SessionRow {
    /// Index into the live session slice.
    pub index: usize,
    /// App title (Cursor, Claude Code, …).
    pub title: String,
    /// Workload/installation grouping key; unknown Node launches stay separate.
    pub group_key: String,
    /// Runtime, PID and uptime.
    pub subtitle: String,
    /// Number of Node executables among all member processes.
    pub node_procs: usize,
    /// Observed idle duration or missing coverage.
    pub status_detail: String,
    /// Full working directory and launch origin.
    pub source_tip: String,
    /// Session kind.
    pub kind: SessionKind,
    /// RSS.
    pub rss: u64,
    /// CPU percent.
    pub cpu: f32,
    /// Member process count.
    pub procs: usize,
    /// Activity label.
    pub status: String,
    /// Project folder name.
    pub project: String,
}

/// Flattened Sessions line.
#[derive(Clone, Debug)]
pub enum Line {
    /// Several sessions that share a title.
    Group {
        /// Collapse key (the title).
        key: String,
        /// Title.
        title: String,
        /// Number of Node executable processes.
        node_procs: usize,
        /// Kind of the first member.
        kind: SessionKind,
        /// Sum RSS.
        rss: u64,
        /// Sum CPU.
        cpu: f32,
        /// Sum process counts.
        procs: usize,
        /// Member count.
        count: usize,
        /// Worst activity label.
        status: String,
        /// Shared project, or em dash.
        project: String,
        /// Children currently shown.
        expanded: bool,
        /// Live-session indexes.
        members: Vec<usize>,
    },
    /// One session. `nested` when it sits under a group.
    Session {
        /// Live-session index.
        index: usize,
        /// Indented under a group.
        nested: bool,
        /// App title.
        title: String,
    },
}

/// Group by title. Singleton titles stay a plain row. Empty `expanded` is collapsed.
#[must_use]
pub fn table_lines(rows: &[SessionRow], sort: Sort, expanded: &HashSet<String>) -> Vec<Line> {
    let mut buckets: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, row) in rows.iter().enumerate() {
        buckets
            .entry(row.group_key.clone())
            .or_default()
            .push(index);
    }
    let mut keys: Vec<String> = buckets.keys().cloned().collect();
    keys.sort_by(|left, right| cmp_bucket(left, right, &buckets, rows, sort));
    if sort.desc {
        keys.reverse();
    }
    let mut lines = Vec::new();
    for key in keys {
        let mut indexes = buckets.remove(&key).unwrap_or_default();
        sort_indexes(&mut indexes, rows, sort);
        push_bucket(&mut lines, &key, &indexes, rows, expanded);
    }
    lines
}

fn push_bucket(
    lines: &mut Vec<Line>,
    key: &str,
    indexes: &[usize],
    rows: &[SessionRow],
    expanded: &HashSet<String>,
) {
    if indexes.len() == 1 {
        lines.push(Line::Session {
            index: rows[indexes[0]].index,
            nested: false,
            title: rows[indexes[0]].title.clone(),
        });
        return;
    }
    let open = expanded.contains(key);
    let members: Vec<usize> = indexes.iter().map(|&i| rows[i].index).collect();
    lines.push(roll_group(key, indexes, rows, open, members.clone()));
    if open {
        for &row_index in indexes {
            lines.push(Line::Session {
                index: rows[row_index].index,
                nested: true,
                title: rows[row_index].title.clone(),
            });
        }
    }
}

fn roll_group(
    key: &str,
    indexes: &[usize],
    rows: &[SessionRow],
    expanded: bool,
    members: Vec<usize>,
) -> Line {
    Line::Group {
        key: key.to_owned(),
        title: rows[indexes[0]].title.clone(),
        node_procs: indexes.iter().map(|&i| rows[i].node_procs).sum(),
        kind: rows[indexes[0]].kind,
        rss: indexes.iter().map(|&i| rows[i].rss).sum(),
        cpu: indexes.iter().map(|&i| rows[i].cpu).sum(),
        procs: indexes.iter().map(|&i| rows[i].procs).sum(),
        count: indexes.len(),
        status: worst_status(indexes.iter().map(|&i| rows[i].status.as_str())),
        project: shared_project(indexes, rows),
        expanded,
        members,
    }
}

fn sort_indexes(indexes: &mut [usize], rows: &[SessionRow], sort: Sort) {
    indexes.sort_by(|&left, &right| cmp_row(&rows[left], &rows[right], sort));
    if sort.desc {
        indexes.reverse();
    }
}

fn cmp_bucket(
    left: &str,
    right: &str,
    buckets: &BTreeMap<String, Vec<usize>>,
    rows: &[SessionRow],
    sort: Sort,
) -> std::cmp::Ordering {
    let left_idx = &buckets[left];
    let right_idx = &buckets[right];
    match sort.col {
        Col::Name => left.cmp(right),
        Col::Procs => sum_procs(left_idx, rows).cmp(&sum_procs(right_idx, rows)),
        Col::Cpu => sum_cpu(left_idx, rows).total_cmp(&sum_cpu(right_idx, rows)),
        Col::Status | Col::Safety => {
            worst_status(left_idx.iter().map(|&i| rows[i].status.as_str())).cmp(&worst_status(
                right_idx.iter().map(|&i| rows[i].status.as_str()),
            ))
        }
        Col::Size => sum_rss(left_idx, rows).cmp(&sum_rss(right_idx, rows)),
    }
}

fn cmp_row(left: &SessionRow, right: &SessionRow, sort: Sort) -> std::cmp::Ordering {
    match sort.col {
        Col::Name => left.title.cmp(&right.title),
        Col::Procs => left.procs.cmp(&right.procs),
        Col::Cpu => left.cpu.total_cmp(&right.cpu),
        Col::Status | Col::Safety => left.status.cmp(&right.status),
        Col::Size => left.rss.cmp(&right.rss),
    }
}

fn sum_rss(indexes: &[usize], rows: &[SessionRow]) -> u64 {
    indexes.iter().map(|&i| rows[i].rss).sum()
}

fn sum_cpu(indexes: &[usize], rows: &[SessionRow]) -> f32 {
    indexes.iter().map(|&i| rows[i].cpu).sum()
}

fn sum_procs(indexes: &[usize], rows: &[SessionRow]) -> usize {
    indexes.iter().map(|&i| rows[i].procs).sum()
}

fn shared_project(indexes: &[usize], rows: &[SessionRow]) -> String {
    let first = rows[indexes[0]].project.as_str();
    if indexes.iter().all(|&i| rows[i].project == first) {
        first.to_owned()
    } else {
        "Multiple workspaces".to_owned()
    }
}

pub(super) fn project_name(path: &std::path::Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned)
        .unwrap_or_else(|| crate::format::short_path(path))
}

fn worst_status<'a>(labels: impl Iterator<Item = &'a str>) -> String {
    const ORDER: &[&str] = &[
        "High CPU",
        "Possibly stale",
        "Forgotten",
        "Orphan helper",
        "Idle, heavy RAM",
        "Network active",
        "Active now",
        "Low activity",
        "Quiet agent",
        "Observing",
        "Working",
        "Idle",
        "Unknown",
    ];
    let set: Vec<&str> = labels.collect();
    ORDER
        .iter()
        .copied()
        .find(|label| set.contains(label))
        .or_else(|| set.first().copied())
        .unwrap_or("Unknown")
        .to_owned()
}
