use super::*;
use crate::tests::proc;

#[test]
fn cursor_worker_is_an_agent_with_actual_workspace_not_version_project() {
    let mut p = proc(
        7,
        None,
        "node",
        Some("/work/ipad"),
        &[
            "/tools/cursor-agent",
            "--use-system-ca",
            "/tools/cursor-agent/versions/2026.09.26-dd393fe/index.js",
            "worker",
            "start",
        ],
        1,
        0.0,
    );
    p.exe = Some(PathBuf::from(
        "/tools/cursor-agent/versions/2026.09.26-dd393fe/node",
    ));
    let identity = node_identity(&p).unwrap();
    assert_eq!(identity.workload, "Cursor agent");
    assert_eq!(identity.kind, Some(SessionKind::Cursor));
    assert_eq!(identity.version_hint, None);
}

#[test]
fn skips_node_loaders_and_keeps_real_script() {
    let mut p = proc(
        7,
        None,
        "node",
        Some("/work/demo"),
        &[
            "node",
            "--require",
            "/tools/tsx/preflight.cjs",
            "--import",
            "file:///tools/tsx/loader.mjs",
            "./monitor.ts",
            "--token",
            "***",
        ],
        1,
        0.0,
    );
    p.exe = Some(PathBuf::from(
        "/home/me/.nvm/versions/node/v22.13.1/bin/node",
    ));
    let identity = node_identity(&p).unwrap();
    assert_eq!(
        identity.entrypoint,
        Some(PathBuf::from("/work/demo/./monitor.ts"))
    );
    assert_eq!(identity.workload, "monitor.ts");
    assert_eq!(identity.version_hint.as_deref(), Some("v22.13.1"));
    assert_eq!(identity.kind, None);
}

#[test]
fn overwritten_npm_title_does_not_parse_environment_as_script() {
    let p = proc(
        7,
        None,
        "node",
        None,
        &[
            "npm exec @playwright/mcp@latest",
            "HOME=/home/me",
            "PATH=/tools/node",
        ],
        1,
        0.0,
    );
    let identity = node_identity(&p).unwrap();
    assert_eq!(identity.entrypoint, None);
    assert_eq!(identity.workload, "@playwright/mcp@latest");
    assert_eq!(identity.kind, Some(SessionKind::Mcp));
}

#[test]
fn plugin_identity_includes_plugin_not_its_version() {
    let p = proc(
        7,
        None,
        "node",
        Some("/home/me/.codex/plugins/cache/openai-bundled/codex-app-tools/0.1.5"),
        &["node", "./server.mjs"],
        1,
        0.0,
    );
    let identity = node_identity(&p).unwrap();
    assert_eq!(identity.workload, "codex-app-tools");
    assert_eq!(identity.kind, Some(SessionKind::Mcp));
}

#[test]
fn node_agent_package_is_recognized_and_inline_code_is_not_a_workload_name() {
    let p = proc(
        7,
        None,
        "node",
        None,
        &[
            "node",
            "/tools/node_modules/@anthropic-ai/claude-code/cli.js",
        ],
        1,
        0.0,
    );
    assert_eq!(
        node_identity(&p).unwrap().kind,
        Some(SessionKind::ClaudeCode)
    );
    let p = proc(
        7,
        None,
        "node",
        None,
        &["node", "-e", "console.log('vite mcp')"],
        1,
        0.0,
    );
    assert_eq!(node_identity(&p).unwrap().workload, "inline script");
    assert_eq!(node_identity(&p).unwrap().kind, None);
}
