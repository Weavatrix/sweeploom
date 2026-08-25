//! Safety class from process name/exe. Conservative: unknown elevated OS
//! processes stay unknown, never auto-recommended.

use std::path::Path;

use sweeploom_core::ProcessSafetyClass;

const SYSTEM_CRITICAL: &[&str] = &[
    "System",
    "Registry",
    "smss.exe",
    "csrss.exe",
    "wininit.exe",
    "winlogon.exe",
    "services.exe",
    "lsass.exe",
    "svchost.exe",
    "explorer.exe",
    "dwm.exe",
    "sihost.exe",
    "fontdrvhost.exe",
    "init",
    "systemd",
    "launchd",
    "kernel_task",
    "WindowServer",
    "loginwindow",
    "Finder",
    "Dock",
];

const DEVELOPER_TOOLS: &[&str] = &[
    "cargo",
    "cargo.exe",
    "rustc",
    "rustc.exe",
    "rust-analyzer",
    "rust-analyzer.exe",
    "node",
    "node.exe",
    "bun",
    "bun.exe",
    "npm",
    "npm.exe",
    "pnpm",
    "pnpm.exe",
    "yarn",
    "yarn.exe",
    "python",
    "python.exe",
    "python3",
    "uv",
    "uv.exe",
    "java",
    "java.exe",
    "dotnet",
    "dotnet.exe",
    "go",
    "go.exe",
    "gopls",
    "cmake",
    "ninja",
    "clang",
    "gcc",
];

const AGENTS: &[&str] = &[
    "claude",
    "claude.exe",
    "codex",
    "codex.exe",
    "opencode",
    "opencode.exe",
    "cursor",
    "cursor.exe",
    "gemini",
    "gemini.exe",
];

/// Classify a process from its image name and optional executable path.
#[must_use]
pub fn classify_process(name: &str, exe: Option<&Path>) -> ProcessSafetyClass {
    if matches_any(name, exe, SYSTEM_CRITICAL) {
        return ProcessSafetyClass::SystemCritical;
    }
    if matches_any(name, exe, AGENTS) {
        return ProcessSafetyClass::Agent;
    }
    if matches_any(name, exe, DEVELOPER_TOOLS) {
        return ProcessSafetyClass::DeveloperTool;
    }
    ProcessSafetyClass::Unknown
}

fn matches_any(name: &str, exe: Option<&Path>, needles: &[&str]) -> bool {
    let file_stem = exe
        .and_then(Path::file_name)
        .and_then(|item| item.to_str())
        .unwrap_or_default();
    needles
        .iter()
        .any(|needle| name.eq_ignore_ascii_case(needle) || file_stem.eq_ignore_ascii_case(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lsass_is_critical() {
        assert_eq!(
            classify_process("lsass.exe", None),
            ProcessSafetyClass::SystemCritical
        );
    }

    #[test]
    fn claude_is_agent() {
        assert_eq!(
            classify_process("claude.exe", Some(Path::new(r"C:\Users\me\claude.exe"))),
            ProcessSafetyClass::Agent
        );
    }
}
