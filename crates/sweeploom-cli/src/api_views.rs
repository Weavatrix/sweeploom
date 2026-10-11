//! Serializable rows returned by the CLI / MCP operations.

use std::path::PathBuf;

use serde::Serialize;

/// Live session row. Command lines are omitted on purpose.
#[derive(Clone, Debug, Serialize)]
pub struct SessionView {
    /// Session kind label.
    pub kind: String,
    /// Process count.
    pub processes: usize,
    /// Combined RSS.
    pub rss_bytes: u64,
    /// Combined CPU percent.
    pub cpu_percent: f32,
    /// Recommendation label.
    pub recommendation: String,
    /// Activity label.
    pub activity: String,
    /// Project path, if attributed.
    pub project: Option<String>,
    /// True when a RAM/CPU/quiet plan would include this session (dry-run).
    pub planned: bool,
}

/// Disk inventory child.
#[derive(Clone, Debug, Serialize)]
pub struct DiskChild {
    /// Path.
    pub path: String,
    /// Logical bytes.
    pub logical_bytes: u64,
    /// Category.
    pub category: String,
}

/// Disk inventory summary.
#[derive(Clone, Debug, Serialize)]
pub struct DiskInventory {
    /// Scan root.
    pub root: String,
    /// Entries visited.
    pub entries: u64,
    /// Project count.
    pub projects: usize,
    /// Walk hit a cap.
    pub capped: bool,
    /// Root logical bytes.
    pub logical_bytes: u64,
    /// First children.
    pub children: Vec<DiskChild>,
}

/// Project heat + git safety.
#[derive(Clone, Debug, Serialize)]
pub struct ProjectView {
    /// Project path.
    pub path: String,
    /// Detected kind.
    pub kind: String,
    /// Source heat.
    pub source: String,
    /// Artifact heat.
    pub artifact: String,
    /// Git safety label.
    pub git: String,
}

/// AI store child.
#[derive(Clone, Debug, Serialize)]
pub struct AiChildView {
    /// Candidate id.
    pub id: u64,
    /// Class label.
    pub class: String,
    /// Path relative to the store.
    pub relative: String,
    /// Logical bytes.
    pub logical_bytes: u64,
    /// File count.
    pub file_count: u64,
    /// Always-on token estimate.
    pub prompt_tokens: u64,
    /// Context advice (`keep` / `park?` / empty).
    pub advice: String,
    /// Whether this child may be deleted after an explicit apply.
    pub can_clean: bool,
}

/// AI store listing.
#[derive(Clone, Debug, Serialize)]
pub struct AiStoreView {
    /// Store title.
    pub title: String,
    /// Root path.
    pub path: String,
    /// Total logical bytes.
    pub logical_bytes: u64,
    /// Walk hit a cap.
    pub capped: bool,
    /// Children.
    pub entries: Vec<AiChildView>,
}

/// One cleanup candidate the user or agent can review.
#[derive(Clone, Debug, Serialize)]
pub struct CandidateView {
    /// Candidate id.
    pub id: u64,
    /// Title.
    pub title: String,
    /// Path.
    pub path: String,
    /// Logical bytes.
    pub logical_bytes: u64,
    /// Pre-selected as SAFE generated.
    pub selected: bool,
    /// Blocked by safety.
    pub blocked: bool,
    /// Deletion strategy label.
    pub deletion: String,
}

/// Browser pressure snapshot.
#[derive(Clone, Debug, Serialize)]
pub struct BrowserView {
    /// Companion freshness.
    pub companion: String,
    /// Host count.
    pub hosts: usize,
    /// Combined browser RSS.
    pub rss_bytes: u64,
    /// Tab count when the companion is fresh.
    pub tabs: Option<usize>,
}

/// Result of [`apply_cleanup`](super::apply_cleanup).
#[derive(Clone, Debug, Serialize)]
pub struct ApplyReport {
    /// True when a plan ran.
    pub ok: bool,
    /// Why it did not run, if it did not.
    pub error: Option<String>,
    /// Deleted entries.
    pub deleted: u64,
    /// Skipped because they changed.
    pub skipped_changed: u64,
    /// Failed entries.
    pub failed: u64,
    /// Receipt id.
    pub receipt: Option<u64>,
}

/// Apply request. MCP must set `confirm`; the CLI uses `--apply` as that confirm.
#[derive(Clone, Debug)]
pub struct ApplyRequest {
    /// Required on MCP. The CLI sets this from `--apply`.
    pub confirm: bool,
    /// Scan root.
    pub root: PathBuf,
    /// If empty, apply pre-selected SAFE rows. If set, only those ids.
    pub ids: Vec<u64>,
}
