//! One catalog for the CLI (hands) and MCP (agents).

/// Desktop product name.
pub const PRODUCT: &str = "SweepLoom";
/// Human CLI name. `--apply` is enough; no second confirm phrase.
pub const CLI_NAME: &str = "sweeploom";
/// MCP registry name for agents.
pub const MCP_SERVER: &str = "io.github.Weavatrix/sweeploom";

/// How a command may write.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteClass {
    /// Inspect / plan only.
    Read,
    /// Human CLI write (`--apply`, companion install). No extra MCP phrase.
    CliWrite,
    /// MCP write. Requires `confirm=true` after a review tool.
    McpGated,
}

/// One command that CLI, MCP, or both may run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tool {
    /// Stable id (`sessions`, `list_ai_stores`).
    pub id: &'static str,
    /// CLI verb, if humans type it.
    pub cli: Option<&'static str>,
    /// Offered to agents over stdio MCP.
    pub mcp: bool,
    /// Write policy.
    pub writes: WriteClass,
}

/// Publish contract.
pub const TOOLS: &[Tool] = &[
    Tool {
        id: "list_sessions",
        cli: Some("sessions"),
        mcp: true,
        writes: WriteClass::Read,
    },
    Tool {
        id: "disk_inventory",
        cli: Some("scan"),
        mcp: true,
        writes: WriteClass::Read,
    },
    Tool {
        id: "list_projects",
        cli: Some("projects"),
        mcp: true,
        writes: WriteClass::Read,
    },
    Tool {
        id: "list_ai_stores",
        cli: Some("ai"),
        mcp: true,
        writes: WriteClass::Read,
    },
    Tool {
        id: "bench_compare",
        cli: Some("bench"),
        mcp: false,
        writes: WriteClass::Read,
    },
    Tool {
        id: "advise_context",
        cli: None,
        mcp: true,
        writes: WriteClass::Read,
    },
    Tool {
        id: "cleanup_candidates",
        cli: Some("clean"),
        mcp: true,
        writes: WriteClass::Read,
    },
    Tool {
        id: "explain_candidate",
        cli: None,
        mcp: true,
        writes: WriteClass::Read,
    },
    Tool {
        id: "list_browser",
        cli: Some("browser"),
        mcp: true,
        writes: WriteClass::Read,
    },
    Tool {
        id: "apply_cleanup",
        cli: None,
        mcp: true,
        writes: WriteClass::McpGated,
    },
    Tool {
        id: "clean_apply",
        cli: Some("clean"),
        mcp: false,
        writes: WriteClass::CliWrite,
    },
    Tool {
        id: "companion_install",
        cli: Some("companion-install"),
        mcp: false,
        writes: WriteClass::CliWrite,
    },
    Tool {
        id: "companion_host",
        cli: Some("companion-host"),
        mcp: false,
        writes: WriteClass::Read,
    },
];

/// MCP tools in catalog order.
pub fn mcp_tools() -> impl Iterator<Item = &'static Tool> {
    TOOLS.iter().filter(|tool| tool.mcp)
}

/// Print the CLI/MCP contract. Not used on the stdio serve path.
pub fn print_catalog() {
    println!("{CLI_NAME}  {PRODUCT}");
    println!("{MCP_SERVER}  mcp_tools={}", mcp_tools().count());
    for tool in TOOLS {
        let surface = match (tool.cli, tool.mcp) {
            (Some(cli), true) => format!("cli={cli} mcp"),
            (Some(cli), false) => format!("cli={cli}"),
            (None, true) => "mcp".to_owned(),
            (None, false) => "hidden".to_owned(),
        };
        let writes = match tool.writes {
            WriteClass::Read => "read",
            WriteClass::CliWrite => "cli-write",
            WriteClass::McpGated => "mcp-confirm",
        };
        println!("  {:<22} {writes:<12} {surface}", tool.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identities_split_hands_and_agents() {
        assert_eq!(CLI_NAME, "sweeploom");
        assert_eq!(PRODUCT, "SweepLoom");
        assert_eq!(MCP_SERVER, "io.github.Weavatrix/sweeploom");
    }

    #[test]
    fn mcp_writes_only_through_confirm() {
        for tool in mcp_tools() {
            assert!(
                matches!(tool.writes, WriteClass::Read | WriteClass::McpGated),
                "{} has {:?}",
                tool.id,
                tool.writes
            );
        }
        let apply = TOOLS
            .iter()
            .find(|tool| tool.id == "apply_cleanup")
            .unwrap();
        assert!(apply.mcp);
        assert_eq!(apply.writes, WriteClass::McpGated);
    }

    #[test]
    fn cli_apply_is_flag_not_mcp_phrase() {
        let apply = TOOLS.iter().find(|tool| tool.id == "clean_apply").unwrap();
        assert!(!apply.mcp);
        assert_eq!(apply.writes, WriteClass::CliWrite);
        assert_eq!(apply.cli, Some("clean"));
    }

    #[test]
    fn weavatrix_marketplace_and_codex_plugin_parse() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let marketplace: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(".agents/plugins/marketplace.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(marketplace["name"], "weavatrix");
        assert_eq!(marketplace["plugins"][0]["name"], "sweeploom");
        assert_eq!(
            marketplace["plugins"][0]["source"]["path"],
            "./plugins/sweeploom"
        );

        let plugin: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("plugins/sweeploom/.codex-plugin/plugin.json"))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(plugin["name"], "sweeploom");
        assert_eq!(plugin["author"]["name"], "Weavatrix");
        assert_eq!(plugin["interface"]["developerName"], "Weavatrix");
        assert_eq!(plugin["mcpServers"], "./.mcp.json");

        let mcp: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join("plugins/sweeploom/.mcp.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(mcp["sweeploom"]["command"], "npx");
        assert_eq!(
            mcp["sweeploom"]["args"],
            serde_json::json!(["-y", "sweeploom", "mcp"])
        );

        let npm: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(root.join("package.json")).unwrap())
                .unwrap();
        assert_eq!(npm["name"], "sweeploom");
        assert_eq!(npm["bin"]["sweeploom"], "npm/cli.js");
        assert_eq!(npm["bin"]["sweeploom-mcp"], "npm/mcp.js");

        let cursor: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(root.join(".cursor/mcp.json")).unwrap())
                .unwrap();
        assert_eq!(cursor["mcpServers"]["sweeploom"]["command"], "sweeploom");
        assert_eq!(
            cursor["mcpServers"]["sweeploom"]["args"],
            serde_json::json!(["mcp"])
        );

        let claude: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(root.join(".mcp.json")).unwrap())
                .unwrap();
        assert_eq!(claude["mcpServers"]["sweeploom"]["command"], "sweeploom");
        assert_eq!(
            claude["mcpServers"]["sweeploom"]["args"],
            serde_json::json!(["mcp"])
        );
    }

    #[test]
    fn cargo_lock_bans_async_http_stack() {
        let lock = std::fs::read_to_string(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock"),
        )
        .unwrap();
        for name in ["tokio", "hyper", "reqwest"] {
            let line = format!("name = \"{name}\"");
            assert!(
                !lock.lines().any(|item| item.trim() == line),
                "{name} is banned in deny.toml but is in Cargo.lock"
            );
        }
    }

    #[test]
    fn published_mcp_surface() {
        let ids: Vec<_> = mcp_tools().map(|tool| tool.id).collect();
        assert_eq!(
            ids,
            [
                "list_sessions",
                "disk_inventory",
                "list_projects",
                "list_ai_stores",
                "advise_context",
                "cleanup_candidates",
                "explain_candidate",
                "list_browser",
                "apply_cleanup",
            ]
        );
    }
}
