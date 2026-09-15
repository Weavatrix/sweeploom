//! Unique reclaim accounting. Overlapping offers must not be summed twice.

use std::path::{Path, PathBuf};

use crate::candidate::Candidate;
use crate::plan::DeletionStrategy;

/// One measured size with honesty about coverage.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SizeMeasurement {
    /// Visited logical bytes.
    pub bytes: u64,
    /// Files visited.
    pub files: u32,
    /// Read errors.
    pub errors: u32,
    /// False when a cap or error stopped the walk.
    pub complete: bool,
}

impl SizeMeasurement {
    /// Exact measurement of a fully visited tree.
    #[must_use]
    pub const fn exact(bytes: u64, files: u32) -> Self {
        Self {
            bytes,
            files,
            errors: 0,
            complete: true,
        }
    }

    /// Lower bound. UI must render this as `>=`.
    #[must_use]
    pub const fn lower_bound(bytes: u64, files: u32, errors: u32) -> Self {
        Self {
            bytes,
            files,
            errors,
            complete: false,
        }
    }
}

/// Alternative cleanup group (Cargo Light/Balanced/Full share one root).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReclaimGroup {
    /// Paths that are alternatives of each other; at most one should execute.
    pub alternatives: Vec<PathBuf>,
    /// Unique executable estimate for this group.
    pub unique: SizeMeasurement,
}

/// Deduplicated reclaim set used by GUI, CLI, Overview, and receipts.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReclaimSet {
    /// Alternative groups.
    pub groups: Vec<ReclaimGroup>,
    /// Unique executable estimate across all groups.
    pub unique_estimate: SizeMeasurement,
    /// Inspect-only or otherwise non-executable rows.
    pub rejected: usize,
}

/// Build a unique reclaim set from selected candidates.
#[must_use]
pub fn reclaim_set(candidates: &[Candidate]) -> ReclaimSet {
    let executable: Vec<&Candidate> = candidates
        .iter()
        .filter(|item| !item.safety.is_blocked() && item.deletion != DeletionStrategy::InspectOnly)
        .collect();
    let rejected = candidates.len().saturating_sub(executable.len());
    let unique_paths = unique_roots(executable.iter().map(|item| item.path.as_path()));
    let mut bytes = 0_u64;
    let mut files = 0_u32;
    let mut complete = true;
    for candidate in &executable {
        if unique_paths.iter().any(|path| path == &candidate.path) {
            bytes = bytes.saturating_add(candidate.logical_bytes);
            files = files.saturating_add(u32::try_from(candidate.file_count).unwrap_or(u32::MAX));
        }
    }
    if unique_paths.len() < executable.len() {
        complete = true;
    }
    ReclaimSet {
        groups: vec![ReclaimGroup {
            alternatives: unique_paths,
            unique: SizeMeasurement {
                bytes,
                files,
                errors: 0,
                complete,
            },
        }],
        unique_estimate: SizeMeasurement {
            bytes,
            files,
            errors: 0,
            complete,
        },
        rejected,
    }
}

/// Keep only roots that are not contained by another selected path.
#[must_use]
pub fn unique_roots<'a>(paths: impl IntoIterator<Item = &'a Path>) -> Vec<PathBuf> {
    let mut items: Vec<PathBuf> = paths.into_iter().map(Path::to_path_buf).collect();
    items.sort();
    items.dedup();
    let mut kept = Vec::new();
    for path in items {
        if kept.iter().any(|parent: &PathBuf| path.starts_with(parent)) {
            continue;
        }
        kept.retain(|child| !child.starts_with(&path));
        kept.push(path);
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::CandidateId;
    use crate::rebuild::RebuildAssessment;
    use crate::safety::SafetyAssessment;
    use crate::{ActivityEvidence, CandidateKind, CandidateOwner, UserPolicy};

    fn candidate(path: &str, bytes: u64, deletion: DeletionStrategy) -> Candidate {
        Candidate {
            id: CandidateId(1),
            kind: CandidateKind::BuildArtifact,
            owner: CandidateOwner::User,
            path: PathBuf::from(path),
            logical_bytes: bytes,
            allocated_bytes: None,
            file_count: 1,
            activity: ActivityEvidence::default(),
            safety: SafetyAssessment::safe(),
            rebuild: RebuildAssessment::default(),
            deletion,
            evidence: Vec::new(),
            user_policy: UserPolicy::Default,
        }
    }

    #[test]
    fn nested_cargo_offers_count_once() {
        let set = reclaim_set(&[
            candidate("/proj/target", 20, DeletionStrategy::PermanentGenerated),
            candidate(
                "/proj/target/debug",
                15,
                DeletionStrategy::PermanentGenerated,
            ),
            candidate(
                "/proj/target/debug/incremental",
                4,
                DeletionStrategy::PermanentGenerated,
            ),
        ]);
        assert_eq!(set.unique_estimate.bytes, 20);
    }

    #[test]
    fn inspect_only_is_not_executable_reclaim() {
        let set = reclaim_set(&[candidate("/proj/store", 99, DeletionStrategy::InspectOnly)]);
        assert_eq!(set.unique_estimate.bytes, 0);
        assert_eq!(set.rejected, 1);
    }
}
