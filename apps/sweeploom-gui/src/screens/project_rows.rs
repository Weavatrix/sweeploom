//! Project table rows. Grouping does not walk the disk.

use std::collections::{BTreeMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use sweeploom_core::CandidateOwner;
use sweeploom_dev::DevKind;
use sweeploom_storage::DiskUsage;

use crate::app::SweepLoomApp;
use crate::format::row_caption;
use crate::sort::{Col, Sort};

#[path = "project_groups.rs"]
mod groups;
use groups::size_order;
pub(crate) use groups::table_lines;

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
        cards.push(ProjectCard::from_acc(
            acc,
            facts,
            &app.project_sizes,
            &paths,
        ));
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
        let group_kind = facts
            .kinds
            .first()
            .copied()
            .unwrap_or(DevKind::Other)
            .label();
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

pub(crate) fn size_caption(usage: Option<DiskUsage>, error: Option<&str>) -> String {
    match usage {
        Some(u) if u.complete => crate::format::format_bytes(u.bytes),
        Some(u) if u.bytes > 0 => format!(">= {}", crate::format::format_bytes(u.bytes)),
        Some(u) if u.errors > 0 => "Unavailable".into(),
        None if error.is_some() => "Unavailable".into(),
        _ => "Measuring…".into(),
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
        let card = ProjectCard::from_acc(acc, facts, &Default::default(), &HashSet::from([path]));
        assert_eq!(card.bytes, None);
        assert_eq!(card.artifact_bytes, 22_000);
        assert_eq!(
            size_caption(card.bytes, card.size_error.as_deref()),
            "Measuring…"
        );
    }
}
