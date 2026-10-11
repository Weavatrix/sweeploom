//! Rule-based session detectors. No cloud lookup.

use std::path::Path;

use sweeploom_core::{ProcessSnapshot, SessionKind};

#[path = "detector_names.rs"]
mod names;
use names::{DEV_SERVER_NEEDLES, LANGUAGE_SERVERS, MCP_BINARIES, MCP_NEEDLES, TERMINALS};

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
            MCP_BINARIES,
        )),
        Box::new(CommandContains::new("mcp", SessionKind::Mcp, MCP_NEEDLES)),
        Box::new(CommandContains::new(
            "vite",
            SessionKind::DevServer,
            DEV_SERVER_NEEDLES,
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
            LANGUAGE_SERVERS,
        )),
        Box::new(NamedDetector::new(
            "terminal",
            SessionKind::Terminal,
            TERMINALS,
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
#[path = "detectors_tests.rs"]
mod tests;
