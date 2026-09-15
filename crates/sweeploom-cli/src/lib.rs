//! SweepLoom library: inspect a workstation, advise context, apply a reviewed plan.
//!
//! SweepLoom is a [Weavatrix](https://github.com/Weavatrix) product.
//!
//! - **CLI** (`sweeploom`) is for hands. `--apply` is the consent.
//! - **MCP** (`sweeploom mcp`) is for agents. Apply requires `confirm=true`.
//!
//! Neither surface terminates live agents. Neither rewrites `AGENTS.md`.
//!
//! ```
//! use sweeploom::{classify_name, estimated_prompt_tokens, AiClass};
//!
//! assert_eq!(classify_name("AGENTS.md").label(), "Context");
//! assert!(!classify_name(".credentials.json").can_clean());
//! assert_eq!(
//!     estimated_prompt_tokens(AiClass::History, "history.jsonl", 2_000_000),
//!     0
//! );
//! ```
//!
//! ```
//! use std::time::Duration;
//! use sweeploom::{advise_one, AiClass, ContextAdvice};
//!
//! assert_eq!(
//!     advise_one("AGENTS.md", AiClass::Context, Some(Duration::from_secs(90 * 24 * 3600))),
//!     ContextAdvice::Keep
//! );
//! ```
//!
//! More copy-paste samples: the crate README and `docs/LIBRARY.md`.

#![cfg_attr(not(test), warn(missing_docs))]

pub mod api;
pub mod bytes;
pub mod surface;

#[cfg(test)]
mod agent_cases;
mod cmd_ai;
mod cmd_bench;
mod cmd_browser;
mod cmd_clean;
mod cmd_companion;
mod cmd_companion_install;
mod cmd_projects;
mod cmd_sessions;
#[cfg(feature = "mcp")]
mod mcp;

pub use api::{
    AiChildView, AiStoreView, ApplyReport, ApplyRequest, BrowserView, CandidateView, DiskInventory,
    ProjectView, SessionView, advise_one, apply_cleanup, cleanup_candidates, disk_inventory,
    explain_candidate, list_ai_stores, list_browser, list_projects, list_sessions,
    list_sessions_plan,
};
pub use surface::{
    CLI_NAME, MCP_SERVER, PRODUCT, TOOLS, Tool, WriteClass, mcp_tools, print_catalog,
};
pub use sweeploom_ai::{
    AiClass, ContextAdvice, advise_context, classify_name, estimated_prompt_tokens,
    estimated_prompt_tokens_with_files, inspect_offers,
};

/// Run the `sweeploom` CLI.
pub fn run_cli(args: impl Iterator<Item = String>) {
    let mut args = args.peekable();
    let cmd = args.next().unwrap_or_else(|| "help".to_owned());
    match cmd.as_str() {
        "sessions" => cmd_sessions::run(args),
        "ai" => cmd_ai::run(),
        "bench" => cmd_bench::run(),
        "mcp" => {
            #[cfg(feature = "mcp")]
            crate::mcp::run(args);
            #[cfg(not(feature = "mcp"))]
            {
                eprintln!("this build was compiled without the mcp feature");
                std::process::exit(2);
            }
        }
        "browser" => cmd_browser::run(),
        "companion-host" => cmd_companion::run(),
        "companion-install" => cmd_companion_install::run(args),
        "scan" => cmd_scan(&arg_root(args.next())),
        "projects" => cmd_projects::run(&arg_root(args.next())),
        "clean" => {
            let rest: Vec<String> = args.collect();
            let apply = rest.iter().any(|item| item == "--apply");
            let root = rest.into_iter().find(|item| item != "--apply");
            cmd_clean::run(&arg_root(root), apply);
        }
        "help" | "--help" | "-h" => print_help(),
        other => {
            eprintln!("unknown command: {other}");
            print_help();
            std::process::exit(2);
        }
    }
}

fn arg_root(arg: Option<String>) -> std::path::PathBuf {
    arg.map(std::path::PathBuf::from)
        .unwrap_or_else(|| sweeploom_platform::UserLocations::current().home)
}

fn print_help() {
    eprintln!(
        "\
SweepLoom — reclaim your workstation without losing your workspace
CLI is for hands (`--apply` is consent). MCP is for agents (confirm=true on apply).

Usage:
  sweeploom sessions [--free-ram GB] [--reduce-cpu PERCENT] [--quiet]
  sweeploom ai
  sweeploom bench
  sweeploom mcp [--list]
  sweeploom browser
  sweeploom companion-host
  sweeploom companion-install [--chromium-id ID from edge://extensions]
  sweeploom scan [path]
  sweeploom projects [path]
  sweeploom clean [path] [--apply]
"
    );
}

fn cmd_scan(root: &std::path::Path) {
    match disk_inventory(root) {
        Ok(report) => {
            println!(
                "root={} entries={} projects={} capped={} logical={}",
                report.root,
                report.entries,
                report.projects,
                report.capped,
                bytes::format_bytes(report.logical_bytes)
            );
            for child in report.children.iter().take(15) {
                println!(
                    "  {:>10}  {}  {}",
                    bytes::format_bytes(child.logical_bytes),
                    child.path,
                    child.category
                );
            }
        }
        Err(error) => {
            eprintln!("scan failed: {error}");
            std::process::exit(1);
        }
    }
}
