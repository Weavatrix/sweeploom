//! Safety is independent of recommendation. Blockers always win.

/// How safe a destructive action is, given current evidence.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SafetyLevel {
    /// Regenerable generated/cache data with no blockers.
    Safe,
    /// Likely regenerable, but with residual uncertainty.
    LowRisk,
    /// User data or ambiguous content. Never auto-selected.
    Review,
    /// High chance of losing work.
    Dangerous,
    /// Must not be selected without removing the blocker.
    Blocked,
}

/// Why a candidate cannot be auto-acted on.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Blocker {
    /// A live process has cwd or a command path inside the candidate.
    ActiveProcess,
    /// Filesystem write inside the safety window.
    RecentWrite,
    /// Git reports dirty tracked files.
    DirtyTrackedFiles,
    /// Untracked user files are present.
    UntrackedFiles,
    /// Git state could not be determined.
    UnknownGitState,
    /// Candidate path escaped through a symlink.
    SymlinkEscape,
    /// Windows reparse point escape.
    ReparsePointEscape,
    /// Permission boundary would be crossed.
    PermissionBoundary,
    /// Candidate sits on another filesystem than the scan root.
    MountedFilesystemBoundary,
    /// Path, identity, or size changed after the plan was built.
    CandidateChangedAfterPlan,
    /// Explicitly protected path.
    ProtectedPath,
    /// User pinned the candidate or project.
    UserPinned,
    /// Shared build directory currently in use.
    SharedBuildDirectoryInUse,
    /// Kernel, desktop shell, security software, or unknown elevated OS process.
    SystemCriticalProcess,
}

/// Non-blocking caution attached to a candidate or session.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Warning {
    /// High rebuild cost if deleted.
    HighRebuildCost,
    /// Estimated reclaim may be smaller than RSS/logical size.
    SharedMemoryOrHardlinks,
    /// Network was recently active.
    RecentNetworkActivity,
    /// Project has uncommitted changes; termination is still allowed.
    DirtyGitWorktree,
    /// Exact byte accounting is unavailable on this platform/capability.
    NetworkRateUnavailable,
}

/// Combined safety verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SafetyAssessment {
    /// Final level after applying blockers.
    pub level: SafetyLevel,
    /// Hard blockers. Non-empty implies [`SafetyLevel::Blocked`].
    pub blockers: Vec<Blocker>,
    /// Soft warnings.
    pub warnings: Vec<Warning>,
    /// How confident the assessment is.
    pub confidence: crate::evidence::Confidence,
}

impl SafetyAssessment {
    /// Construct a blocked assessment.
    #[must_use]
    pub fn blocked(blocker: Blocker) -> Self {
        Self {
            level: SafetyLevel::Blocked,
            blockers: vec![blocker],
            warnings: Vec::new(),
            confidence: crate::evidence::Confidence::Exact,
        }
    }

    /// Construct a safe assessment with no blockers.
    #[must_use]
    pub fn safe() -> Self {
        Self {
            level: SafetyLevel::Safe,
            blockers: Vec::new(),
            warnings: Vec::new(),
            confidence: crate::evidence::Confidence::Exact,
        }
    }

    /// User data / ambiguous content. Never auto-selected.
    #[must_use]
    pub fn review() -> Self {
        Self {
            level: SafetyLevel::Review,
            blockers: Vec::new(),
            warnings: Vec::new(),
            confidence: crate::evidence::Confidence::Strong,
        }
    }

    /// True when the candidate may not be auto-selected.
    #[must_use]
    pub fn is_blocked(&self) -> bool {
        self.level == SafetyLevel::Blocked || !self.blockers.is_empty()
    }

    /// Recompute level from blockers. Recommendation must call this, never invent a bypass.
    #[must_use]
    pub fn with_normalized_level(mut self) -> Self {
        if !self.blockers.is_empty() {
            self.level = SafetyLevel::Blocked;
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blockers_force_blocked_level() {
        let assessment = SafetyAssessment {
            level: SafetyLevel::Safe,
            blockers: vec![Blocker::ActiveProcess],
            warnings: Vec::new(),
            confidence: crate::evidence::Confidence::Strong,
        }
        .with_normalized_level();
        assert_eq!(assessment.level, SafetyLevel::Blocked);
        assert!(assessment.is_blocked());
    }
}
