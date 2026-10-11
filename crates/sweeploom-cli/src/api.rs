//! Shared SweepLoom CLI / MCP operations. Inspect first; apply is explicit.

use std::path::Path;
use std::time::{Duration, SystemTime};

use sweeploom_ai::{
    AiClass, ContextAdvice, advise_context, estimated_prompt_tokens_with_files, inspect_offers,
};
use sweeploom_core::{Candidate, DeletionStrategy, ExecutionContext};
use sweeploom_dev::{ReviewRow, classify_project, collect_review_from, inspect};
use sweeploom_exec::{apply_plan_with, build_plan_with};
use sweeploom_general::collect_offers;
use sweeploom_platform::UserLocations;
use sweeploom_process::ProcessSampler;
use sweeploom_storage::{InventoryLimits, scan_inventory};

#[path = "api_live.rs"]
mod live;
#[path = "api_views.rs"]
mod views;

pub use live::{list_browser, list_sessions, list_sessions_plan};
pub use views::{
    AiChildView, AiStoreView, ApplyReport, ApplyRequest, BrowserView, CandidateView, DiskChild,
    DiskInventory, ProjectView, SessionView,
};

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

#[cfg(test)]
#[path = "api_tests.rs"]
mod tests;
