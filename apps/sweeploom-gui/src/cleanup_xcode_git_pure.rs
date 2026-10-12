//! App Store edition Git probe: read `.git` with `weavatrix-git`, no `git` process.
use super::super::verdict::GitState;
use std::path::Path;
use weavatrix_git::{Repository, WorktreeEvidence, WorktreeSafetyLevel};

/// Branch, last commit time and an uncommitted-change count from the repository files.
pub(super) fn state(root: &Path) -> GitState {
    let repository = Repository::open(root).ok();
    let last_commit = repository.as_ref().and_then(|repo| {
        let head = repo.resolve("HEAD").ok()?;
        repo.commit_metadata(head)
            .ok()
            .map(|commit| commit.committer_time)
    });
    let dirty = repository
        .as_ref()
        .and_then(|repo| repo.worktree_safety().ok())
        .and_then(|safety| match safety.level {
            WorktreeSafetyLevel::Clean | WorktreeSafetyLevel::IgnoredOnly => Some(0),
            WorktreeSafetyLevel::Unknown => None,
            WorktreeSafetyLevel::HasUntracked | WorktreeSafetyLevel::DirtyTracked => {
                let tracked = safety
                    .evidence
                    .iter()
                    .filter(|item| matches!(item, WorktreeEvidence::TrackedDirty { .. }))
                    .count();
                let untracked = usize::try_from(safety.untracked_count).unwrap_or(usize::MAX);
                Some(tracked.saturating_add(untracked).max(1))
            }
        });
    GitState {
        branch: super::branch(root),
        last_commit,
        dirty,
    }
}
