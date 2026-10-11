//! Full project sizes are measured separately from Explorer and cleanup offers.
//!
//! A refresh never blanks the table: the last measurement stays visible
//! (marked saved) until a fresh one replaces it.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crossbeam_channel::{Receiver, TryRecvError};
use sweeploom_dev::{DevKind, classify_project, read_manifest, workspace_root};
use sweeploom_storage::{DiskUsage, directory_disk_usage};

type Measured = (PathBuf, Result<DiskUsage, String>);

/// Saved sizes younger than this are not re-walked on launch; Refresh still does.
const RECENT_MS: u64 = 12 * 60 * 60 * 1000;

/// Marker facts for one project row, read once per Review rebuild.
#[derive(Clone)]
pub(crate) struct ProjectFacts {
    pub kinds: Vec<DevKind>,
    /// Cargo unit label; workspace members note the shared target.
    pub cargo: Option<String>,
}

impl ProjectFacts {
    fn read(path: &Path) -> Self {
        let cargo = read_manifest(path).map(|manifest| {
            let units = manifest.units_label();
            if workspace_root(path) != path {
                format!("{units} · shared target")
            } else {
                units
            }
        });
        Self {
            kinds: classify_project(path),
            cargo,
        }
    }
}

#[derive(Default)]
pub(crate) struct ProjectSizes {
    values: HashMap<PathBuf, Result<DiskUsage, String>>,
    /// Measured since the last refresh. Everything else is a saved value.
    fresh: HashSet<PathBuf>,
    /// Saved values recent enough to skip until the next explicit refresh.
    recent: HashSet<PathBuf>,
    worker: Option<Receiver<Measured>>,
    discard_worker: bool,
    revision: u64,
    facts: HashMap<PathBuf, ProjectFacts>,
}

impl ProjectSizes {
    pub(crate) fn restore(history: &crate::scan_history::ScanHistory) -> Self {
        let mut sizes = Self::default();
        let now = crate::scan_history::now_ms();
        for series in history
            .series()
            .iter()
            .filter(|series| series.source == crate::scan_history::Source::Projects)
        {
            let latest = series.latest();
            if now.saturating_sub(latest.at) < RECENT_MS {
                sizes.recent.insert(series.path.clone());
            }
            sizes.values.insert(series.path.clone(), Ok(latest.usage));
        }
        sizes
    }

    pub(crate) fn is_cached(&self, path: &Path) -> bool {
        self.values.contains_key(path) && !self.fresh.contains(path)
    }

    pub(crate) fn get(&self, path: &Path) -> Option<&Result<DiskUsage, String>> {
        self.values.get(path)
    }

    pub(crate) fn revision(&self) -> u64 {
        self.revision
    }

    /// Re-measure everything in the background, keeping current values on screen.
    pub(crate) fn invalidate(&mut self) {
        self.fresh.clear();
        self.recent.clear();
        self.discard_worker = self.worker.is_some();
        self.revision = self.revision.wrapping_add(1);
    }

    pub(crate) fn facts(&mut self, path: &Path) -> ProjectFacts {
        self.facts
            .entry(path.to_path_buf())
            .or_insert_with(|| ProjectFacts::read(path))
            .clone()
    }

    /// Manifests may have changed; read kinds and Cargo units again on demand.
    pub(crate) fn forget_facts(&mut self) {
        self.facts.clear();
        self.revision = self.revision.wrapping_add(1);
    }

    pub(crate) fn poll(
        &mut self,
        paths: &[PathBuf],
        ctx: &eframe::egui::Context,
        history: &mut crate::scan_history::ScanHistory,
    ) {
        if let Some(rx) = &self.worker {
            loop {
                match rx.try_recv() {
                    Ok((path, usage)) if !self.discard_worker => {
                        if let Ok(usage) = &usage {
                            history.record_batch(
                                crate::scan_history::Source::Projects,
                                vec![(path.clone(), *usage, basis())],
                                crate::scan_history::now_ms(),
                            );
                        }
                        self.fresh.insert(path.clone());
                        self.values.insert(path, usage);
                        self.revision = self.revision.wrapping_add(1);
                    }
                    Ok(_) => {}
                    Err(TryRecvError::Empty) => return,
                    Err(TryRecvError::Disconnected) => break,
                }
            }
            self.worker = None;
            self.discard_worker = false;
        }
        let paths: Vec<_> = paths
            .iter()
            .filter(|path| !self.fresh.contains(*path) && !self.recent.contains(*path))
            .take(8)
            .cloned()
            .collect();
        if paths.is_empty() {
            return;
        }
        let (tx, rx) = crossbeam_channel::unbounded();
        self.worker = Some(rx);
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            for path in paths {
                let usage = std::panic::catch_unwind(|| directory_disk_usage(&path))
                    .map_err(|_| "Directory measurement failed".to_owned())
                    .and_then(|result| result.map_err(|error| error.to_string()));
                if tx.send((path, usage)).is_err() {
                    break;
                }
                ctx.request_repaint();
            }
        });
    }
}

fn basis() -> crate::scan_history::Basis {
    if cfg!(unix) {
        crate::scan_history::Basis::Allocated
    } else {
        crate::scan_history::Basis::Logical
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn history(name: &str) -> crate::scan_history::ScanHistory {
        crate::scan_history::ScanHistory::load(std::env::temp_dir().join(format!(
            "sweeploom-sizes-{name}-{}.json",
            std::process::id()
        )))
    }

    #[test]
    fn refresh_discards_in_flight_measurements_before_cleanup() {
        let path = PathBuf::from("/project");
        let (tx, rx) = crossbeam_channel::unbounded();
        let mut sizes = ProjectSizes {
            worker: Some(rx),
            ..Default::default()
        };
        let usage = DiskUsage {
            bytes: 9000,
            ..Default::default()
        };
        tx.send((path.clone(), Ok(usage))).unwrap();
        drop(tx);
        sizes.invalidate();
        let mut history = history("inflight");
        sizes.poll(&[], &eframe::egui::Context::default(), &mut history);
        assert!(sizes.get(&path).is_none());
        assert!(sizes.worker.is_none());
        assert!(!sizes.discard_worker);
    }

    #[test]
    fn refresh_keeps_measured_sizes_visible_until_remeasured() {
        let path = PathBuf::from("/kept-project");
        let (tx, rx) = crossbeam_channel::unbounded();
        let mut sizes = ProjectSizes {
            worker: Some(rx),
            ..Default::default()
        };
        let usage = DiskUsage {
            bytes: 4096,
            ..Default::default()
        };
        tx.send((path.clone(), Ok(usage))).unwrap();
        drop(tx);
        let mut history = history("kept");
        let ctx = eframe::egui::Context::default();
        sizes.poll(&[], &ctx, &mut history);
        assert!(!sizes.is_cached(&path));
        sizes.invalidate();
        assert_eq!(sizes.get(&path).unwrap().as_ref().unwrap().bytes, 4096);
        assert!(sizes.is_cached(&path), "shown as saved while re-measuring");
    }
}
