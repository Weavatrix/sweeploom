//! Logical sessions on top of the OS process tree.
//!
//! Grouping uses ancestry, cwd, command-line signatures, and listening ports.
//! Project attribution never guesses from the process name alone.

#![cfg_attr(not(test), warn(missing_docs))]

mod attribution;
mod detectors;
mod forgotten;
mod group;
mod plan;

pub use attribution::{AttributionRoots, attribute_projects};
pub use detectors::{SessionDetector, SessionEvidence, builtin_detectors, classify_process};
pub use forgotten::{ForgottenInput, score_session};
pub use group::group_sessions;
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
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};

    use sweeploom_core::{
        NetworkSnapshot, ProcessKey, ProcessSafetyClass, ProcessSnapshot, ProjectId, SessionKind,
    };

    fn proc(
        pid: u32,
        parent: Option<u32>,
        name: &str,
        cwd: Option<&str>,
        command: &[&str],
        rss: u64,
        cpu: f32,
    ) -> ProcessSnapshot {
        let started = Some(SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000));
        ProcessSnapshot {
            key: ProcessKey::new(pid, started),
            pid,
            parent: parent.map(|item| ProcessKey::new(item, started)),
            name: name.to_owned(),
            exe: None,
            cwd: cwd.map(PathBuf::from),
            command: command.iter().map(|item| (*item).to_owned()).collect(),
            started_at: started,
            runtime: Duration::from_secs(4 * 24 * 3600),
            rss_bytes: rss,
            virtual_bytes: rss,
            cpu_percent: cpu,
            accumulated_cpu_ms: 0,
            disk_read_delta: 0,
            disk_write_delta: 0,
            network: NetworkSnapshot::default(),
            project: None,
            session: None,
            safety_class: ProcessSafetyClass::Unknown,
        }
    }

    #[test]
    fn groups_claude_tree_and_attributes_project() {
        let project = PathBuf::from("/work/kablay");
        let processes = vec![
            proc(1, None, "init", None, &["init"], 8_000, 0.0),
            proc(
                10,
                Some(1),
                "claude",
                Some("/work/kablay"),
                &["claude"],
                200_000_000,
                0.1,
            ),
            proc(
                11,
                Some(10),
                "node",
                Some("/work/kablay"),
                &["node", "vite"],
                400_000_000,
                0.0,
            ),
            proc(
                12,
                Some(10),
                "node",
                Some("/work/kablay"),
                &["node", "mcp-server"],
                80_000_000,
                0.0,
            ),
            proc(
                13,
                Some(10),
                "powershell.exe",
                Some("/work/kablay"),
                &["powershell"],
                20_000_000,
                0.0,
            ),
        ];
        let mut snapshot = ProcessSnapshotSet {
            captured_at: SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_400_000),
            processes,
            memory: sweeploom_process::HostMemory::default(),
            cpu: sweeploom_process::HostCpu::default(),
            total_rss_bytes: 0,
        };
        let roots = AttributionRoots {
            projects: vec![project.clone()],
            current_project: Some(ProjectId(project.clone())),
        };
        let sessions = sessions_from_snapshot(&mut snapshot, &roots);
        let claude = sessions
            .iter()
            .find(|session| session.kind == SessionKind::ClaudeCode)
            .expect("claude session");
        assert_eq!(claude.processes.len(), 1);
        assert_eq!(claude.project, Some(ProjectId(project.clone())));
        assert!(
            sessions
                .iter()
                .any(|session| session.kind == SessionKind::DevServer)
        );
        assert!(
            sessions
                .iter()
                .any(|session| session.kind == SessionKind::Mcp)
        );
        assert!(
            sessions
                .iter()
                .any(|session| session.kind == SessionKind::Terminal)
        );
    }

    #[test]
    fn leftover_parent_keeps_unclassified_children() {
        let processes = vec![
            proc(40, None, "Slack.exe", None, &["Slack"], 200_000_000, 0.0),
            proc(
                41,
                Some(40),
                "crashpad.exe",
                None,
                &["crashpad"],
                20_000_000,
                0.0,
            ),
        ];
        let mut snapshot = ProcessSnapshotSet {
            captured_at: SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_400_000),
            processes,
            memory: sweeploom_process::HostMemory::default(),
            cpu: sweeploom_process::HostCpu::default(),
            total_rss_bytes: 0,
        };
        let sessions = sessions_from_snapshot(
            &mut snapshot,
            &AttributionRoots {
                projects: Vec::new(),
                current_project: None,
            },
        );
        let slack = sessions
            .iter()
            .find(|session| session.kind == SessionKind::GenericApp && session.processes.len() == 2)
            .expect("slack tree");
        assert_eq!(slack.processes.len(), 2);
    }

    #[test]
    fn cursor_keeps_unclassified_node_helper() {
        let processes = vec![
            proc(1, None, "init", None, &["init"], 8_000, 0.0),
            proc(
                50,
                Some(1),
                "Cursor.exe",
                None,
                &["Cursor"],
                300_000_000,
                0.2,
            ),
            proc(
                51,
                Some(50),
                "node.exe",
                None,
                &["node", "language-host"],
                80_000_000,
                0.0,
            ),
            proc(
                52,
                Some(50),
                "powershell.exe",
                None,
                &["powershell"],
                15_000_000,
                0.0,
            ),
        ];
        let mut snapshot = ProcessSnapshotSet {
            captured_at: SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_400_000),
            processes,
            memory: sweeploom_process::HostMemory::default(),
            cpu: sweeploom_process::HostCpu::default(),
            total_rss_bytes: 0,
        };
        let sessions = sessions_from_snapshot(
            &mut snapshot,
            &AttributionRoots {
                projects: Vec::new(),
                current_project: None,
            },
        );
        let cursor = sessions
            .iter()
            .find(|session| session.kind == SessionKind::GenericApp)
            .expect("cursor");
        assert_eq!(cursor.processes.len(), 2);
        assert!(
            sessions
                .iter()
                .any(|session| session.kind == SessionKind::Terminal)
        );
    }
}
