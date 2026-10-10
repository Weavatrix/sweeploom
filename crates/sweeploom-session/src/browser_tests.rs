use crate::{group_sessions, tests::proc};
use std::{collections::HashSet, path::PathBuf};
use sweeploom_core::SessionKind;

#[test]
fn macos_chrome_and_safari_have_distinct_complete_trees() {
    let processes = vec![
        proc(
            10,
            None,
            "Google Chrome",
            None,
            &["Google Chrome"],
            100,
            1.0,
        ),
        proc(
            11,
            Some(10),
            "Google Chrome Helper (Renderer)",
            None,
            &["Google Chrome Helper (Renderer)", "--type=renderer"],
            200,
            2.0,
        ),
        proc(
            12,
            Some(10),
            "Google Chrome Helper",
            None,
            &["Google Chrome Helper", "--type=gpu-process"],
            300,
            3.0,
        ),
        proc(20, None, "Safari", None, &["Safari"], 400, 4.0),
        proc(
            21,
            Some(20),
            "com.apple.WebKit.WebContent",
            None,
            &[],
            500,
            5.0,
        ),
        proc(30, None, "SafariNotificationAgent", None, &[], 10, 0.0),
        proc(
            40,
            None,
            "node",
            None,
            &["node", "https://example.test/chrome"],
            1,
            0.0,
        ),
    ];
    let sessions = group_sessions(&processes);
    let chrome = sessions
        .iter()
        .find(|session| session.processes[0].pid == 10)
        .unwrap();
    let safari = sessions
        .iter()
        .find(|session| session.processes[0].pid == 20)
        .unwrap();
    assert_eq!(chrome.kind, SessionKind::Browser);
    assert_eq!(chrome.processes.len(), 3);
    assert_eq!(chrome.rss_bytes, 600);
    assert_eq!(safari.kind, SessionKind::Browser);
    assert_eq!(safari.processes.len(), 2);
    assert_eq!(safari.rss_bytes, 900);
    assert_eq!(
        sessions
            .iter()
            .filter(|session| session.kind == SessionKind::Browser)
            .count(),
        3
    );
    let keys: Vec<_> = sessions
        .iter()
        .flat_map(|session| &session.processes)
        .collect();
    assert_eq!(keys.len(), processes.len());
    assert_eq!(keys.iter().collect::<HashSet<_>>().len(), processes.len());
    assert_eq!(
        sessions
            .iter()
            .find(|session| session.processes[0].pid == 40)
            .unwrap()
            .kind,
        SessionKind::GenericApp
    );
}

#[test]
fn executable_metadata_recovers_truncated_browser_and_orphan_helper() {
    let mut main = proc(10, None, "Google Chro", None, &[], 100, 0.0);
    main.exe = Some(PathBuf::from(
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    ));
    let helper = proc(
        20,
        None,
        "Google Chrome Helper (Renderer)",
        None,
        &["Google Chrome Helper (Renderer)", "--type=renderer"],
        200,
        0.0,
    );
    let sessions = group_sessions(&[main, helper]);
    assert_eq!(sessions.len(), 2);
    assert!(
        sessions
            .iter()
            .all(|session| session.kind == SessionKind::Browser)
    );
}
