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
        Err(_) => classify_open_error(path),
    }
}

fn classify_open_error(path: &Path) -> GitSafety {
    if ancestor_has_git(path) {
        GitSafety::Unknown
    } else {
        GitSafety::NotARepository
    }
}

fn ancestor_has_git(path: &Path) -> bool {
    let mut current = Some(path);
    for _ in 0..24 {
        let Some(dir) = current else {
            break;
        };
        if dir.join(".git").exists() {
            return true;
        }
        current = dir.parent();
    }
    false
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

    /// True when generated cleanup may be auto-selected.
    #[must_use]
    pub fn allows_generated_cleanup(&self) -> bool {
        !self.assessment().is_blocked()
    }

    /// Short Git state for UI/CLI. Does not leak Weavatrix types to the shell.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::NotARepository => "none",
            Self::Unknown => "unknown",
            Self::Known(safety) => match safety.level {
                WorktreeSafetyLevel::Clean => "clean",
                WorktreeSafetyLevel::IgnoredOnly => "ignored-only",
                WorktreeSafetyLevel::HasUntracked => "untracked",
                WorktreeSafetyLevel::DirtyTracked => "dirty",
                WorktreeSafetyLevel::Unknown => "unknown",
            },
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

    #[test]
    fn git_dir_with_unreadable_repo_is_unknown_not_safe() {
        let root = std::env::temp_dir().join(format!("sweeploom-git-unk-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".git")).unwrap();
        std::fs::write(root.join(".git").join("HEAD"), "not-a-repo").unwrap();
        let git = inspect(&root);
        std::fs::remove_dir_all(&root).ok();
        assert!(matches!(git, GitSafety::Unknown | GitSafety::Known(_)));
        if matches!(git, GitSafety::Unknown) {
            assert!(git.assessment().is_blocked());
            assert_ne!(git.label(), "none");
        }
    }
}
