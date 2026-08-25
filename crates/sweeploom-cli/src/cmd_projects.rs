//! `sweeploom projects` — heat, git safety, cargo offers.

use std::path::Path;

use sweeploom_dev::{cargo_offers, classify_project, inspect};
use sweeploom_storage::{InventoryLimits, scan_inventory};
use weavatrix_git::WorktreeSafetyLevel;

use crate::bytes::format_bytes;

pub fn run(root: &Path) {
    let report = match scan_inventory(root, InventoryLimits::default()) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("scan failed: {error}");
            std::process::exit(1);
        }
    };
    let now = std::time::SystemTime::now();
    for project in &report.projects {
        let (source, artifact) = report.project_heat(project, now);
        let git = inspect(project);
        println!(
            "{}\tkind={:?}\tsource={:?}\tartifact={:?}\tgit={}",
            project.display(),
            classify_project(project),
            source,
            artifact,
            git_label(&git)
        );
        for offer in cargo_offers(project, &[]) {
            println!(
                "  cargo {:?}\t{}\trebuild={:?}{}",
                offer.mode,
                format_bytes(offer.logical_bytes),
                offer.rebuild,
                if offer.blocked { "\tBLOCKED" } else { "" }
            );
        }
    }
}

fn git_label(git: &sweeploom_dev::GitSafety) -> &'static str {
    match git {
        sweeploom_dev::GitSafety::NotARepository => "none",
        sweeploom_dev::GitSafety::Unknown => "unknown",
        sweeploom_dev::GitSafety::Known(safety) => match safety.level {
            WorktreeSafetyLevel::Clean => "clean",
            WorktreeSafetyLevel::IgnoredOnly => "ignored-only",
            WorktreeSafetyLevel::HasUntracked => "untracked",
            WorktreeSafetyLevel::DirtyTracked => "dirty",
            WorktreeSafetyLevel::Unknown => "unknown",
        },
    }
}
