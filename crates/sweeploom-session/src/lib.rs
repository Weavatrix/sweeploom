//! Logical sessions on top of the OS process tree.
//!
//! Grouping uses ancestry, cwd, command-line signatures, and listening ports.
//! Project attribution never guesses from the process name alone.

#![cfg_attr(not(test), warn(missing_docs))]

mod attribution;
mod detectors;
mod forgotten;
mod group;
mod idle_agent;
mod node;
mod plan;

#[cfg(test)]
mod session_bench;

#[cfg(test)]
mod browser_tests;
#[cfg(test)]
mod ownership_tests;
#[cfg(test)]
mod with_without_bench;

pub use attribution::{AttributionRoots, attribute_projects};
pub use detectors::{SessionDetector, SessionEvidence, builtin_detectors, classify_process};
pub use forgotten::{ForgottenInput, mark_orphan_mcp, score_session};
pub use group::group_sessions;
pub use idle_agent::{IDLE_BENCH_KINDS, idle_2gb_agent, idle_agent_survive, naive_idle_reclaim};
pub use node::{NodeIdentity, is_node, is_tool_installation, node_identity, version_from_path};
pub use plan::{is_reclaim_candidate, plan_free_ram, plan_quiet_workstation, plan_reduce_cpu};

use sweeploom_core::{LiveSession, ProcessSnapshot};
use sweeploom_process::ProcessSnapshotSet;

/// Build logical sessions from a process snapshot.
#[must_use]
pub fn sessions_from_snapshot(
    snapshot: &mut ProcessSnapshotSet,
    roots: &AttributionRoots,
) -> Vec<LiveSession> {
    snapshot.resolve_parents();
    attribute_projects(&mut snapshot.processes, roots);
    let mut sessions = group_sessions(&snapshot.processes);
    for session in &mut sessions {
        *session = score_session(
            session,
            snapshot.captured_at,
            roots.current_project.as_ref(),
        );
    }
    mark_orphan_mcp(&mut sessions, &snapshot.processes);
    stamp_session_ids(&mut snapshot.processes, &sessions);
    sessions
}

fn stamp_session_ids(processes: &mut [ProcessSnapshot], sessions: &[LiveSession]) {
    for process in processes {
        process.session = sessions
            .iter()
            .find(|session| session.processes.contains(&process.key))
            .map(|session| session.id);
    }
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
