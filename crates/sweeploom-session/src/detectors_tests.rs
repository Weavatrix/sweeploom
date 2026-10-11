use super::*;
use std::time::Duration;

use sweeploom_core::{NetworkSnapshot, ProcessKey, ProcessSafetyClass};

fn process(name: &str, command: &[&str]) -> ProcessSnapshot {
    ProcessSnapshot {
        key: ProcessKey {
            pid: 7,
            started_at_unix_ms: 1,
        },
        pid: 7,
        parent: None,
        name: name.to_owned(),
        exe: None,
        cwd: None,
        command: command.iter().map(|item| (*item).to_owned()).collect(),
        started_at: None,
        runtime: Duration::from_secs(1),
        rss_bytes: 1,
        virtual_bytes: 1,
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
fn detects_claude_and_vite() {
    assert_eq!(
        classify_process(&process("claude.exe", &["claude"])).map(|item| item.kind),
        Some(SessionKind::ClaudeCode)
    );
    assert_eq!(
        classify_process(&process(
            "node.exe",
            &["node", "./node_modules/vite/bin/vite.js"]
        ))
        .map(|item| item.kind),
        Some(SessionKind::DevServer)
    );
    assert_eq!(
        classify_process(&process(
            "chrome.exe",
            &["chrome", "https://example.test/docs/claude"]
        ))
        .map(|item| item.kind),
        Some(SessionKind::Browser)
    );
    assert_eq!(
        classify_process(&process("chrome.exe", &["chrome"])).map(|item| item.kind),
        Some(SessionKind::Browser)
    );
    assert_eq!(
        classify_process(&process("powershell.exe", &["powershell"])).map(|item| item.kind),
        Some(SessionKind::Terminal)
    );
    assert_eq!(
        classify_process(&process("Cursor.exe", &["Cursor"])).map(|item| item.kind),
        Some(SessionKind::Cursor)
    );
    assert_eq!(
        classify_process(&process("opencode.exe", &["opencode"])).map(|item| item.kind),
        Some(SessionKind::OpenCode)
    );
    assert_eq!(
        classify_process(&process(
            "node.exe",
            &["node", "--mcp", "C:\\tools\\server.js"]
        ))
        .map(|item| item.kind),
        Some(SessionKind::Mcp)
    );
    assert_eq!(
        classify_process(&process("grok.exe", &["grok"])).map(|item| item.kind),
        Some(SessionKind::Grok)
    );
    assert_eq!(
        classify_process(&process("weavatrix.exe", &["weavatrix", "mcp", "."]))
            .map(|item| item.kind),
        Some(SessionKind::Mcp)
    );
    assert_eq!(
        classify_process(&process(
            "node.exe",
            &["node", "C:\\npm\\weavatrix\\bin\\mcp.js"]
        ))
        .map(|item| item.kind),
        Some(SessionKind::Mcp)
    );
    assert_eq!(
        classify_process(&process("sweeploom-mcp.exe", &["sweeploom-mcp"])).map(|item| item.kind),
        Some(SessionKind::Mcp)
    );
    assert_eq!(
        classify_process(&process("sweeploom.exe", &["sweeploom", "scan"])).map(|item| item.kind),
        None
    );
}
