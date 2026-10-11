use crate::{AttributionRoots, group_sessions, sessions_from_snapshot, tests::proc};
use std::time::{Duration, SystemTime};
use sweeploom_core::{Recommendation, SessionActivity, SessionKind};
use sweeploom_process::ProcessSnapshotSet;

#[test]
fn each_process_has_one_owning_session() {
    let processes = vec![
        proc(500, None, "claude", None, &["claude"], 200_000_000, 0.0),
        proc(100, Some(500), "claude", None, &["claude"], 80_000_000, 0.0),
    ];
    let sessions = group_sessions(&processes);
    let child = processes[1].key;
    let owners = sessions
        .iter()
        .filter(|session| session.processes.contains(&child))
        .count();
    assert_eq!(owners, 1);
    assert_eq!(
        sessions
            .iter()
            .find(|session| session.kind == SessionKind::ClaudeCode)
            .map(|session| session.processes.len()),
        Some(2)
    );
}

#[test]
fn mcp_child_of_cursor_is_attached_not_orphan() {
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
            &["node", "--mcp", "server.js"],
            80_000_000,
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
    let mcp = sessions
        .iter()
        .find(|session| session.kind == SessionKind::Mcp)
        .expect("mcp");
    assert_ne!(mcp.activity, SessionActivity::OrphanCandidate);
    assert_eq!(mcp.recommendation.recommendation, Recommendation::Keep);
    assert!(
        sessions
            .iter()
            .any(|session| session.kind == SessionKind::Cursor)
    );
}

#[test]
fn stray_mcp_is_orphan_candidate() {
    let processes = vec![
        proc(1, None, "init", None, &["init"], 8_000, 0.0),
        proc(
            90,
            Some(1),
            "node.exe",
            None,
            &["node", "mcp-server"],
            80_000_000,
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
    let mcp = sessions
        .iter()
        .find(|session| session.kind == SessionKind::Mcp)
        .expect("mcp");
    assert_eq!(mcp.activity, SessionActivity::OrphanCandidate);
    assert_eq!(mcp.recommendation.recommendation, Recommendation::Optional);
}
