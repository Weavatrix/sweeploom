//! Node is a runtime, not a workload. Keep the script, installation and cwd separate.

use std::path::{Path, PathBuf};

use sweeploom_core::{ProcessSnapshot, SessionKind};

#[path = "node_paths.rs"]
mod paths;
pub use paths::{is_tool_installation, version_from_path};

/// Observable identity of one Node process. No script or runtime is executed here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeIdentity {
    /// Actual executable reported by the OS, when available.
    pub executable: Option<PathBuf>,
    /// Version suggested by an installation path (not a runtime version query).
    pub version_hint: Option<String>,
    /// Script resolved against the actual working directory, when available.
    pub entrypoint: Option<PathBuf>,
    /// Package, tool or script name instead of the generic `node` caption.
    pub workload: String,
    /// Signature used for recognized tool classification.
    pub kind: Option<SessionKind>,
}

/// Is this process running the Node executable?
#[must_use]
pub fn is_node(process: &ProcessSnapshot) -> bool {
    [
        Some(process.name.as_str()),
        process.exe.as_ref().and_then(|p| p.file_name()?.to_str()),
    ]
    .into_iter()
    .flatten()
    .any(|name| matches!(name.to_ascii_lowercase().as_str(), "node" | "node.exe"))
}

/// Describe Node using executable/entrypoint evidence, without searching arbitrary arguments.
#[must_use]
pub fn node_identity(process: &ProcessSnapshot) -> Option<NodeIdentity> {
    if !is_node(process) {
        return None;
    }
    let executable = process.exe.clone().or_else(|| {
        process
            .command
            .first()
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
    });
    let argv0 = process.command.first().map(String::as_str).unwrap_or("");
    let script = script_argument(&process.command);
    let entrypoint = script.map(|value| {
        let path = PathBuf::from(value);
        if path.is_relative() {
            process
                .cwd
                .as_ref()
                .map(|cwd| cwd.join(&path))
                .unwrap_or(path)
        } else {
            path
        }
    });
    let script_text = entrypoint
        .as_ref()
        .map(|p| p.to_string_lossy().replace('\\', "/"));
    let package = script_text.as_deref().and_then(package_name);
    let tool = leaf(argv0);
    let cursor_agent = tool == "cursor-agent"
        || script_text
            .as_deref()
            .is_some_and(|p| p.contains("/cursor-agent/versions/") && leaf(p) == "index.js");
    // Some Node tools overwrite argv[0] with their npm command. The remaining
    // tokens can be environment assignments, so they must not become entrypoints.
    let npm_title = argv0
        .strip_prefix("npm exec ")
        .map(|title| title.split_whitespace().next().unwrap_or(title).to_owned())
        .or_else(|| {
            argv0
                .strip_prefix("npm run ")
                .map(|p| format!("npm run {p}"))
        });
    let mut workload = if cursor_agent {
        "Cursor agent".to_owned()
    } else if matches!(tool, "claude" | "codex" | "gemini") {
        tool.to_owned()
    } else if let Some(title) = npm_title {
        title
    } else if let Some(package) = package {
        package
    } else if let Some(path) = script_text.as_deref() {
        script_workload(path)
    } else if process.command.iter().any(|p| {
        matches!(p.as_str(), "-e" | "--eval" | "-p" | "--print") || p.starts_with("--eval=")
    }) {
        "inline script".to_owned()
    } else {
        "runtime / REPL".to_owned()
    };
    // Plugin server.mjs needs the plugin name, not its version folder.
    if let Some(plugin) = plugin_workload(process.cwd.as_deref(), script_text.as_deref()) {
        workload = plugin;
    }
    let kind = if cursor_agent {
        Some(SessionKind::Cursor)
    } else if matches!(tool, "claude" | "codex" | "gemini") {
        Some(match tool {
            "claude" => SessionKind::ClaudeCode,
            "codex" => SessionKind::Codex,
            _ => SessionKind::Gemini,
        })
    } else {
        tool_kind(&workload, script_text.as_deref(), &process.command)
    };
    Some(NodeIdentity {
        version_hint: executable.as_ref().and_then(|p| version_from_path(p)),
        executable,
        entrypoint,
        workload,
        kind,
    })
}

/// Script file name; generic entry files keep their package folder.
fn script_workload(path: &str) -> String {
    let filename = leaf(path);
    if matches!(
        filename,
        "index.js" | "index.mjs" | "server.js" | "server.mjs" | "cli.js" | "cli.mjs"
    ) {
        let parts: Vec<_> = path
            .split('/')
            .filter(|s| !s.is_empty() && *s != ".")
            .collect();
        let parent = parts
            .iter()
            .rev()
            .skip(1)
            .find(|s| !matches!(**s, "dist" | "build" | "bin" | "lib" | "src"));
        parent
            .map(|p| format!("{p} / {filename}"))
            .unwrap_or_else(|| filename.to_owned())
    } else {
        filename.to_owned()
    }
}

/// Plugin name from a `plugins/cache/<vendor>/<plugin>/<version>` cwd.
fn plugin_workload(cwd: Option<&Path>, script: Option<&str>) -> Option<String> {
    let normalized = cwd?.to_string_lossy().replace('\\', "/");
    let (_, suffix) = normalized.split_once("/plugins/cache/")?;
    let parts: Vec<_> = suffix.split('/').collect();
    (parts.len() >= 3
        && script.is_some_and(|p| matches!(leaf(p), "server.mjs" | "server.js" | "index.js")))
    .then(|| parts[1].to_owned())
}

fn leaf(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

fn script_argument(command: &[String]) -> Option<&str> {
    if command.first().is_some_and(|p| p.starts_with("npm ")) {
        return None;
    }
    let mut args = command.iter().skip(1);
    while let Some(arg) = args.next() {
        if matches!(arg.as_str(), "-e" | "--eval" | "-p" | "--print")
            || arg.starts_with("--eval=")
            || arg.starts_with("--print=")
        {
            return None;
        }
        if matches!(
            arg.as_str(),
            "-r" | "--require"
                | "--loader"
                | "--experimental-loader"
                | "--import"
                | "--inspect-port"
                | "--conditions"
                | "-C"
                | "--input-type"
                | "--title"
        ) {
            args.next();
            continue;
        }
        if arg == "--" {
            return args.next().map(String::as_str);
        }
        if arg.starts_with('-') {
            continue;
        }
        if arg.contains('=') || arg.contains("://") || arg == "***" {
            continue;
        }
        return Some(arg);
    }
    None
}

fn package_name(path: &str) -> Option<String> {
    let rest = path.rsplit_once("/node_modules/")?.1;
    let mut parts = rest.split('/');
    let name = parts.next()?;
    if name == ".bin" {
        return parts.next().map(str::to_owned);
    }
    if name.starts_with('@') {
        Some(format!("{name}/{}", parts.next()?))
    } else {
        Some(name.to_owned())
    }
}

fn tool_kind(workload: &str, script: Option<&str>, command: &[String]) -> Option<SessionKind> {
    let first = workload.split_whitespace().next().unwrap_or(workload);
    let name = first
        .rsplit_once('@')
        .filter(|(package, _)| !package.is_empty())
        .map(|(package, _)| package)
        .unwrap_or(first);
    let name = name
        .trim_end_matches(".mjs")
        .trim_end_matches(".cjs")
        .trim_end_matches(".js")
        .trim_end_matches(".ts");
    if name == "@anthropic-ai/claude-code" {
        return Some(SessionKind::ClaudeCode);
    }
    if name == "@openai/codex" {
        return Some(SessionKind::Codex);
    }
    if name == "@google/gemini-cli" {
        return Some(SessionKind::Gemini);
    }
    if matches!(
        name,
        "typescript-language-server" | "vscode-langservers-extracted"
    ) || script.is_some_and(|p| matches!(leaf(p), "tsserver.js" | "eslintServer.js"))
    {
        return Some(SessionKind::LanguageServer);
    }
    if workload.contains("/server-") && workload.starts_with("@modelcontextprotocol/")
        || name.starts_with("mcp-server-")
        || name.ends_with("-mcp")
        || matches!(
            name,
            "mcp-server"
                | "mcp_server"
                | "weavatrix"
                | "weavatrix-mcp"
                | "weavatrix-git"
                | "weavatrix-md"
                | "sweeploom-mcp"
                | "codex-app-tools"
        )
        || workload.starts_with("@playwright/mcp")
        || command.iter().any(|arg| arg == "--mcp" || arg == "mcp")
        || script.is_some_and(|p| p.split('/').any(|c| c == "weavatrix") && leaf(p) == "mcp.js")
    {
        return Some(SessionKind::Mcp);
    }
    if matches!(
        name,
        "vite" | "next" | "nuxt" | "webpack" | "parcel" | "nodemon"
    ) {
        return Some(SessionKind::DevServer);
    }
    if matches!(name, "playwright" | "vitest" | "jest") {
        return Some(SessionKind::TestRunner);
    }
    None
}

#[cfg(test)]
#[path = "node_tests.rs"]
mod tests;
