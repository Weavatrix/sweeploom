//! simctl confirmation: device sets and runtimes are queried side by side, then merged.
use super::{
    HashMap, HashSet, Item, Kind, LIST_TIMEOUT, Path, Sim, Verdict, disk, item, list, runtimes,
    simctl, sims,
};

const STOPPED: &str = "simctl worker stopped";

/// All simctl calls run side by side: a busy CoreSimulator costs one timeout, not several.
pub(super) fn confirm(
    home: &Path,
    items: &mut Vec<Item>,
    facts: &mut HashMap<String, Sim>,
) -> Vec<String> {
    let sets: Vec<disk::Set> = disk::sets(home)
        .into_iter()
        .filter(|set| {
            set.scope.is_none()
                || items
                    .iter()
                    .any(|item| item.kind == Kind::Device && item.scope == set.scope)
        })
        .collect();
    let (lists, runtime) = std::thread::scope(|scope| {
        let runtime = scope.spawn(|| simctl(None, &["runtime", "list", "--json"], LIST_TIMEOUT));
        let lists: Vec<_> = sets
            .iter()
            .map(|set| scope.spawn(move || list(set.scope.as_deref())))
            .collect();
        (
            lists
                .into_iter()
                .map(|handle| handle.join().unwrap_or_else(|_| Err(STOPPED.to_owned())))
                .collect::<Vec<_>>(),
            runtime.join().unwrap_or_else(|_| Err(STOPPED.to_owned())),
        )
    });
    let installed = runtime.as_deref().ok().and_then(runtimes::identifiers);
    let mut notes = Vec::new();
    for (set, result) in sets.iter().zip(lists) {
        match result {
            Ok(value) => merge(items, facts, set, sims(&value), installed.as_ref()),
            Err(error) => notes.push(format!(
                "{} devices: {error}. Showing folders from disk; removal needs simctl.",
                set.label
            )),
        }
    }
    if let Err(error) = runtime.and_then(|text| runtimes::add(&text, items, facts)) {
        notes.push(format!("Runtime inventory: {error}"));
    }
    notes
}

/// simctl is authoritative for its set: registered folders it no longer lists become leftovers.
pub(super) fn merge(
    items: &mut Vec<Item>,
    facts: &mut HashMap<String, Sim>,
    set: &disk::Set,
    mut listed: Vec<Sim>,
    installed: Option<&HashSet<String>>,
) {
    settle(&mut listed, installed);
    let ids: HashSet<String> = listed.iter().map(|sim| sim.udid.clone()).collect();
    for row in items.iter_mut() {
        if row.kind == Kind::Device && row.scope == set.scope && !ids.contains(&row.id) {
            facts.remove(&row.id);
            if let Some(dir) = row.path.as_deref().and_then(Path::parent) {
                *row = disk::orphan(set, &row.id.clone(), dir.to_owned());
            }
        }
    }
    let stranded = stranded(&listed);
    if !stranded.is_empty() {
        let count = stranded.len();
        items.push(Item {
            id: format!("unavailable:{}:{count}", set.label.replace(' ', "-")),
            name: format!("Delete {count} unavailable {} devices", set.label),
            kind: Kind::SimUnavailable,
            bytes: Some(stranded.iter().filter_map(|sim| sim.bytes).sum()),
            usage: None,
            status: Verdict::Suggested.status(&format!(
                "Their runtime is no longer installed; deletes exactly these {count} shut-down devices"
            )),
            scope: set.scope.clone(),
            path: None,
            selected: false,
            enabled: true,
        });
    }
    for sim in listed {
        let row = item(&sim, set);
        match items.iter_mut().find(|old| old.id == row.id) {
            Some(old) => *old = row,
            None => items.push(row),
        }
        facts.insert(sim.udid.clone(), sim);
    }
}

/// A runtime that is installed but not loaded makes devices look unavailable for a while;
/// without a runtime inventory every unavailable device is treated that way.
pub(super) fn settle(listed: &mut [Sim], installed: Option<&HashSet<String>>) {
    for sim in listed.iter_mut().filter(|sim| sim.unavailable.is_some()) {
        if installed.is_none_or(|ids| ids.contains(&sim.runtime)) {
            sim.unavailable = None;
            sim.loading = true;
        }
    }
}

/// Shut-down devices whose runtime is really gone: the only bulk-deletion targets.
pub(super) fn stranded(listed: &[Sim]) -> Vec<&Sim> {
    listed
        .iter()
        .filter(|sim| sim.unavailable.is_some() && sim.state == "Shutdown")
        .collect()
}
