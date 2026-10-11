//! Git dirty check for session projects, off the UI thread and cached.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use eframe::egui;

const FRESH: Duration = Duration::from_secs(60);

#[derive(Clone, Copy)]
struct Entry {
    at: Instant,
    blocked: Option<bool>,
    running: bool,
}

static CACHE: Mutex<Option<HashMap<PathBuf, Entry>>> = Mutex::new(None);

/// Last known "has Git changes" for `project`. `None` until the first check lands.
/// A stale or missing entry starts one background inspect; the frame never blocks.
pub fn blocked(ctx: &egui::Context, project: &Path) -> Option<bool> {
    let mut guard = CACHE.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let cache = guard.get_or_insert_with(HashMap::new);
    let entry = cache.get(project).copied();
    let stale = entry.is_none_or(|e| !e.running && e.at.elapsed() >= FRESH);
    if stale {
        cache.insert(
            project.to_owned(),
            Entry {
                at: Instant::now(),
                blocked: entry.and_then(|e| e.blocked),
                running: true,
            },
        );
        let path = project.to_owned();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let blocked = sweeploom_dev::inspect(&path).assessment().is_blocked();
            let mut guard = CACHE.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            guard.get_or_insert_with(HashMap::new).insert(
                path,
                Entry {
                    at: Instant::now(),
                    blocked: Some(blocked),
                    running: false,
                },
            );
            ctx.request_repaint();
        });
    }
    entry.and_then(|e| e.blocked)
}
