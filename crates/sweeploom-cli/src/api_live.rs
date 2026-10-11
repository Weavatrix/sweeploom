//! Live process / browser views. Sampling only; nothing is terminated.

use std::time::{Duration, SystemTime};

use sweeploom_browser::{BrowserPressure, load_snapshot};
use sweeploom_core::ProjectId;
use sweeploom_network::enrich_network;
use sweeploom_platform::UserLocations;
use sweeploom_process::ProcessSampler;
use sweeploom_session::{
    AttributionRoots, plan_free_ram, plan_quiet_workstation, plan_reduce_cpu,
    sessions_from_snapshot,
};

use super::{BrowserView, SessionView};

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
