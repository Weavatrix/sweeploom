//! Git safety via `weavatrix-git`. SweepLoom does not reimplement Git.

use std::path::Path;

use sweeploom_core::{Blocker, Confidence, SafetyAssessment, Warning};
use weavatrix_git::{Repository, WorktreeSafety, WorktreeSafetyLevel};

/// Inspect a path that may belong to a Git worktree.
#[must_use]
pub fn inspect(path: &Path) -> GitSafety {
    match Repository::open(path) {
        Ok(repository) => match repository.worktree_safety() {
            Ok(safety) => GitSafety::Known(safety),
            Err(_) => GitSafety::Unknown,
        },
        Err(_) => GitSafety::NotARepository,
    }
}

/// Result of a Git safety probe.
#[derive(Clone, Debug)]
pub enum GitSafety {
    /// Path is not inside a Git repository.
    NotARepository,
    /// Repository could not be classified.
    Unknown,
    /// Weavatrix Git produced a safety summary.
    Known(WorktreeSafety),
}

impl GitSafety {
    /// Map Git evidence onto SweepLoom blockers. Ignored-only is not dirty.
    #[must_use]
    pub fn assessment(&self) -> SafetyAssessment {
        match self {
            Self::NotARepository => SafetyAssessment::safe(),
            Self::Unknown => SafetyAssessment::blocked(Blocker::UnknownGitState),
            Self::Known(safety) => assessment_from_safety(safety),
        }
    }
}

fn assessment_from_safety(safety: &WorktreeSafety) -> SafetyAssessment {
    match safety.level {
        WorktreeSafetyLevel::Clean | WorktreeSafetyLevel::IgnoredOnly => {
            let mut assessment = SafetyAssessment::safe();
            if safety.submodule_unknown {
                assessment.warnings.push(Warning::DirtyGitWorktree);
                assessment.confidence = Confidence::Strong;
            }
            assessment
        }
        WorktreeSafetyLevel::HasUntracked => SafetyAssessment::blocked(Blocker::UntrackedFiles),
        WorktreeSafetyLevel::DirtyTracked => SafetyAssessment::blocked(Blocker::DirtyTrackedFiles),
        WorktreeSafetyLevel::Unknown => SafetyAssessment::blocked(Blocker::UnknownGitState),
    }
}

impl GitSafety {
    /// True when generated cleanup may be auto-selected.
    #[must_use]
    pub fn allows_generated_cleanup(&self) -> bool {
        !self.assessment().is_blocked()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use weavatrix_git::WorktreeKind;

    #[test]
    fn ignored_only_does_not_block() {
        let safety = WorktreeSafety {
            tracked_dirty: false,
            staged_dirty: false,
            untracked_count: 0,
            ignored_count: 3,
            submodule_unknown: false,
            kind: WorktreeKind::Primary,
            level: WorktreeSafetyLevel::IgnoredOnly,
            evidence: Vec::new(),
        };
        let git = GitSafety::Known(safety);
        assert!(git.allows_generated_cleanup());
        assert!(!git.assessment().is_blocked());
    }

    #[test]
    fn untracked_blocks_auto_cleanup() {
        let safety = WorktreeSafety {
            tracked_dirty: false,
            staged_dirty: false,
            untracked_count: 1,
            ignored_count: 0,
            submodule_unknown: false,
            kind: WorktreeKind::Primary,
            level: WorktreeSafetyLevel::HasUntracked,
            evidence: Vec::new(),
        };
        assert!(!GitSafety::Known(safety).allows_generated_cleanup());
    }
}
