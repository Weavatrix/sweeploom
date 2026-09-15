//! Session detector / forgotten policy gold vs the previous Cursor-as-GenericApp rules.

use std::time::{Duration, SystemTime};

use sweeploom_core::{
    LiveSession, NetworkSnapshot, ProcessKey, ProcessSafetyClass, ProcessSnapshot, Recommendation,
    SessionActivity, SessionDiskUsage, SessionId, SessionKind, SessionNetworkUsage,
    SessionRecommendation, SessionSafety,
};

use super::{
    AttributionRoots, classify_process, group_sessions, mark_orphan_mcp, score_session,
    sessions_from_snapshot,
};
use sweeploom_process::ProcessSnapshotSet;

fn proc(
    pid: u32,
    parent: Option<u32>,
    name: &str,
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
        cwd: None,
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

fn scored_session(rss: u64, kind: SessionKind) -> LiveSession {
    LiveSession {
        id: SessionId(1),
        kind,
        project: None,
        processes: Vec::new(),
        started_at: Some(SystemTime::UNIX_EPOCH),
        observed_last_activity: Some(SystemTime::UNIX_EPOCH),
        rss_bytes: rss,
        cpu_percent: 0.0,
        disk: SessionDiskUsage::default(),
        network: SessionNetworkUsage::default(),
        activity: SessionActivity::Unknown,
        safety: SessionSafety::user(),
        recommendation: SessionRecommendation {
            recommendation: Recommendation::Keep,
            estimated_reclaimable_rss: 0,
        },
    }
}

#[test]
fn bench_session_accuracy_and_speed() {
    let cursor =
        classify_process(&proc(1, None, "Cursor.exe", &["Cursor"], 1, 0.0)).map(|item| item.kind);
    let mcp_flag = classify_process(&proc(
        2,
        None,
        "node.exe",
        &["node", "--mcp", "server.js"],
        1,
        0.0,
    ))
    .map(|item| item.kind);
    let slack =
        classify_process(&proc(3, None, "Slack.exe", &["Slack"], 1, 0.0)).map(|item| item.kind);

    eprintln!("detect Cursor.exe -> {cursor:?} (want Some(Cursor))");
    eprintln!("detect node --mcp -> {mcp_flag:?} (want Some(Mcp))");
    eprintln!("detect Slack.exe -> {slack:?} (want None, leftover GenericApp)");

    assert_eq!(cursor, Some(SessionKind::Cursor));
    assert_eq!(mcp_flag, Some(SessionKind::Mcp));
    assert_eq!(slack, None);

    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10 * 3600);
    let idle_cursor = score_session(
        &scored_session(2_000_000_000, SessionKind::Cursor),
        now,
        None,
    );
    let idle_generic = score_session(
        &scored_session(2_000_000_000, SessionKind::GenericApp),
        now,
        None,
    );
    eprintln!(
        "idle Cursor 2GB -> {:?} {:?} (agents stay Keep to keep chat context)",
        idle_cursor.recommendation.recommendation, idle_cursor.activity
    );
    eprintln!(
        "idle GenericApp 2GB -> {:?} (must stay Keep)",
        idle_generic.recommendation.recommendation
    );
    assert_eq!(
        idle_cursor.recommendation.recommendation,
        Recommendation::Keep
    );
    assert_eq!(
        idle_generic.recommendation.recommendation,
        Recommendation::Keep
    );

    let mut snapshot = ProcessSnapshotSet {
        captured_at: now,
        processes: vec![
            proc(1, None, "init", &["init"], 8_000, 0.0),
            proc(50, Some(1), "Cursor.exe", &["Cursor"], 300_000_000, 0.2),
            proc(
                51,
                Some(50),
                "node.exe",
                &["node", "--mcp", "server.js"],
                80_000_000,
                0.0,
            ),
            proc(
                90,
                Some(1),
                "node.exe",
                &["node", "mcp-server"],
                40_000_000,
                0.0,
            ),
        ],
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
    let attached = sessions.iter().any(|session| {
        session.kind == SessionKind::Mcp
            && session.activity != SessionActivity::OrphanCandidate
            && session.recommendation.recommendation == Recommendation::Keep
            && session.processes.iter().any(|key| key.pid == 51)
    });
    let stray = sessions.iter().any(|session| {
        session.kind == SessionKind::Mcp
            && session.activity == SessionActivity::OrphanCandidate
            && session.recommendation.recommendation == Recommendation::Optional
            && session.processes.iter().any(|key| key.pid == 90)
    });
    eprintln!("MCP under Cursor attached (not orphan)={attached}");
    eprintln!("stray MCP orphan={stray}");
    assert!(attached);
    assert!(stray);
    let tree = snapshot.processes.clone();
    let start = std::time::Instant::now();
    let rounds = 8_000_u32;
    let mut sink = 0_u64;
    for _ in 0..rounds {
        let mut grouped = group_sessions(&tree);
        mark_orphan_mcp(&mut grouped, &tree);
        sink = sink.wrapping_add(grouped.len() as u64);
    }
    let elapsed = start.elapsed();
    let ns = elapsed.as_nanos() / u128::from(rounds);
    eprintln!("group_sessions+orphan {rounds} trees in {elapsed:?} (~{ns} ns/tree) sink={sink}");
    assert!(ns < 5_000_000, "grouping got slower than 5ms/tree: {ns} ns");
}

#[test]
fn bench_token_policy_keeps_context() {
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10 * 3600);
    let agents = [
        SessionKind::ClaudeCode,
        SessionKind::Codex,
        SessionKind::Cursor,
        SessionKind::OpenCode,
        SessionKind::Gemini,
        SessionKind::Grok,
    ];
    eprintln!("--- token policy: live agents Keep (context preserved) ---");
    for kind in agents {
        let scored = score_session(&scored_session(2_000_000_000, kind), now, None);
        eprintln!(
            "  idle {kind:?} 2GB -> {:?} {:?} reclaim={}",
            scored.recommendation.recommendation,
            scored.activity,
            scored.recommendation.estimated_reclaimable_rss
        );
        assert_eq!(
            scored.recommendation.recommendation,
            Recommendation::Keep,
            "{kind:?} must not be reclaim-planned (would drop chat context)"
        );
        assert_eq!(scored.recommendation.estimated_reclaimable_rss, 0);
    }

    let mut snapshot = ProcessSnapshotSet {
        captured_at: now,
        processes: vec![
            proc(1, None, "init", &["init"], 8_000, 0.0),
            proc(50, Some(1), "Cursor.exe", &["Cursor"], 300_000_000, 0.0),
            proc(
                51,
                Some(50),
                "node.exe",
                &["node", "--mcp", "server.js"],
                80_000_000,
                0.0,
            ),
            proc(
                90,
                Some(1),
                "node.exe",
                &["node", "mcp-server"],
                40_000_000,
                0.0,
            ),
        ],
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
    let attached_rss: u64 = sessions
        .iter()
        .filter(|session| {
            session.kind == SessionKind::Mcp
                && session.recommendation.recommendation == Recommendation::Keep
        })
        .map(|session| session.rss_bytes)
        .sum();
    let orphan_rss: u64 = sessions
        .iter()
        .filter(|session| {
            session.kind == SessionKind::Mcp
                && session.activity == SessionActivity::OrphanCandidate
                && session.recommendation.recommendation == Recommendation::Optional
        })
        .map(|session| session.rss_bytes)
        .sum();
    eprintln!(
        "attached MCP Keep RSS={attached_rss}  orphan MCP Optional RSS={orphan_rss} (only orphan is reclaimable)"
    );
    assert_eq!(attached_rss, 80_000_000);
    assert_eq!(orphan_rss, 40_000_000);
}
