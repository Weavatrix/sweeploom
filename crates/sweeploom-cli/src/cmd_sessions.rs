//! `sweeploom sessions` — live grouping plus optional RAM/CPU/quiet plan.

use std::time::Duration;

use sweeploom_core::{ProjectId, SessionId};
use sweeploom_network::enrich_network;
use sweeploom_platform::UserLocations;
use sweeploom_process::ProcessSampler;
use sweeploom_session::{
    AttributionRoots, plan_free_ram, plan_quiet_workstation, plan_reduce_cpu,
    sessions_from_snapshot,
};

use crate::bytes::format_bytes;

pub fn run(args: impl Iterator<Item = String>) {
    let (free_ram, reduce_cpu, quiet) = parse(args);
    let mut sampler = ProcessSampler::new();
    let mut snapshot = sampler.refresh(Duration::from_millis(200));
    snapshot.resolve_parents();
    let _capability = enrich_network(&mut snapshot.processes);
    let locations = UserLocations::current();
    let current_project = std::env::current_dir().ok().map(ProjectId);
    let roots = AttributionRoots {
        projects: vec![locations.home.clone()],
        current_project: current_project.clone(),
    };
    let sessions = sessions_from_snapshot(&mut snapshot, &roots);
    println!(
        "processes={} sessions={} rss_total={}",
        snapshot.processes.len(),
        sessions.len(),
        format_bytes(snapshot.total_rss_bytes)
    );
    let planned = planned_ids(
        &sessions,
        current_project.as_ref(),
        free_ram,
        reduce_cpu,
        quiet,
    );
    for session in sessions.iter().take(20) {
        let project = session
            .project
            .as_ref()
            .map(|item| item.0.display().to_string())
            .unwrap_or_else(|| "-".to_owned());
        let mark = if planned.contains(&session.id) {
            "[x]"
        } else {
            "[ ]"
        };
        println!(
            "{mark} {:<18} proc={:<3} rss={:<10} cpu={:>5.1}%  rec={:?}  {}",
            session.label(),
            session.processes.len(),
            format_bytes(session.rss_bytes),
            session.cpu_percent,
            session.recommendation.recommendation,
            project
        );
    }
    if quiet || free_ram.is_some() || reduce_cpu.is_some() {
        println!(
            "plan={} session(s); dry-run only — terminate stays in the GUI after confirm",
            planned.len()
        );
    }
}

fn planned_ids(
    sessions: &[sweeploom_core::LiveSession],
    current_project: Option<&ProjectId>,
    free_ram: Option<f64>,
    reduce_cpu: Option<f32>,
    quiet: bool,
) -> Vec<SessionId> {
    if quiet {
        return plan_quiet_workstation(sessions, current_project);
    }
    let mut ids = Vec::new();
    if let Some(gb) = free_ram {
        ids.extend(plan_free_ram(sessions, (gb * 1_000_000_000.0) as u64));
    }
    if let Some(cpu) = reduce_cpu {
        for id in plan_reduce_cpu(sessions, cpu) {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    ids
}

fn parse(args: impl Iterator<Item = String>) -> (Option<f64>, Option<f32>, bool) {
    let items: Vec<String> = args.collect();
    let mut free_ram = None;
    let mut reduce_cpu = None;
    let mut quiet = false;
    let mut index = 0;
    while index < items.len() {
        match items[index].as_str() {
            "--free-ram" => {
                index += 1;
                free_ram = items.get(index).and_then(|item| item.parse().ok());
            }
            "--reduce-cpu" => {
                index += 1;
                reduce_cpu = items.get(index).and_then(|item| item.parse().ok());
            }
            "--quiet" => quiet = true,
            other => {
                eprintln!("unknown sessions flag: {other}");
                std::process::exit(2);
            }
        }
        index += 1;
    }
    (free_ram, reduce_cpu, quiet)
}
