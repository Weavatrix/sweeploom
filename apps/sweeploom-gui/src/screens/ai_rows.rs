//! AI store table rows. Grouping does not walk the disk.

use std::collections::{BTreeMap, HashSet};

use sweeploom_ai::{AiClass, AiOffer};
use sweeploom_core::DeletionStrategy;

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
    flatten(offers, group)
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

/// True when this child may be checked for an explicit clean.
#[must_use]
pub fn can_clean(offer: &AiOffer, entry: usize) -> bool {
    offer.entries.get(entry).is_some_and(|item| {
        item.class.can_clean()
            && item.candidate.deletion != DeletionStrategy::InspectOnly
            && !item.candidate.safety.is_blocked()
    })
}

fn sort_entries(offers: &mut [AiOffer], sort: Sort) {
    for offer in offers.iter_mut() {
        offer.entries.sort_by(|left, right| match sort.col {
            Col::Name => left.relative.cmp(&right.relative),
            Col::Status | Col::Procs => left.class.label().cmp(right.class.label()),
            Col::Size | Col::Cpu => left
                .candidate
                .logical_bytes
                .cmp(&right.candidate.logical_bytes),
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
            bytes: offer.candidate.logical_bytes,
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
            .map(|&(offer, entry)| offers[offer].entries[entry].candidate.logical_bytes)
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
