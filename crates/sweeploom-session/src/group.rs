//! Hierarchical grouping: project → terminal/agent session → helpers.

use std::collections::{HashMap, HashSet};

use sweeploom_core::{
    LiveSession, ProcessKey, ProcessSafetyClass, ProcessSnapshot, Recommendation, SessionActivity,
    SessionDiskUsage, SessionId, SessionKind, SessionNetworkUsage, SessionRecommendation,
    SessionSafety,
};

use crate::detectors::classify_process;
use crate::forgotten::safety_from_class;

/// Group processes into logical sessions.
#[must_use]
pub fn group_sessions(processes: &[ProcessSnapshot]) -> Vec<LiveSession> {
    let by_key: HashMap<ProcessKey, &ProcessSnapshot> = processes
        .iter()
        .map(|process| (process.key, process))
        .collect();
    let mut assigned: HashSet<ProcessKey> = HashSet::new();
    let mut sessions = Vec::new();
    let mut next_id = 1_u64;

    let mut roots: Vec<&ProcessSnapshot> = processes
        .iter()
        .filter(|process| classify_process(process).is_some())
        .collect();
    roots.sort_by_key(|process| process.pid);

    for root in roots {
        if assigned.contains(&root.key) {
            continue;
        }
        let kind = classify_process(root)
            .map(|item| item.kind)
            .unwrap_or(SessionKind::Unknown);
        let members = collect_descendants(root.key, processes, &by_key);
        for member in &members {
            assigned.insert(*member);
        }
        sessions.push(build_session(SessionId(next_id), kind, &members, &by_key));
        next_id += 1;
    }

    for process in processes {
        if assigned.contains(&process.key) {
            continue;
        }
        if process.safety_class == ProcessSafetyClass::SystemCritical {
            continue;
        }
        assigned.insert(process.key);
        sessions.push(build_session(
            SessionId(next_id),
            SessionKind::GenericApp,
            &[process.key],
            &by_key,
        ));
        next_id += 1;
    }

    sessions.sort_by_key(|session| std::cmp::Reverse(session.rss_bytes));
    sessions
}

fn collect_descendants(
    root: ProcessKey,
    processes: &[ProcessSnapshot],
    _by_key: &HashMap<ProcessKey, &ProcessSnapshot>,
) -> Vec<ProcessKey> {
    let mut members = vec![root];
    let mut changed = true;
    while changed {
        changed = false;
        for process in processes {
            if members.contains(&process.key) {
                continue;
            }
            if let Some(parent) = process.parent
                && members.contains(&parent)
            {
                members.push(process.key);
                changed = true;
            }
        }
    }
    members
}

fn build_session(
    id: SessionId,
    kind: SessionKind,
    members: &[ProcessKey],
    by_key: &HashMap<ProcessKey, &ProcessSnapshot>,
) -> LiveSession {
    let snapshots: Vec<&&ProcessSnapshot> =
        members.iter().filter_map(|key| by_key.get(key)).collect();
    let rss_bytes = snapshots.iter().map(|item| item.rss_bytes).sum();
    let cpu_percent = snapshots.iter().map(|item| item.cpu_percent).sum();
    let started_at = snapshots.iter().filter_map(|item| item.started_at).min();
    let project = snapshots.iter().find_map(|item| {
        item.project.as_ref().and_then(|attribution| {
            if attribution.confidence == sweeploom_core::Confidence::Exact
                || attribution.confidence == sweeploom_core::Confidence::Strong
            {
                Some(attribution.project.clone())
            } else {
                None
            }
        })
    });
    let safety = snapshots
        .iter()
        .map(|item| safety_from_class(item.safety_class))
        .find(|item| item.terminate_disabled)
        .unwrap_or_else(SessionSafety::user);
    let listening_ports = snapshots
        .iter()
        .flat_map(|item| item.network.listening_ports.iter().copied())
        .collect();
    LiveSession {
        id,
        kind,
        project,
        processes: members.to_vec(),
        started_at,
        observed_last_activity: started_at,
        rss_bytes,
        cpu_percent,
        disk: SessionDiskUsage {
            read_bytes: snapshots.iter().map(|item| item.disk_read_delta).sum(),
            write_bytes: snapshots.iter().map(|item| item.disk_write_delta).sum(),
        },
        network: SessionNetworkUsage {
            connections_available: snapshots
                .iter()
                .any(|item| item.network.connections_available),
            byte_rate_available: snapshots
                .iter()
                .any(|item| item.network.byte_rate_available),
            observed_rx_bytes: snapshots
                .iter()
                .map(|item| item.network.observed_rx_bytes)
                .sum(),
            observed_tx_bytes: snapshots
                .iter()
                .map(|item| item.network.observed_tx_bytes)
                .sum(),
            listening_ports,
        },
        activity: SessionActivity::Unknown,
        safety,
        recommendation: SessionRecommendation {
            recommendation: Recommendation::Keep,
            estimated_reclaimable_rss: 0,
        },
    }
}
