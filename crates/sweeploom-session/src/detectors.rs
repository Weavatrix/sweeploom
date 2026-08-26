//! Rule-based session detectors. No cloud lookup.

use std::path::Path;

use sweeploom_core::{ProcessSnapshot, SessionKind};

/// Evidence produced by a detector.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionEvidence {
    /// Detected kind.
    pub kind: SessionKind,
    /// Detector id.
    pub detector: &'static str,
}

/// Classify a process into an optional session kind.
pub trait SessionDetector {
    /// Detector name.
    fn id(&self) -> &'static str;
    /// Classify one process.
    fn classify(&self, process: &ProcessSnapshot) -> Option<SessionEvidence>;
}

/// Built-in detectors, in priority order (agents before generic node).
#[must_use]
pub fn builtin_detectors() -> Vec<Box<dyn SessionDetector + Send + Sync>> {
    vec![
        Box::new(NamedDetector::new(
            "claude",
            SessionKind::ClaudeCode,
            &["claude", "claude.exe"],
        )),
        Box::new(NamedDetector::new(
            "codex",
            SessionKind::Codex,
            &["codex", "codex.exe"],
        )),
        Box::new(CommandContains::new(
            "mcp",
            SessionKind::Mcp,
            &["mcp-server", "mcp_server", "@modelcontextprotocol"],
        )),
        Box::new(CommandContains::new(
            "vite",
            SessionKind::DevServer,
            &["vite", "next", "nuxt", "webpack", "parcel"],
        )),
        Box::new(NamedDetector::new(
            "cargo",
            SessionKind::Build,
            &["cargo", "cargo.exe", "rustc", "rustc.exe"],
        )),
        Box::new(NamedDetector::new(
            "python-dev",
            SessionKind::DevServer,
            &["uvicorn", "gunicorn", "flask", "django"],
        )),
        Box::new(NamedDetector::new(
            "lsp",
            SessionKind::LanguageServer,
            &[
                "rust-analyzer",
                "rust-analyzer.exe",
                "gopls",
                "typescript-language-server",
            ],
        )),
        Box::new(NamedDetector::new(
            "terminal",
            SessionKind::Terminal,
            &[
                "cmd",
                "cmd.exe",
                "powershell",
                "powershell.exe",
                "pwsh",
                "pwsh.exe",
                "WindowsTerminal.exe",
                "bash",
                "bash.exe",
                "zsh",
                "fish",
                "wt.exe",
            ],
        )),
        Box::new(BrowserDetector),
        Box::new(CommandContains::new(
            "playwright",
            SessionKind::TestRunner,
            &["playwright"],
        )),
    ]
}

/// Run all detectors; first match wins.
#[must_use]
pub fn classify_process(process: &ProcessSnapshot) -> Option<SessionEvidence> {
    for detector in builtin_detectors() {
        if let Some(evidence) = detector.classify(process) {
            return Some(evidence);
        }
    }
    None
}

struct NamedDetector {
    id: &'static str,
    kind: SessionKind,
    names: &'static [&'static str],
}

impl NamedDetector {
    const fn new(id: &'static str, kind: SessionKind, names: &'static [&'static str]) -> Self {
        Self { id, kind, names }
    }
}

impl SessionDetector for NamedDetector {
    fn id(&self) -> &'static str {
        self.id
    }

    fn classify(&self, process: &ProcessSnapshot) -> Option<SessionEvidence> {
        if name_matches(&process.name, process.exe.as_deref(), self.names)
            || command_contains(&process.command, self.names)
        {
            Some(SessionEvidence {
                kind: self.kind,
                detector: self.id,
            })
        } else {
            None
        }
    }
}

struct CommandContains {
    id: &'static str,
    kind: SessionKind,
    needles: &'static [&'static str],
}

impl CommandContains {
    const fn new(id: &'static str, kind: SessionKind, needles: &'static [&'static str]) -> Self {
        Self { id, kind, needles }
    }
}

impl SessionDetector for CommandContains {
    fn id(&self) -> &'static str {
        self.id
    }

    fn classify(&self, process: &ProcessSnapshot) -> Option<SessionEvidence> {
        if command_contains(&process.command, self.needles) {
            Some(SessionEvidence {
                kind: self.kind,
                detector: self.id,
            })
        } else {
            None
        }
    }
}

struct BrowserDetector;

impl SessionDetector for BrowserDetector {
    fn id(&self) -> &'static str {
        "browser"
    }

    fn classify(&self, process: &ProcessSnapshot) -> Option<SessionEvidence> {
        const NAMES: &[&str] = &[
            "chrome",
            "chrome.exe",
            "msedge",
            "msedge.exe",
            "firefox",
            "firefox.exe",
            "brave",
            "brave.exe",
            "safari",
        ];
        if name_matches(&process.name, process.exe.as_deref(), NAMES) {
            Some(SessionEvidence {
                kind: SessionKind::Browser,
                detector: self.id(),
            })
        } else {
            None
        }
    }
}

fn name_matches(name: &str, exe: Option<&Path>, needles: &[&str]) -> bool {
    let file = exe
        .and_then(Path::file_name)
        .and_then(|item| item.to_str())
        .unwrap_or_default();
    needles
        .iter()
        .any(|needle| name.eq_ignore_ascii_case(needle) || file.eq_ignore_ascii_case(needle))
}

fn command_contains(command: &[String], needles: &[&str]) -> bool {
    command.iter().any(|part| {
        let lower = part.to_ascii_lowercase();
        needles.iter().any(|needle| lower.contains(needle))
    })
}

#[cfg(test)]
mod tests {
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
            classify_process(&process("chrome.exe", &["chrome"])).map(|item| item.kind),
            Some(SessionKind::Browser)
        );
        assert_eq!(
            classify_process(&process("powershell.exe", &["powershell"])).map(|item| item.kind),
            Some(SessionKind::Terminal)
        );
    }
}
