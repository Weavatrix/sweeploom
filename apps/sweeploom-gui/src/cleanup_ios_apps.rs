//! Installed simulator apps matched to local Xcode projects and their Git state.
use super::{HashMap, Item, Kind, Path, Sim, Verdict, pretty_runtime, verdict, xcode};
use std::fs;
use verdict::AppEvidence;

/// Bundle identifiers of apps installed on a simulator (`data/Containers/Bundle/Application`).
pub(super) fn installed(data: &Path) -> Vec<String> {
    let mut ids: Vec<String> = fs::read_dir(data.join("Containers/Bundle/Application"))
        .into_iter()
        .flatten()
        .flatten()
        .flat_map(|container| {
            fs::read_dir(container.path())
                .into_iter()
                .flatten()
                .flatten()
        })
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "app"))
        .filter_map(|app| xcode::plist::read(&app.join("Info.plist"), "CFBundleIdentifier"))
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

pub(super) fn status(sim: &Sim, apps: &[AppEvidence], now: i64) -> String {
    if sim.loading {
        return Verdict::Review.status(&format!(
            "{} is installed but not loaded right now; refresh before deciding · {}",
            pretty_runtime(&sim.runtime),
            sim.state
        ));
    }
    let running = !sim.state.is_empty() && sim.state != "Shutdown";
    let (verdict, detail) = verdict::device(
        running,
        sim.unavailable.as_deref(),
        sim.last_used,
        apps,
        now,
    );
    let state = if sim.state.is_empty() {
        "state unknown (simctl unavailable)"
    } else {
        sim.state.as_str()
    };
    verdict.status(&format!(
        "{detail} · {} · {state}",
        pretty_runtime(&sim.runtime)
    ))
}

/// Replace device statuses with verdicts; returns indexes of changed rows.
pub(super) fn enrich(home: &Path, items: &mut [Item], facts: &HashMap<String, Sim>) -> Vec<usize> {
    if !items.iter().any(|item| item.kind == Kind::Device) {
        return Vec::new();
    }
    let projects = xcode::Projects::load(home, &xcode::recorded_workspaces(home));
    let now = verdict::now();
    let mut changed = Vec::new();
    for (index, item) in items.iter_mut().enumerate() {
        let Some(sim) = facts.get(&item.id).filter(|_| item.kind == Kind::Device) else {
            continue;
        };
        let apps: Vec<AppEvidence> = sim
            .data
            .as_deref()
            .map(installed)
            .unwrap_or_default()
            .into_iter()
            .map(|bundle| AppEvidence {
                project: projects.app(&bundle),
                bundle,
            })
            .collect();
        item.status = status(sim, &apps, now);
        changed.push(index);
    }
    changed
}
