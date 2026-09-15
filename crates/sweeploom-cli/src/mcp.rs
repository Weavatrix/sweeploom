//! MCP stdio server via mcport. Apply is gated; the CLI is not this path.

use std::path::PathBuf;

use mcport::{McpServer, ToolError, tool, tools};
use sweeploom_platform::UserLocations;

use crate::api::{self, ApplyRequest};
use crate::surface;

fn root_or_home(root: Option<String>) -> PathBuf {
    root.filter(|item| !item.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| UserLocations::current().home)
}

fn json<T: serde::Serialize>(value: &T) -> Result<String, ToolError> {
    serde_json::to_string_pretty(value).map_err(|error| ToolError::new(error.to_string()))
}

/// List live workstation sessions. Read-only. Does not terminate processes.
#[tool(read_only)]
fn list_sessions() -> Result<String, ToolError> {
    json(&api::list_sessions())
}

/// Scan a folder tree (metadata only). Default root is the user home.
#[tool(read_only)]
fn disk_inventory(root: Option<String>) -> Result<String, ToolError> {
    json(&api::disk_inventory(&root_or_home(root)).map_err(ToolError::new)?)
}

/// List discovered projects with source/artifact heat and git safety.
#[tool(read_only)]
fn list_projects(root: Option<String>) -> Result<String, ToolError> {
    json(&api::list_projects(&root_or_home(root)).map_err(ToolError::new)?)
}

/// Inspect local AI stores. Never opens secrets or SQLite.
#[tool(read_only)]
fn list_ai_stores() -> Result<String, ToolError> {
    json(&api::list_ai_stores())
}

/// Advise always-on context (AGENTS.md / skills). Never writes files.
#[tool(read_only)]
fn advise_context(relative: String) -> Result<String, ToolError> {
    let class = sweeploom_ai::classify_name(&relative);
    let advice = api::advise_one(&relative, class, None);
    json(&serde_json::json!({
        "relative": relative,
        "class": class.label(),
        "advice": advice.label(),
    }))
}

/// List generated cleanup candidates. Dry-run. Review before apply_cleanup.
#[tool(read_only)]
fn cleanup_candidates(root: Option<String>) -> Result<String, ToolError> {
    json(&api::cleanup_candidates(&root_or_home(root)))
}

/// Explain one cleanup candidate by id from cleanup_candidates or list_ai_stores.
#[tool(read_only)]
fn explain_candidate(id: u64, root: Option<String>) -> Result<String, ToolError> {
    let found = api::explain_candidate(&root_or_home(root), id)
        .ok_or_else(|| ToolError::new(format!("unknown candidate {id}")))?;
    json(&found)
}

/// Browser process pressure. Tab counts only with a fresh companion.
#[tool(read_only)]
fn list_browser() -> Result<String, ToolError> {
    json(&api::list_browser())
}

/// Apply a reviewed cleanup. Requires confirm=true. Does not terminate processes.
#[tool]
fn apply_cleanup(
    confirm: bool,
    root: Option<String>,
    ids: Option<Vec<u64>>,
) -> Result<String, ToolError> {
    let report = api::apply_cleanup(ApplyRequest {
        confirm,
        root: root_or_home(root),
        ids: ids.unwrap_or_default(),
    });
    if !report.ok {
        return Err(ToolError::new(
            report.error.unwrap_or_else(|| "apply refused".to_owned()),
        ));
    }
    json(&report)
}

/// The served tool set. One place, so tests see what agents see.
fn server() -> McpServer {
    McpServer::new(surface::PRODUCT, env!("CARGO_PKG_VERSION")).tools(tools![
        list_sessions,
        disk_inventory,
        list_projects,
        list_ai_stores,
        advise_context,
        cleanup_candidates,
        explain_candidate,
        list_browser,
        apply_cleanup
    ])
}

/// Serve MCP on stdio.
pub fn serve() -> std::io::Result<()> {
    server().serve()
}

pub fn run(args: impl Iterator<Item = String>) {
    let items: Vec<String> = args.collect();
    if items
        .iter()
        .any(|item| matches!(item.as_str(), "--list" | "list" | "--help" | "-h" | "help"))
    {
        crate::surface::print_catalog();
        return;
    }
    if let Err(error) = serve() {
        eprintln!("sweeploom mcp: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::{WriteClass, mcp_tools};

    fn served_tools() -> Vec<serde_json::Value> {
        let mut server = server();
        let mut output = Vec::new();
        mcport::serve_message(
            &mut server,
            r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#,
            &mut output,
        )
        .unwrap();
        let catalog: serde_json::Value = serde_json::from_slice(&output).unwrap();
        catalog["result"]["tools"].as_array().unwrap().clone()
    }

    /// `surface::TOOLS` promises agents which tools only read. The served
    /// `tools/list` must advertise the same thing, or hosts treat every
    /// inspect tool as a possible write and gate it like `apply_cleanup`.
    #[test]
    fn served_annotations_match_surface_write_class() {
        let served = served_tools();
        for tool in mcp_tools() {
            let row = served
                .iter()
                .find(|item| item["name"] == tool.id)
                .unwrap_or_else(|| panic!("{} is in surface::TOOLS but not served", tool.id));
            let read_only = row["annotations"]["readOnlyHint"] == true;
            match tool.writes {
                WriteClass::Read => assert!(
                    read_only,
                    "{} is WriteClass::Read but is served without readOnlyHint",
                    tool.id
                ),
                WriteClass::McpGated | WriteClass::CliWrite => {
                    assert!(!read_only, "{} writes but is served as read-only", tool.id)
                }
            }
        }
    }
}
