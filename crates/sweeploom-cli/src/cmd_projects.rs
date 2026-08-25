//! `sweeploom projects` — heat, git safety, cargo/node offers.

use std::path::Path;

use sweeploom_dev::{cargo_offers, classify_project, inspect, node_offers, python_offers};
use sweeploom_storage::{InventoryLimits, scan_inventory};

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
        println!(
            "{}\tkind={:?}\tsource={:?}\tartifact={:?}\tgit={}",
            project.display(),
            classify_project(project),
            source,
            artifact,
            inspect(project).label()
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
        for offer in node_offers(project, &[]) {
            println!(
                "  node_modules\t{}\trebuild={:?}{}",
                format_bytes(offer.logical_bytes),
                offer.rebuild,
                if offer.blocked { "\tBLOCKED" } else { "" }
            );
        }
        for offer in python_offers(project, &[]) {
            println!(
                "  python {}\t{}\trebuild={:?}{}",
                offer.label,
                format_bytes(offer.logical_bytes),
                offer.rebuild,
                if offer.blocked { "\tBLOCKED" } else { "" }
            );
        }
    }
}
