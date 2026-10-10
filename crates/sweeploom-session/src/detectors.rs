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
        Box::new(BrowserDetector),
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
        Box::new(NamedDetector::new(
            "cursor",
            SessionKind::Cursor,
            &["cursor", "cursor.exe"],
        )),
        Box::new(NamedDetector::new(
            "opencode",
            SessionKind::OpenCode,
            &["opencode", "opencode.exe"],
        )),
        Box::new(NamedDetector::new(
            "gemini",
            SessionKind::Gemini,
            &["gemini", "gemini.exe"],
        )),
        Box::new(NamedDetector::new(
            "grok",
            SessionKind::Grok,
            &["grok", "grok.exe"],
        )),
        Box::new(NamedDetector::new(
            "mcp-bin",
            SessionKind::Mcp,
            &[
                "mcp-server",
                "mcp-server.exe",
                "mcp_server",
                "mcp_server.exe",
                "weavatrix",
                "weavatrix.exe",
                "weavatrix-mcp",
                "weavatrix-mcp.exe",
                "weavatrix-git",
                "weavatrix-git.exe",
                "weavatrix-md",
                "weavatrix-md.exe",
                "sweeploom-mcp",
                "sweeploom-mcp.exe",
            ],
        )),
        Box::new(CommandContains::new(
            "mcp",
            SessionKind::Mcp,
            &[
                "mcp-server",
                "mcp_server",
                "@modelcontextprotocol",
                "--mcp",
                "weavatrix",
                "weavatrix-mcp",
                "weavatrix-git",
                "weavatrix-md",
                "sweeploom-mcp",
            ],
        )),
        Box::new(CommandContains::new(
            "vite",
            SessionKind::DevServer,
            &[
                "vite", "next", "nuxt", "webpack", "parcel", "nodemon", "tsx", "esbuild",
            ],
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
    if let Some(identity) = crate::node_identity(process) {
        // Loader arguments and arbitrary project path names are not workload
        // signatures. Node is classified from its actual entrypoint only.
        return identity.kind.map(|kind| SessionEvidence {
            kind,
            detector: "node-workload",
        });
    }
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
        if name_matches(&process.name, process.exe.as_deref(), self.names) {
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
        if command_contains_entrypoint(&process.command, self.needles) {
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
        if sweeploom_core::browser_identity(process).is_some() {
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

fn command_contains_entrypoint(command: &[String], needles: &[&str]) -> bool {
    command.iter().skip(1).any(|part| {
        if looks_like_url(part) {
            return false;
        }
        let lower = part.to_ascii_lowercase();
        needles.iter().any(|needle| {
            if looks_like_entrypoint(part) {
                lower.contains(needle)
            } else {
                token_is_needle(&lower, needle)
            }
        })
    })
}

fn token_is_needle(part: &str, needle: &str) -> bool {
    let name = part
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(part)
        .trim_end_matches(".exe");
    name == needle || name == format!("{needle}.js")
}

fn looks_like_url(part: &str) -> bool {
    let lower = part.to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("file:")
        || lower.contains("://")
}

fn looks_like_entrypoint(part: &str) -> bool {
    part.contains('/')
        || part.contains('\\')
        || part.ends_with(".js")
        || part.ends_with(".mjs")
        || part.ends_with(".cjs")
        || part.ends_with(".ts")
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
            classify_process(&process("sweeploom-mcp.exe", &["sweeploom-mcp"]))
                .map(|item| item.kind),
            Some(SessionKind::Mcp)
        );
        assert_eq!(
            classify_process(&process("sweeploom.exe", &["sweeploom", "scan"]))
                .map(|item| item.kind),
            None
        );
    }
}
