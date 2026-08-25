//! SweepLoom CLI. GUI is the primary surface; this is for scripts and agents.

mod bytes;
mod cmd_clean;
mod cmd_projects;

use std::env;
use std::path::PathBuf;
use std::time::Duration;

use sweeploom_network::enrich_network;
use sweeploom_platform::UserLocations;
use sweeploom_process::ProcessSampler;
use sweeploom_session::{AttributionRoots, sessions_from_snapshot};
use sweeploom_storage::{InventoryLimits, scan_inventory};

use bytes::format_bytes;

fn main() {
    let mut args = env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| "help".to_owned());
    match cmd.as_str() {
        "sessions" => cmd_sessions(),
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
  sweeploom sessions
  sweeploom scan [path]
  sweeploom projects [path]
  sweeploom clean [path] [--apply]
"
    );
}

fn cmd_sessions() {
    let mut sampler = ProcessSampler::new();
    let mut snapshot = sampler.refresh(Duration::from_millis(200));
    snapshot.resolve_parents();
    let _capability = enrich_network(&mut snapshot.processes);
    let locations = UserLocations::current();
    let roots = AttributionRoots {
        projects: vec![locations.home.clone()],
        current_project: None,
    };
    let sessions = sessions_from_snapshot(&mut snapshot, &roots);
    println!(
        "processes={} sessions={} rss_total={}",
        snapshot.processes.len(),
        sessions.len(),
        format_bytes(snapshot.total_rss_bytes)
    );
    for session in sessions.iter().take(20) {
        let project = session
            .project
            .as_ref()
            .map(|item| item.0.display().to_string())
            .unwrap_or_else(|| "-".to_owned());
        println!(
            "{:<18} proc={:<3} rss={:<10} cpu={:>5.1}%  rec={:?}  {}",
            session.label(),
            session.processes.len(),
            format_bytes(session.rss_bytes),
            session.cpu_percent,
            session.recommendation.recommendation,
            project
        );
    }
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
