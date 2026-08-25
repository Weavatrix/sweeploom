//! Disk cleanup candidate contract.

use std::path::PathBuf;

use crate::activity::ActivityEvidence;
use crate::evidence::Evidence;
use crate::ids::{CandidateId, ProjectId, ToolId};
use crate::plan::DeletionStrategy;
use crate::policy::UserPolicy;
use crate::rebuild::RebuildAssessment;
use crate::safety::SafetyAssessment;

/// What kind of reclaimable object this is.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CandidateKind {
    /// `target`, `build`, DerivedData, and similar.
    BuildArtifact,
    /// `node_modules`, `.venv`, and similar.
    DependencyTree,
    /// Package-manager caches.
    PackageCache,
    /// Tool caches such as Cargo home.
    ToolCache,
    /// IDE caches.
    IdeCache,
    /// AI/agent session store.
    AiSession,
    /// AI/agent cache.
    AiCache,
    /// Agent worktree.
    AgentWorktree,
    /// Temporary file.
    TempFile,
    /// Temporary directory.
    TempDirectory,
    /// Crash dump.
    CrashDump,
    /// Log file or directory.
    Log,
    /// Browser cache.
    BrowserCache,
    /// Old installer in Downloads.
    OldInstaller,
    /// Old archive.
    OldArchive,
    /// Large file inspection.
    LargeFile,
    /// Empty directory.
    EmptyDirectory,
    /// Duplicate file group member.
    DuplicateFile,
    /// Container resource.
    ContainerResource,
    /// Simulator data.
    SimulatorData,
    /// ML model cache.
    MlModelCache,
}

/// Who owns a candidate.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CandidateOwner {
    /// A discovered project.
    Project(ProjectId),
    /// A tool such as Cargo or npm.
    Tool(ToolId),
    /// A named application.
    Application(String),
    /// Operating system.
    System,
    /// User-owned location such as Downloads.
    User,
}

/// One reclaimable item with evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Candidate {
    /// Snapshot-local identifier.
    pub id: CandidateId,
    /// Kind.
    pub kind: CandidateKind,
    /// Owner.
    pub owner: CandidateOwner,
    /// Canonical path. Never follow a symlink blindly to produce this.
    pub path: PathBuf,
    /// Sum of `metadata.len()` values.
    pub logical_bytes: u64,
    /// Allocated size when the platform can report it.
    pub allocated_bytes: Option<u64>,
    /// Number of files under the path.
    pub file_count: u64,
    /// Activity evidence.
    pub activity: ActivityEvidence,
    /// Safety assessment. Independent of recommendation.
    pub safety: SafetyAssessment,
    /// Rebuild cost.
    pub rebuild: RebuildAssessment,
    /// How deletion should proceed if approved.
    pub deletion: DeletionStrategy,
    /// Inspectable reasons.
    pub evidence: Vec<Evidence>,
    /// User policy.
    pub user_policy: UserPolicy,
}

impl Candidate {
    /// Best estimate of bytes the user might actually get back.
    #[must_use]
    pub fn estimated_reclaimable_bytes(&self) -> u64 {
        self.allocated_bytes.unwrap_or(self.logical_bytes)
    }
}
