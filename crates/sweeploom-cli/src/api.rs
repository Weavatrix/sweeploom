//! Shared SweepLoom CLI / MCP operations. Inspect first; apply is explicit.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::Serialize;
use sweeploom_ai::{
    AiClass, ContextAdvice, advise_context, estimated_prompt_tokens_with_files, inspect_offers,
};
use sweeploom_browser::{BrowserPressure, load_snapshot};
use sweeploom_core::{Candidate, DeletionStrategy, ExecutionContext, ProjectId};
use sweeploom_dev::{ReviewRow, classify_project, collect_review_from, inspect};
use sweeploom_exec::{apply_plan_with, build_plan_with};
use sweeploom_general::collect_offers;
use sweeploom_network::enrich_network;
use sweeploom_platform::UserLocations;
use sweeploom_process::ProcessSampler;
use sweeploom_session::{
    AttributionRoots, plan_free_ram, plan_quiet_workstation, plan_reduce_cpu,
    sessions_from_snapshot,
};
use sweeploom_storage::{InventoryLimits, scan_inventory};

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

/// Result of [`apply_cleanup`].
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

/// List live sessions. Does not terminate anything.
#[must_use]
pub fn list_sessions() -> Vec<SessionView> {
    list_sessions_plan(None, None, false)
}

/// List sessions and mark a dry-run plan.
#[must_use]
pub fn list_sessions_plan(
    free_ram_gb: Option<f64>,
    reduce_cpu: Option<f32>,
    quiet: bool,
) -> Vec<SessionView> {
    let mut sampler = ProcessSampler::new();
    let mut snapshot = sampler.refresh(Duration::from_millis(200));
    snapshot.resolve_parents();
    let _ = enrich_network(&mut snapshot.processes);
    let locations = UserLocations::current();
    let current_project = std::env::current_dir().ok().map(ProjectId);
    let roots = AttributionRoots {
        projects: vec![locations.home.clone()],
        current_project: current_project.clone(),
    };
    let sessions = sessions_from_snapshot(&mut snapshot, &roots);
    let planned = planned_ids(
        &sessions,
        current_project.as_ref(),
        free_ram_gb,
        reduce_cpu,
        quiet,
    );
    sessions
        .iter()
        .map(|session| SessionView {
            kind: session.label().to_owned(),
            processes: session.processes.len(),
            rss_bytes: session.rss_bytes,
            cpu_percent: session.cpu_percent,
            recommendation: session.recommendation.recommendation.label().to_owned(),
            activity: session.activity.label().to_owned(),
            project: session
                .project
                .as_ref()
                .map(|item| item.0.display().to_string()),
            planned: planned.contains(&session.id),
        })
        .collect()
}

/// Scan a disk tree. Metadata only.
pub fn disk_inventory(root: &Path) -> Result<DiskInventory, String> {
    let report =
        scan_inventory(root, InventoryLimits::default()).map_err(|error| error.to_string())?;
    Ok(DiskInventory {
        root: report.root.display().to_string(),
        entries: report.entries,
        projects: report.projects.len(),
        capped: report.capped,
        logical_bytes: report.tree.logical_bytes,
        children: report
            .tree
            .children
            .iter()
            .take(32)
            .map(|child| DiskChild {
                path: child.path.display().to_string(),
                logical_bytes: child.logical_bytes,
                category: format!("{:?}", child.category),
            })
            .collect(),
    })
}

/// Project heat and git safety under `root`.
pub fn list_projects(root: &Path) -> Result<Vec<ProjectView>, String> {
    let report =
        scan_inventory(root, InventoryLimits::default()).map_err(|error| error.to_string())?;
    let now = SystemTime::now();
    Ok(report
        .projects
        .iter()
        .map(|project| {
            let (source, artifact) = report.project_heat(project, now);
            ProjectView {
                path: project.display().to_string(),
                kind: classify_project(project)
                    .into_iter()
                    .map(|item| item.label())
                    .collect::<Vec<_>>()
                    .join(","),
                source: source.label().to_owned(),
                artifact: artifact.label().to_owned(),
                git: inspect(project).label().to_owned(),
            }
        })
        .collect())
}

/// Inspect AI stores. Never opens secrets or SQLite.
#[must_use]
pub fn list_ai_stores() -> Vec<AiStoreView> {
    inspect_offers(&UserLocations::current())
        .into_iter()
        .map(|offer| AiStoreView {
            title: offer.title,
            path: offer.candidate.path.display().to_string(),
            logical_bytes: offer.candidate.logical_bytes,
            capped: offer.capped,
            entries: offer
                .entries
                .iter()
                .map(|entry| {
                    let tokens = estimated_prompt_tokens_with_files(
                        entry.class,
                        &entry.relative,
                        entry.candidate.logical_bytes,
                        entry.candidate.file_count,
                    );
                    let idle = entry
                        .candidate
                        .activity
                        .latest_any_modified
                        .and_then(|stamp| SystemTime::now().duration_since(stamp).ok());
                    AiChildView {
                        id: entry.candidate.id.0,
                        class: entry.class.label().to_owned(),
                        relative: entry.relative.clone(),
                        logical_bytes: entry.candidate.logical_bytes,
                        file_count: entry.candidate.file_count,
                        prompt_tokens: tokens,
                        advice: advise_context(&entry.relative, entry.class, idle)
                            .label()
                            .to_owned(),
                        can_clean: entry.class.can_clean(),
                    }
                })
                .collect(),
        })
        .collect()
}

/// Advise one always-on context path. Never writes `alwaysApply`.
#[must_use]
pub fn advise_one(relative: &str, class: AiClass, idle: Option<Duration>) -> ContextAdvice {
    advise_context(relative, class, idle)
}

/// Generated cleanup candidates (dry-run).
#[must_use]
pub fn cleanup_candidates(root: &Path) -> Vec<CandidateView> {
    review_rows(root)
        .into_iter()
        .map(|row| CandidateView {
            id: row.candidate.id.0,
            title: row.title,
            path: row.candidate.path.display().to_string(),
            logical_bytes: row.candidate.logical_bytes,
            selected: row.selected,
            blocked: row.candidate.safety.is_blocked(),
            deletion: format!("{:?}", row.candidate.deletion),
        })
        .collect()
}

/// Explain one candidate by id.
#[must_use]
pub fn explain_candidate(root: &Path, id: u64) -> Option<CandidateView> {
    cleanup_candidates(root)
        .into_iter()
        .find(|item| item.id == id)
        .or_else(|| {
            list_ai_stores()
                .into_iter()
                .flat_map(|store| store.entries)
                .find(|item| item.id == id)
                .map(|item| CandidateView {
                    id: item.id,
                    title: item.relative.clone(),
                    path: item.relative,
                    logical_bytes: item.logical_bytes,
                    selected: false,
                    blocked: !item.can_clean,
                    deletion: if item.can_clean {
                        "PermanentGenerated".to_owned()
                    } else {
                        "InspectOnly".to_owned()
                    },
                })
        })
}

/// Browser process pressure. Tab counts only with a fresh companion.
#[must_use]
pub fn list_browser() -> BrowserView {
    let mut sampler = ProcessSampler::new();
    let mut snapshot = sampler.refresh(Duration::from_millis(200));
    snapshot.resolve_parents();
    let _ = enrich_network(&mut snapshot.processes);
    let locations = UserLocations::current();
    let roots = AttributionRoots {
        projects: vec![locations.home.clone()],
        current_project: None,
    };
    let sessions = sessions_from_snapshot(&mut snapshot, &roots);
    let pressure = BrowserPressure::from_live(&sessions, &snapshot.processes);
    let now_ms = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|item| u64::try_from(item.as_millis()).unwrap_or(0))
        .unwrap_or(0);
    let stored = load_snapshot(&locations.app_data).ok().flatten();
    let companion = if stored.as_ref().is_some_and(|item| item.is_fresh(now_ms)) {
        "connected"
    } else if stored.is_some() {
        "stale"
    } else {
        "disconnected"
    };
    BrowserView {
        companion: companion.to_owned(),
        hosts: pressure.hosts.len(),
        rss_bytes: pressure.rss_bytes(),
        tabs: stored
            .as_ref()
            .filter(|item| item.is_fresh(now_ms))
            .map(|item| item.tabs.tabs.len()),
    }
}

/// Apply a reviewed cleanup. MCP must pass `confirm: true`. The CLI uses `--apply`.
///
/// Never terminates processes. Never deletes inspect-only / blocked rows.
#[must_use]
pub fn apply_cleanup(request: ApplyRequest) -> ApplyReport {
    if !request.confirm {
        return ApplyReport {
            ok: false,
            error: Some(
                "refused: MCP apply needs confirm=true after cleanup_candidates; CLI uses --apply"
                    .to_owned(),
            ),
            deleted: 0,
            skipped_changed: 0,
            failed: 0,
            receipt: None,
        };
    }
    let processes = sample_processes();
    let rows = review_rows(&request.root);
    let selected: Vec<Candidate> = rows
        .into_iter()
        .filter(|row| {
            if row.candidate.safety.is_blocked()
                || row.candidate.deletion == DeletionStrategy::InspectOnly
            {
                return false;
            }
            if request.ids.is_empty() {
                row.selected
            } else {
                request.ids.contains(&row.candidate.id.0)
            }
        })
        .map(|row| row.candidate)
        .collect();
    if selected.is_empty() {
        return ApplyReport {
            ok: false,
            error: Some("nothing eligible to apply".to_owned()),
            deleted: 0,
            skipped_changed: 0,
            failed: 0,
            receipt: None,
        };
    }
    let ctx = ExecutionContext::observed(&processes);
    let plan = build_plan_with(&selected, None, &ctx);
    let (report, receipt) = apply_plan_with(&plan, &ctx);
    ApplyReport {
        ok: true,
        error: None,
        deleted: report.counts.deleted,
        skipped_changed: report.counts.skipped_changed,
        failed: report.counts.failed,
        receipt: Some(receipt.plan.0),
    }
}

fn review_rows(root: &Path) -> Vec<ReviewRow> {
    let processes = sample_processes();
    let mut rows = collect_review_from(root, &processes, 128);
    let locations = UserLocations::current();
    if root == locations.home || root == locations.temp {
        for offer in collect_offers(&locations) {
            rows.push(ReviewRow {
                candidate: offer.candidate,
                selected: offer.selected,
                title: offer.title,
            });
        }
    }
    rows
}

fn sample_processes() -> Vec<sweeploom_core::ProcessSnapshot> {
    let mut sampler = ProcessSampler::new();
    sampler.refresh(Duration::from_millis(80)).processes
}

fn planned_ids(
    sessions: &[sweeploom_core::LiveSession],
    current_project: Option<&ProjectId>,
    free_ram_gb: Option<f64>,
    reduce_cpu: Option<f32>,
    quiet: bool,
) -> Vec<sweeploom_core::SessionId> {
    if quiet {
        return plan_quiet_workstation(sessions, current_project);
    }
    let mut ids = Vec::new();
    if let Some(gb) = free_ram_gb {
        ids.extend(plan_free_ram(sessions, (gb * 1_000_000_000.0) as u64));
    }
    if let Some(cpu) = reduce_cpu {
        for id in plan_reduce_cpu(sessions, cpu) {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_apply_without_confirm_does_not_run() {
        let report = apply_cleanup(ApplyRequest {
            confirm: false,
            root: std::env::temp_dir(),
            ids: Vec::new(),
        });
        assert!(!report.ok);
        assert_eq!(report.deleted, 0);
        assert!(
            report
                .error
                .as_deref()
                .unwrap_or("")
                .contains("confirm=true")
        );
    }

    #[test]
    fn advise_does_not_claim_history() {
        assert_eq!(
            advise_one("history.jsonl", AiClass::History, None).label(),
            ""
        );
        assert_eq!(
            advise_one("AGENTS.md", AiClass::Context, None).label(),
            "keep"
        );
    }
}
