//! Folder and kind groups for the Projects table. Totals are computed once
//! per group and frame; ancestor checks are map lookups, not pairwise scans.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

use sweeploom_storage::DiskUsage;

use super::{Line, ProjectCard, ProjectGroup};
use crate::sort::{Col, Sort};

pub(crate) fn size_order(
    left: Option<DiskUsage>,
    right: Option<DiskUsage>,
    desc: bool,
) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let value =
        |usage: Option<DiskUsage>| usage.filter(|u| u.complete || u.bytes > 0).map(|u| u.bytes);
    match (value(left), value(right)) {
        (Some(left), Some(right)) => {
            if desc {
                right.cmp(&left)
            } else {
                left.cmp(&right)
            }
        }
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

pub(crate) fn table_lines(
    cards: &[ProjectCard],
    group: ProjectGroup,
    collapsed: &HashSet<String>,
    sort: Sort,
) -> Vec<Line> {
    if group == ProjectGroup::None {
        return (0..cards.len()).map(Line::Project).collect();
    }
    let mut buckets: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, card) in cards.iter().enumerate() {
        buckets
            .entry(card.group_key(group))
            .or_default()
            .push(index);
    }
    // Title and total once per group; sorting compares the cached values.
    let mut groups: Vec<(String, Vec<usize>, String, Option<DiskUsage>)> = buckets
        .into_iter()
        .map(|(key, indexes)| {
            let title = group_title(cards, group, &key, &indexes);
            let bytes = group_bytes(cards, &indexes);
            (key, indexes, title, bytes)
        })
        .collect();
    groups.sort_by(|left, right| {
        let order = match sort.col {
            Col::Size => return size_order(left.3, right.3, sort.desc).then(left.0.cmp(&right.0)),
            Col::Name => left.2.to_lowercase().cmp(&right.2.to_lowercase()),
            _ if group == ProjectGroup::Kind => kind_rank(&left.0).cmp(&kind_rank(&right.0)),
            _ => left.0.cmp(&right.0),
        }
        .then(left.0.cmp(&right.0));
        if sort.desc { order.reverse() } else { order }
    });
    let mut lines = Vec::new();
    for (key, indexes, title, bytes) in groups {
        let expanded = !collapsed.contains(&key);
        lines.push(Line::Group {
            count: indexes.len(),
            key,
            title,
            bytes,
            expanded,
        });
        if expanded {
            lines.extend(indexes.into_iter().map(Line::Project));
        }
    }
    lines
}

fn group_title(cards: &[ProjectCard], group: ProjectGroup, key: &str, indexes: &[usize]) -> String {
    if group == ProjectGroup::Parent {
        return indexes
            .first()
            .and_then(|&index| cards.get(index))
            .map(|card| card.folder.clone())
            .unwrap_or_else(|| key.to_owned());
    }
    key.to_owned()
}

/// Group total, counting nested projects once. Ancestor lookups go through a
/// map, so a folder with hundreds of projects stays cheap every frame.
fn group_bytes(cards: &[ProjectCard], indexes: &[usize]) -> Option<DiskUsage> {
    let members: HashMap<&Path, &ProjectCard> = indexes
        .iter()
        .map(|&index| (cards[index].path.as_path(), &cards[index]))
        .collect();
    let has_ancestor = |card: &ProjectCard, measured_only: bool| {
        card.path.ancestors().skip(1).any(|parent| {
            members
                .get(parent)
                .is_some_and(|outer| !measured_only || outer.bytes.is_some())
        })
    };
    let roots: Vec<&ProjectCard> = members
        .values()
        .copied()
        .filter(|card| !has_ancestor(card, false))
        .collect();
    let mut total = DiskUsage::default();
    let mut found = false;
    for card in members.values() {
        let Some(usage) = card.bytes else {
            continue;
        };
        if has_ancestor(card, true) {
            continue;
        }
        found = true;
        total.bytes = total.bytes.saturating_add(usage.bytes);
        total.logical_bytes = total.logical_bytes.saturating_add(usage.logical_bytes);
        total.files = total.files.saturating_add(usage.files);
        total.errors = total.errors.saturating_add(usage.errors);
    }
    total.complete = roots
        .iter()
        .all(|card| card.bytes.is_some_and(|u| u.complete));
    if found {
        return Some(total);
    }
    roots.iter().all(|card| card.size_error.is_some()).then(|| {
        total.errors = roots.len().max(1) as u64;
        total
    })
}

fn kind_rank(label: &str) -> u8 {
    match label {
        "Cargo" => 0,
        "Node" => 1,
        "Go" => 2,
        "Python" => 3,
        "Xcode" => 4,
        "Swift" => 5,
        "JVM" => 6,
        ".NET" => 7,
        _ => 8,
    }
}
