//! SweepLoom CLI. GUI is the primary surface; this is for scripts and agents.

mod bytes;
mod cmd_browser;
mod cmd_clean;
mod cmd_companion;
mod cmd_projects;
mod cmd_sessions;

use std::env;
use std::path::PathBuf;

use sweeploom_platform::UserLocations;
use sweeploom_storage::{InventoryLimits, scan_inventory};

use bytes::format_bytes;

fn main() {
    let mut args = env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| "help".to_owned());
    match cmd.as_str() {
        "sessions" => cmd_sessions::run(args),
        "browser" => cmd_browser::run(),
        "companion-host" => cmd_companion::run(),
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

fn arg_root(arg: Option<String>) -> PathBuf {
    arg.map(PathBuf::from)
        .unwrap_or_else(|| UserLocations::current().home)
}

fn print_help() {
    eprintln!(
        "\
SweepLoom — reclaim your workstation without losing your workspace

Usage:
  sweeploom sessions [--free-ram GB] [--reduce-cpu PERCENT]
  sweeploom browser
  sweeploom companion-host
  sweeploom scan [path]
  sweeploom projects [path]
  sweeploom clean [path] [--apply]
"
    );
}

fn cmd_scan(root: &std::path::Path) {
    match scan_inventory(root, InventoryLimits::default()) {
        Ok(report) => {
            println!(
                "root={} entries={} projects={} capped={} logical={}",
                report.root.display(),
                report.entries,
                report.projects.len(),
                report.capped,
                format_bytes(report.tree.logical_bytes)
            );
            for child in report.tree.children.iter().take(15) {
                println!(
                    "  {:>10}  {}  {:?}",
                    format_bytes(child.logical_bytes),
                    child.path.display(),
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
