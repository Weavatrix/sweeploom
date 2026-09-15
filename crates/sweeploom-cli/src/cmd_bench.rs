//! `sweeploom bench` — print WITH vs WITHOUT SweepLoom on a fixed gold store.

use std::path::PathBuf;

use sweeploom_ai::{AiClass, advise_context, gold_tax};
#[cfg(test)]
use sweeploom_ai::{GOLD_DUMP, GOLD_HISTORY_AS_PROMPT, GOLD_WITH};
use sweeploom_session::idle_agent_survive;

use crate::api::{ApplyRequest, apply_cleanup};

/// Print the fixed gold-store comparison.
pub fn run() {
    let report = compare();
    print_report(&report);
}

/// WITH vs WITHOUT numbers the CLI and tests share.
#[derive(Debug)]
pub struct Compare {
    /// Naive agent dumps the whole store into the prompt (`bytes/4`).
    pub dump: u64,
    /// Agent that skips cache/secrets but still bills History.
    pub history_as_prompt: u64,
    /// SweepLoom always-on estimate.
    pub with: u64,
    /// `apply_cleanup` without `confirm`.
    pub apply_without_confirm_deleted: u64,
    /// True when that apply was refused.
    pub apply_refused: bool,
    /// Idle 2 GB agents a RAM-booster would kill.
    pub agents_without_kill: usize,
    /// Those same agents SweepLoom keeps.
    pub agents_with_keep: usize,
}

/// Run the gold-store comparison. Does not touch the user's disk.
#[must_use]
pub fn compare() -> Compare {
    let tax = gold_tax();
    let dry = apply_cleanup(ApplyRequest {
        confirm: false,
        root: PathBuf::from("."),
        ids: Vec::new(),
    });
    let (agents_without_kill, agents_with_keep) = idle_agent_survive();
    Compare {
        dump: tax.dump,
        history_as_prompt: tax.history_as_prompt,
        with: tax.with,
        apply_without_confirm_deleted: dry.deleted,
        apply_refused: !dry.ok,
        agents_without_kill,
        agents_with_keep,
    }
}

fn print_report(report: &Compare) {
    println!("SweepLoom bench — AI WITHOUT vs WITH (fixed gold store, not your disk)");
    println!("WITHOUT dump-store/4          {} tok", report.dump);
    println!(
        "WITHOUT History-as-prompt     {} tok",
        report.history_as_prompt
    );
    println!("WITH    always-on estimate    {} tok", report.with);
    println!(
        "apply confirm=false           refused={} deleted={}",
        report.apply_refused, report.apply_without_confirm_deleted
    );
    println!(
        "idle 2GB agents               WITHOUT would-kill={}  WITH Keep={}",
        report.agents_without_kill, report.agents_with_keep
    );
    println!(
        "AGENTS.md                     {}",
        advise_context("AGENTS.md", AiClass::Context, None).label()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bench_with_without_card() {
        let report = compare();
        eprintln!(
            "WITHOUT dump={} hist={} WITH={} refuse={} deleted={} kill={} keep={}",
            report.dump,
            report.history_as_prompt,
            report.with,
            report.apply_refused,
            report.apply_without_confirm_deleted,
            report.agents_without_kill,
            report.agents_with_keep
        );
        assert_eq!(report.dump, GOLD_DUMP);
        assert_eq!(report.history_as_prompt, GOLD_HISTORY_AS_PROMPT);
        assert_eq!(report.with, GOLD_WITH);
        assert!(report.apply_refused);
        assert_eq!(report.apply_without_confirm_deleted, 0);
        assert_eq!(report.agents_without_kill, 6);
        assert_eq!(report.agents_with_keep, 6);
    }
}
