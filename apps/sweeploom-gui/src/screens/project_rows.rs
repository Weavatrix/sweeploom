//! Project table rows. Grouping does not walk the disk.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use sweeploom_core::CandidateOwner;
use sweeploom_dev::DevKind;
use sweeploom_storage::DiskUsage;

use crate::app::SweepLoomApp;
use crate::format::row_caption;
use crate::sort::{Col, Sort};

use super::project_facts::{
    Acc, Bit, artifact_label, cluster_parent, cluster_title, reclaimable_bytes,
};

/// How Projects cluster rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectGroup {
    /// Cargo / Node / Python.
    Kind,
    /// Immediate parent folder.
    Parent,
    /// Flat sortable list.
    None,
}

impl ProjectGroup {
    /// Toolbar label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Kind => "Kind",
            Self::Parent => "Folder",
            Self::None => "None",
        }
    }
}

#[derive(Clone)]
pub(crate) struct ProjectCard {
    pub path: PathBuf,
    pub bytes: Option<DiskUsage>,
    pub size_error: Option<String>,
    pub artifact_bytes: u64,
    pub kinds: String,
    pub group_kind: &'static str,
    pub folder: String,
    pub folder_key: String,
    pub artifacts: String,
}

pub(crate) enum Line {
    Group {
        key: String,
        title: String,
        count: usize,
        bytes: Option<DiskUsage>,
        expanded: bool,
    },
    Project(usize),
}

/// Rebuild cards only when inputs change. Marker facts come from a cache, so a
/// size arriving does not re-read every manifest on the UI thread.
pub(crate) fn refresh_cards(app: &mut SweepLoomApp) {
    let stamp = card_stamp(app);
    if stamp == app.project_card_stamp {
        return;
    }
    let paths = collect_paths(app);
    let accs = collect_accs(app);
    let mut cards = Vec::with_capacity(accs.len());
    for acc in accs {
        let facts = app.project_sizes.facts(&acc.path);
        cards.push(ProjectCard::from_acc(acc, facts, &app.project_sizes, &paths));
    }
    app.project_cards = cards;
    app.project_card_stamp = stamp;
}

fn card_stamp(app: &SweepLoomApp) -> u64 {
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    app.project_roots.hash(&mut hash);
    if let Some(report) = &app.inventory {
        report.projects.hash(&mut hash);
    }
    for row in &app.review {
        row.candidate.path.hash(&mut hash);
        row.candidate.logical_bytes.hash(&mut hash);
        row.title.hash(&mut hash);
    }
    app.project_sizes.revision().hash(&mut hash);
    hash.finish()
}

fn collect_accs(app: &SweepLoomApp) -> Vec<Acc> {
    let mut map: BTreeMap<PathBuf, Acc> = BTreeMap::new();
    for path in &app.project_roots {
        map.entry(path.clone()).or_insert_with(|| Acc::new(path));
    }
    if let Some(report) = &app.inventory {
        for path in &report.projects {
            map.entry(path.clone()).or_insert_with(|| Acc::new(path));
        }
    }
    for row in &app.review {
        let CandidateOwner::Project(id) = &row.candidate.owner else {
            continue;
        };
        let acc = map.entry(id.0.clone()).or_insert_with(|| Acc::new(&id.0));
        acc.bits.push(Bit {
            path: row.candidate.path.clone(),
            bytes: row.candidate.logical_bytes,
            title: row_caption(&row.title),
        });
    }
    map.into_values().collect()
}

fn collect_paths(app: &SweepLoomApp) -> HashSet<PathBuf> {
    let mut paths: HashSet<PathBuf> = app.project_roots.iter().cloned().collect();
    if let Some(report) = &app.inventory {
        paths.extend(report.projects.iter().cloned());
    }
    for row in &app.review {
        if let CandidateOwner::Project(id) = &row.candidate.owner {
            paths.insert(id.0.clone());
        }
    }
    paths
}

pub(crate) fn sort_cards(cards: &mut [ProjectCard], sort: Sort) {
    cards.sort_by(|left, right| {
        if sort.col == Col::Size {
            return size_order(left.bytes, right.bytes, sort.desc).then(left.path.cmp(&right.path));
        }
        let order = match sort.col {
            Col::Name => left
                .name()
                .to_lowercase()
                .cmp(&right.name().to_lowercase())
                .then(left.path.cmp(&right.path)),
            Col::Status => left
                .kinds
                .cmp(&right.kinds)
                .then(left.path.cmp(&right.path)),
            _ => left.path.cmp(&right.path),
        };
        if sort.desc { order.reverse() } else { order }
    });
}

fn size_order(left: Option<DiskUsage>, right: Option<DiskUsage>, desc: bool) -> std::cmp::Ordering {
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

impl ProjectCard {
    pub(crate) fn name(&self) -> &str {
        self.path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("project")
    }

    fn group_key(&self, group: ProjectGroup) -> String {
        match group {
            ProjectGroup::Kind => self.group_kind.to_owned(),
            ProjectGroup::Parent => self.folder_key.clone(),
            ProjectGroup::None => String::new(),
        }
    }

    pub(super) fn from_acc(
        acc: Acc,
        facts: crate::project_sizes::ProjectFacts,
        sizes: &crate::project_sizes::ProjectSizes,
        projects: &HashSet<PathBuf>,
    ) -> Self {
        let group_kind = facts.kinds.first().copied().unwrap_or(DevKind::Other).label();
        let labels = facts
            .kinds
            .iter()
            .map(|kind| kind.label())
            .collect::<Vec<_>>()
            .join(", ");
        let parent = cluster_parent(&acc.path, projects);
        let (bytes, size_error) = match sizes.get(&acc.path) {
            Some(Ok(usage)) => (Some(*usage), None),
            Some(Err(error)) => (None, Some(error.clone())),
            None => (None, None),
        };
        Self {
            folder: cluster_title(&parent),
            folder_key: parent.to_string_lossy().into_owned(),
            size_error,
            artifact_bytes: reclaimable_bytes(&acc.bits),
            artifacts: artifact_label(facts.cargo, &acc.bits),
            path: acc.path,
            bytes,
            kinds: labels,
            group_kind,
        }
    }
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
    total.complete = roots.iter().all(|card| card.bytes.is_some_and(|u| u.complete));
    if found {
        return Some(total);
    }
    roots.iter().all(|card| card.size_error.is_some()).then(|| {
        total.errors = roots.len().max(1) as u64;
        total
    })
}

pub(crate) fn size_caption(usage: Option<DiskUsage>, error: Option<&str>) -> String {
    match usage {
        Some(u) if u.complete => crate::format::format_bytes(u.bytes),
        Some(u) if u.bytes > 0 => format!(">= {}", crate::format::format_bytes(u.bytes)),
        Some(u) if u.errors > 0 => "Unavailable".into(),
        None if error.is_some() => "Unavailable".into(),
        _ => "Measuring…".into(),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifact_size_never_substitutes_for_unmeasured_project_size() {
        let path = PathBuf::from("/project-not-scanned");
        let mut acc = Acc::new(&path);
        acc.bits.push(Bit {
            path: path.join(".vite"),
            bytes: 22_000,
            title: "cache".into(),
        });
        let facts = crate::project_sizes::ProjectFacts {
            kinds: vec![DevKind::Other],
            cargo: None,
        };
        let card =
            ProjectCard::from_acc(acc, facts, &Default::default(), &HashSet::from([path]));
        assert_eq!(card.bytes, None);
        assert_eq!(card.artifact_bytes, 22_000);
        assert_eq!(
            size_caption(card.bytes, card.size_error.as_deref()),
            "Measuring…"
        );
    }
}
