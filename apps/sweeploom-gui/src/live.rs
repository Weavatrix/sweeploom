//! First sample and attribution roots.

use std::time::Duration;

use sweeploom_core::{LiveSession, ProjectId};
use sweeploom_network::enrich_network;
use sweeploom_platform::UserLocations;
use sweeploom_process::{ProcessSampler, ProcessSnapshotSet};
use sweeploom_session::{AttributionRoots, sessions_from_snapshot};

use crate::app::SweepLoomApp;

/// Warm the sampler and group the first snapshot.
pub fn sample_with(
    sampler: &mut ProcessSampler,
    locations: &UserLocations,
) -> (ProcessSnapshotSet, Vec<LiveSession>) {
    let mut snapshot = sampler.refresh(Duration::from_millis(200));
    snapshot.resolve_parents();
    let _ = enrich_network(&mut snapshot.processes);
    let sessions = sessions_from_snapshot(
        &mut snapshot,
        &AttributionRoots {
            projects: vec![locations.home.clone()],
            current_project: std::env::current_dir().ok().map(ProjectId),
        },
    );
    (snapshot, sessions)
}

/// Home plus any inventory project roots.
pub fn session_roots(app: &SweepLoomApp) -> AttributionRoots {
    let mut projects = vec![app.locations.home.clone()];
    if let Some(report) = &app.inventory {
        for project in &report.projects {
            if !projects.iter().any(|item| item == project) {
                projects.push(project.clone());
            }
        }
    }
    AttributionRoots {
        projects,
        current_project: app.current_project.clone(),
    }
}
