//! Immutable cleanup plan, execution report, and receipt.

use std::path::PathBuf;
use std::time::SystemTime;

use crate::artifact::{FileIdentity, MetadataRevision};
use crate::ids::{CandidateId, PlanId};
use crate::live::ProcessSnapshot;
use crate::safety::Blocker;

/// How a candidate should be removed if the user approves it.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DeletionStrategy {
    /// Permanent delete of regenerable generated data.
    PermanentGenerated,
    /// OS trash / recycle bin. May not free space on the same volume.
    Trash,
    /// Native tool API (docker prune, cargo, ...).
    NativeTool,
    /// Archive instead of delete.
    Archive,
    /// Truncate (logs).
    Truncate,
    /// Inspection only — never delete.
    InspectOnly,
}

/// A safety check that must still hold at apply time.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SafetyPrecondition {
    /// Path must still exist as a directory or file of the planned kind.
    PathKindUnchanged,
    /// Native file identity must match.
    FileIdentityMatch,
    /// No new writes after the planned timestamp.
    NoNewerWrites,
    /// No live process using the path.
    NoActiveProcess,
    /// Git state still matches the planned snapshot.
    GitStateUnchanged,
    /// Must not be a symlink/reparse escape.
    NoSymlinkEscape,
}

/// Why an entry was skipped at apply time.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SkipReason {
    /// Candidate changed after the plan was built.
    Changed,
    /// Newly blocked.
    Blocked(Blocker),
    /// User cancelled.
    Cancelled,
    /// Missing at apply time.
    Missing,
    /// Required evidence could not be obtained.
    UnknownEvidence,
    /// Strategy or platform action is unsupported.
    Unsupported,
}

/// One immutable plan entry.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CleanPlanEntry {
    /// Candidate id from the originating snapshot.
    pub candidate_id: CandidateId,
    /// Planned path.
    pub path: PathBuf,
    /// Owning project root when known. Used to repeat artifact authorization.
    pub owner_path: Option<PathBuf>,
    /// Optional native identity (volume + file id).
    pub expected_identity: Option<FileIdentity>,
    /// Directory/file revision captured at plan time.
    pub expected_revision: Option<MetadataRevision>,
    /// True when the planned path was a directory.
    pub expected_is_dir: bool,
    /// Newest write observed at plan time.
    pub expected_latest_write: Option<SystemTime>,
    /// Expected logical bytes.
    pub expected_bytes: u64,
    /// Deletion strategy.
    pub strategy: DeletionStrategy,
    /// Preconditions that must hold at apply time.
    pub required_safety: Vec<SafetyPrecondition>,
}

/// Immutable cleanup plan.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CleanPlan {
    /// Schema version.
    pub version: u32,
    /// Plan id.
    pub id: PlanId,
    /// Creation time.
    pub created_at: SystemTime,
    /// Entries.
    pub entries: Vec<CleanPlanEntry>,
    /// Optional "free at least N bytes" request.
    pub requested_free_bytes: Option<u64>,
    /// Estimated reclaimable bytes at plan time.
    pub estimated_reclaimable_bytes: u64,
}

impl CleanPlan {
    /// Current plan schema.
    pub const VERSION: u32 = 2;
}

/// How process evidence was obtained. An empty observed list is not "unknown".
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProcessEvidence<'a> {
    /// Caller did not sample processes.
    Unknown,
    /// Caller sampled; the slice may be empty.
    Observed(&'a [ProcessSnapshot]),
}

/// Shared apply context for GUI and CLI.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExecutionContext<'a> {
    /// Process evidence. [`ProcessEvidence::Unknown`] is not "no processes".
    pub processes: ProcessEvidence<'a>,
}

impl ExecutionContext<'_> {
    /// No process sample is available.
    #[must_use]
    pub const fn unknown() -> Self {
        Self {
            processes: ProcessEvidence::Unknown,
        }
    }

    /// An observed sample, including an empty machine.
    #[must_use]
    pub const fn observed(processes: &[ProcessSnapshot]) -> ExecutionContext<'_> {
        ExecutionContext {
            processes: ProcessEvidence::Observed(processes),
        }
    }
}

/// Counts written to a receipt.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReceiptCounts {
    /// Successfully deleted entries.
    pub deleted: u64,
    /// Skipped because the candidate changed.
    pub skipped_changed: u64,
    /// Failed entries.
    pub failed: u64,
}

/// Post-execution receipt. Bounded history stores these, not full file lists.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Receipt {
    /// Plan id.
    pub plan: PlanId,
    /// Start.
    pub started: SystemTime,
    /// Finish.
    pub finished: SystemTime,
    /// Selected logical bytes.
    pub selected_logical_bytes: u64,
    /// Estimated physical bytes.
    pub estimated_physical_bytes: u64,
    /// Measured free-space delta when a volume snapshot was taken. `None` is unavailable.
    pub actual_free_space_delta: Option<i64>,
    /// Counts.
    pub counts: ReceiptCounts,
}

/// In-memory execution report used before a receipt is sealed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExecutionReport {
    /// Counts.
    pub counts: ReceiptCounts,
    /// Skip details.
    pub skipped: Vec<(CandidateId, SkipReason)>,
    /// Failed paths with a message (already free of secrets).
    pub failures: Vec<(CandidateId, String)>,
}

impl DeletionStrategy {
    /// Short UI label. Avoids Debug in the Rules screen.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::PermanentGenerated => "Delete generated",
            Self::Trash => "Trash",
            Self::NativeTool => "Native tool",
            Self::Archive => "Archive",
            Self::Truncate => "Truncate",
            Self::InspectOnly => "Inspect only",
        }
    }
}
