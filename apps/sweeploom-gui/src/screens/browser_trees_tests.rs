use super::*;
use std::{path::PathBuf, time::Duration};
use sweeploom_core::{NetworkSnapshot, ProcessKey, ProcessSafetyClass};

fn process(pid: u32, name: &str, command: &[&str]) -> ProcessSnapshot {
    ProcessSnapshot {
        key: ProcessKey::new(pid, None),
        pid,
        parent: None,
        name: name.into(),
        exe: None,
        cwd: None,
        command: command.iter().map(|arg| (*arg).into()).collect(),
        started_at: None,
        runtime: Duration::ZERO,
        rss_bytes: 100,
        virtual_bytes: 100,
        cpu_percent: 0.0,
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
fn macos_browsers_are_visible_with_helpers_counted_once_and_main_protected() {
    let mut chrome = process(10, "Google Chro", &[]);
    chrome.exe = Some(PathBuf::from(
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    ));
    let mut renderer = process(
        11,
        "Google Chrome Helper (Renderer)",
        &["Google Chrome Helper (Renderer)", "--type=renderer"],
    );
    renderer.parent = Some(chrome.key);
    let safari = process(20, "Safari", &["Safari"]);
    let processes = [chrome, renderer, safari];
    let sessions = sweeploom_session::group_sessions(&processes);
    let rows = collect_rows(&sessions, &processes);
    assert_eq!(rows.len(), 2);
    let chrome = rows.iter().find(|row| row.family == "Chrome").unwrap();
    let safari = rows.iter().find(|row| row.family == "Safari").unwrap();
    assert_eq!(chrome.pid, 10);
    assert_eq!(chrome.procs, 2);
    assert_eq!(chrome.rss, 200);
    assert_eq!(safari.pid, 20);
    assert!(!chrome.stoppable);
    assert!(!safari.stoppable);
}

#[test]
fn services_are_labeled_and_never_enabled_as_renderer_stop_targets() {
    let processes = [
        process(30, "SafariNotificationAgent", &[]),
        process(
            40,
            "Google Chrome Helper (Renderer)",
            &["Google Chrome Helper (Renderer)", "--type=renderer"],
        ),
    ];
    let sessions = sweeploom_session::group_sessions(&processes);
    let rows = collect_rows(&sessions, &processes);
    let service = rows.iter().find(|row| row.pid == 30).unwrap();
    assert_eq!(service.family, "Safari");
    assert!(service.role.starts_with("Background service"));
    assert!(!service.stoppable);
    assert!(rows.iter().find(|row| row.pid == 40).unwrap().stoppable);
}
