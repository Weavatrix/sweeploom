//! First sample and attribution roots.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use sweeploom_core::{LiveSession, ObservationTracker, ProcessKey, ProcessSnapshot, ProjectId};
use sweeploom_network::enrich_network;
use sweeploom_platform::UserLocations;
use sweeploom_process::{ProcessSampler, ProcessSnapshotSet};
use sweeploom_session::{AttributionRoots, score_session, sessions_from_snapshot};

use crate::app::SweepLoomApp;

/// Warm the sampler and group the first snapshot.
pub fn sample_with(
    sampler: &mut ProcessSampler,
    _locations: &UserLocations,
) -> (ProcessSnapshotSet, Vec<LiveSession>) {
    let mut snapshot = sampler.refresh(Duration::from_millis(200));
    snapshot.resolve_parents();
    let _ = enrich_network(&mut snapshot.processes);
    let sessions = sessions_from_snapshot(
        &mut snapshot,
        &AttributionRoots {
            projects: marker_project(std::env::current_dir().ok())
                .into_iter()
                .collect(),
            current_project: marker_project(std::env::current_dir().ok()).map(ProjectId),
        },
    );
    (snapshot, sessions)
}

/// Home plus any inventory project roots.
pub fn session_roots(app: &SweepLoomApp) -> AttributionRoots {
    let mut projects = Vec::new();
    if let Some(report) = &app.inventory {
        for project in &report.projects {
            if !projects.iter().any(|item| item == project) {
                projects.push(project.clone());
            }
        }
    }
    for project in &app.project_roots {
        if !projects.iter().any(|item| item == project) {
            projects.push(project.clone());
        }
    }
    AttributionRoots {
        projects,
        current_project: app.current_project.clone(),
    }
}

/// Current project only when cwd has a proven marker. Home is never Exact.
#[must_use]
pub fn current_project() -> Option<ProjectId> {
    marker_project(std::env::current_dir().ok()).map(ProjectId)
}

fn marker_project(path: Option<std::path::PathBuf>) -> Option<std::path::PathBuf> {
    let path = path?;
    [
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "Pipfile",
        "go.mod",
    ]
    .iter()
    .any(|marker| path.join(marker).is_file())
    .then_some(path)
}

/// Remember the last sample where a process was actually busy.
fn note_busy(
    last_busy: &mut HashMap<ProcessKey, SystemTime>,
    processes: &[ProcessSnapshot],
    at: SystemTime,
) {
    for process in processes {
        if process.cpu_percent > 0.5 || process.disk_read_delta > 0 || process.disk_write_delta > 0
        {
            last_busy.insert(process.key, at);
        }
    }
}

/// Stamp last-activity from the busy clock, then re-score.
///
/// Start time is never treated as idle. Unknown idle stays Keep.
fn apply_idle_clock(
    sessions: &mut [LiveSession],
    last_busy: &HashMap<ProcessKey, SystemTime>,
    observation: &ObservationTracker,
    now: SystemTime,
    current: Option<&ProjectId>,
) {
    for session in sessions {
        let busy_now = session.cpu_percent > 0.5
            || session.disk.read_bytes > 0
            || session.disk.write_bytes > 0;
        session.observed_last_activity = if busy_now {
            Some(now)
        } else {
            session
                .processes
                .iter()
                .find_map(|key| observation.observed_idle_since(*key, now))
                .or_else(|| {
                    session
                        .processes
                        .iter()
                        .filter_map(|key| last_busy.get(key).copied())
                        .max()
                        .filter(|_| !observation.has_gap())
                })
        };
        *session = score_session(session, now, current);
    }
}

/// Record history-adjacent busy stamps and score the first snapshot.
pub fn stamp_first(app: &mut SweepLoomApp) {
    let Some(snapshot) = app.snapshot.take() else {
        return;
    };
    app.history
        .record(&snapshot.processes, snapshot.captured_at);
    note_busy(
        &mut app.last_busy,
        &snapshot.processes,
        snapshot.captured_at,
    );
    record_observation(
        &mut app.observation,
        &snapshot.processes,
        snapshot.captured_at,
    );
    apply_idle_clock(
        &mut app.sessions,
        &app.last_busy,
        &app.observation,
        snapshot.captured_at,
        app.current_project.as_ref(),
    );
    app.snapshot = Some(snapshot);
}

/// Group, apply the idle clock, and replace `sessions`.
pub fn rescore(
    sessions: &mut Vec<LiveSession>,
    last_busy: &mut HashMap<ProcessKey, SystemTime>,
    snapshot: &mut ProcessSnapshotSet,
    current: Option<&ProjectId>,
    roots: &AttributionRoots,
    observation: &mut ObservationTracker,
) {
    note_busy(last_busy, &snapshot.processes, snapshot.captured_at);
    record_observation(observation, &snapshot.processes, snapshot.captured_at);
    *sessions = sessions_from_snapshot(snapshot, roots);
    apply_idle_clock(
        sessions,
        last_busy,
        observation,
        snapshot.captured_at,
        current,
    );
}

fn record_observation(
    observation: &mut ObservationTracker,
    processes: &[ProcessSnapshot],
    at: SystemTime,
) {
    observation.record(
        processes.iter().map(|item| item.key),
        processes.iter().filter_map(|item| {
            (item.cpu_percent > 0.5 || item.disk_read_delta > 0 || item.disk_write_delta > 0)
                .then_some(item.key)
        }),
        processes.iter().filter_map(|item| {
            (item.network.byte_rate_available
                && item.network.observed_rx_bytes + item.network.observed_tx_bytes > 0)
                .then_some(item.key)
        }),
        at,
    );
}
