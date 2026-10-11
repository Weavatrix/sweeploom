//! AI store table rows. Grouping does not walk the disk.

use std::collections::{BTreeMap, HashSet};

use sweeploom_ai::{AiClass, AiOffer};

use crate::sort::{Col, Sort};

/// How the AI table is clustered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiGroup {
    /// Claude / Codex / Cursor.
    Tool,
    /// Secret / cache / history / …
    Category,
    /// Flat list of children.
    None,
}

impl AiGroup {
    /// Toolbar label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Tool => "Tool",
            Self::Category => "Category",
            Self::None => "None",
        }
    }
}

/// Flattened table line over [`AiOffer`] entries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    /// Collapsible group header.
    Group {
        /// Collapse key.
        key: String,
        /// Visible title (tool or category name only).
        title: String,
        /// Rolled logical bytes.
        bytes: u64,
        /// Rolled file count.
        files: u64,
        /// Child row count.
        count: usize,
    },
    /// `(offer index, entry index)`.
    Item(usize, usize),
}

/// Sort children in place, then flatten.
#[must_use]
pub fn table_lines(offers: &mut [AiOffer], sort: Sort, group: AiGroup) -> Vec<Line> {
    sort_entries(offers, sort);
    let lines = flatten(offers, group);
    sorted_lines(lines, offers, sort)
}

/// Visible lines after collapsing groups.
#[must_use]
pub fn visible_lines(lines: &[Line], collapsed: &HashSet<String>) -> Vec<Line> {
    let mut out = Vec::with_capacity(lines.len());
    let mut hide = false;
    for line in lines {
        match line {
            Line::Group { key, .. } => {
                hide = collapsed.contains(key);
                out.push(line.clone());
            }
            Line::Item(_, _) if hide => {}
            Line::Item(_, _) => out.push(line.clone()),
        }
    }
    out
}

fn sort_entries(offers: &mut [AiOffer], sort: Sort) {
    for offer in offers.iter_mut() {
        offer.entries.sort_by(|left, right| match sort.col {
            Col::Name => left
                .relative
                .to_lowercase()
                .cmp(&right.relative.to_lowercase()),
            Col::Status => left.class.label().cmp(right.class.label()),
            Col::Procs => left.candidate.file_count.cmp(&right.candidate.file_count),
            Col::Safety => left
                .candidate
                .safety
                .level
                .cmp(&right.candidate.safety.level),
            _ => bytes(&left.candidate).cmp(&bytes(&right.candidate)),
        });
        if sort.desc {
            offer.entries.reverse();
        }
    }
}

fn flatten(offers: &[AiOffer], group: AiGroup) -> Vec<Line> {
    match group {
        AiGroup::None => offers
            .iter()
            .enumerate()
            .flat_map(|(offer, item)| {
                (0..item.entries.len()).map(move |entry| Line::Item(offer, entry))
            })
            .collect(),
        AiGroup::Tool => flatten_tool(offers),
        AiGroup::Category => flatten_category(offers),
    }
}

fn flatten_tool(offers: &[AiOffer]) -> Vec<Line> {
    let mut lines = Vec::new();
    for (index, offer) in offers.iter().enumerate() {
        let tool = tool_name(offer);
        lines.push(Line::Group {
            key: format!("tool:{index}:{tool}"),
            title: tool.to_owned(),
            bytes: bytes(&offer.candidate),
            files: offer.candidate.file_count,
            count: offer.entries.len(),
        });
        lines.extend((0..offer.entries.len()).map(|entry| Line::Item(index, entry)));
    }
    lines
}

fn flatten_category(offers: &[AiOffer]) -> Vec<Line> {
    let mut buckets: BTreeMap<AiClass, Vec<(usize, usize)>> = BTreeMap::new();
    for (offer, item) in offers.iter().enumerate() {
        for (entry, child) in item.entries.iter().enumerate() {
            buckets.entry(child.class).or_default().push((offer, entry));
        }
    }
    let mut lines = Vec::new();
    for (class, items) in buckets {
        let bytes: u64 = items
            .iter()
            .map(|&(offer, entry)| bytes(&offers[offer].entries[entry].candidate))
            .sum();
        let files: u64 = items
            .iter()
            .map(|&(offer, entry)| offers[offer].entries[entry].candidate.file_count)
            .sum();
        lines.push(Line::Group {
            key: format!("class:{}", class.label()),
            title: class.label().to_owned(),
            bytes,
            files,
            count: items.len(),
        });
        lines.extend(
            items
                .into_iter()
                .map(|(offer, entry)| Line::Item(offer, entry)),
        );
    }
    lines
}

fn bytes(candidate: &sweeploom_core::Candidate) -> u64 {
    candidate.allocated_bytes.unwrap_or(candidate.logical_bytes)
}

fn sorted_lines(lines: Vec<Line>, offers: &[AiOffer], sort: Sort) -> Vec<Line> {
    let compare_items = |a: &Line, b: &Line| {
        let (Line::Item(ao, ae), Line::Item(bo, be)) = (a, b) else {
            return std::cmp::Ordering::Equal;
        };
        let a = &offers[*ao].entries[*ae];
        let b = &offers[*bo].entries[*be];
        let order = match sort.col {
            Col::Name => a.relative.to_lowercase().cmp(&b.relative.to_lowercase()),
            Col::Status => a.class.label().cmp(b.class.label()),
            Col::Procs => a.candidate.file_count.cmp(&b.candidate.file_count),
            Col::Safety => a.candidate.safety.level.cmp(&b.candidate.safety.level),
            _ => bytes(&a.candidate).cmp(&bytes(&b.candidate)),
        }
        .then(a.candidate.path.cmp(&b.candidate.path));
        if sort.desc { order.reverse() } else { order }
    };
    if lines.iter().all(|line| matches!(line, Line::Item(_, _))) {
        let mut lines = lines;
        lines.sort_by(compare_items);
        return lines;
    }
    let mut groups: Vec<Vec<Line>> = Vec::new();
    for line in lines {
        if matches!(line, Line::Group { .. }) {
            groups.push(vec![line]);
        } else if let Some(group) = groups.last_mut() {
            group.push(line);
        }
    }
    for group in &mut groups {
        group[1..].sort_by(compare_items);
    }
    groups.sort_by(|a, b| {
        let (
            Line::Group {
                title: an,
                bytes: ab,
                files: af,
                ..
            },
            Line::Group {
                title: bn,
                bytes: bb,
                files: bf,
                ..
            },
        ) = (&a[0], &b[0])
        else {
            return std::cmp::Ordering::Equal;
        };
        let order = match sort.col {
            Col::Size => ab.cmp(bb),
            Col::Procs => af.cmp(bf),
            _ => an.to_lowercase().cmp(&bn.to_lowercase()),
        }
        .then(an.cmp(bn));
        if sort.desc { order.reverse() } else { order }
    });
    groups.into_iter().flatten().collect()
}

fn tool_name(offer: &AiOffer) -> &str {
    match &offer.candidate.owner {
        sweeploom_core::CandidateOwner::Application(name) => name.as_str(),
        _ => "AI",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sweeploom_ai::inspect_offers;
    use sweeploom_platform::UserLocations;

    #[test]
    fn tool_groups_keep_every_child() {
        let mut offers = inspect_offers(&UserLocations::current());
        if offers.is_empty() {
            return;
        }
        let children: usize = offers.iter().map(|item| item.entries.len()).sum();
        let lines = table_lines(&mut offers, Sort::size_desc(), AiGroup::Tool);
        let items = lines
            .iter()
            .filter(|line| matches!(line, Line::Item(_, _)))
            .count();
        assert_eq!(items, children);
        let collapsed = HashSet::from([format!("tool:0:{}", tool_name(&offers[0]))]);
        let visible = visible_lines(&lines, &collapsed);
        assert!(
            visible
                .iter()
                .filter(|line| matches!(line, Line::Item(_, _)))
                .count()
                < items
                || items == 0
        );
    }
}
