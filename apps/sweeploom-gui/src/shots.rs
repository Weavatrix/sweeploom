//! Debug-only screen tour: `SWEEPLOOM_SHOTS=<dir>` saves every screen as PNG, then quits.
//! Optional `SWEEPLOOM_SHOTS_ONLY=sessions,cleanup`, `SWEEPLOOM_SHOTS_PREFIX=final`,
//! `SWEEPLOOM_SHOTS_THEMES=dark` and `SWEEPLOOM_SHOTS_FPS=1` (log frame times).

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use eframe::egui::{self, ViewportCommand};

use crate::app::SweepLoomApp;
use crate::nav::Nav;
use crate::prefs::ThemeMode;

const PANE_KEY: &str = "sweeploom-shot-pane";
/// Pane marker that scrolls the Sessions detail instead of picking a Cleanup pane.
const SCROLL: usize = 99;

struct Shot {
    name: String,
    nav: Nav,
    pane: Option<usize>,
    theme: ThemeMode,
}

struct Tour {
    dir: PathBuf,
    shots: Vec<Shot>,
    index: usize,
    since: Instant,
    asked: bool,
    theme: ThemeMode,
}

static TOUR: Mutex<Option<Option<Tour>>> = Mutex::new(None);

const SCREENS: [(&str, Nav, Option<usize>); 17] = [
    ("overview", Nav::Overview, None),
    ("sessions", Nav::Sessions, None),
    ("sessions-activity", Nav::Sessions, Some(SCROLL)),
    ("history", Nav::History, None),
    ("review", Nav::Storage, None),
    ("explorer", Nav::Explorer, None),
    ("projects", Nav::Projects, None),
    ("cleanup-docker", Nav::Cleanup, Some(0)),
    ("cleanup-ios", Nav::Cleanup, Some(1)),
    ("cleanup-build", Nav::Cleanup, Some(2)),
    ("cleanup-apps", Nav::Cleanup, Some(3)),
    ("cleanup-models", Nav::Cleanup, Some(4)),
    ("cleanup-data", Nav::Cleanup, Some(5)),
    ("scan-history", Nav::DiskHistory, None),
    ("browser", Nav::Browser, None),
    ("ai", Nav::Ai, None),
    ("settings", Nav::Settings, None),
];

fn start(app: &SweepLoomApp) -> Option<Tour> {
    let dir = PathBuf::from(std::env::var_os("SWEEPLOOM_SHOTS")?);
    let _ = std::fs::create_dir_all(&dir);
    let only = std::env::var("SWEEPLOOM_SHOTS_ONLY").unwrap_or_default();
    let prefix = std::env::var("SWEEPLOOM_SHOTS_PREFIX").unwrap_or_else(|_| "shot".into());
    let themes = std::env::var("SWEEPLOOM_SHOTS_THEMES").unwrap_or_else(|_| "dark,light".into());
    let mut shots = Vec::new();
    for (theme, label) in [(ThemeMode::Dark, "dark"), (ThemeMode::Light, "light")] {
        if !themes.contains(label) {
            continue;
        }
        for (name, nav, pane) in SCREENS {
            if only.is_empty() || only.split(',').any(|part| name.contains(part.trim())) {
                shots.push(Shot {
                    name: format!("{prefix}-{label}-{name}"),
                    nav,
                    pane,
                    theme,
                });
            }
        }
    }
    Some(Tour {
        dir,
        shots,
        index: 0,
        since: Instant::now() + Duration::from_secs(3),
        asked: false,
        theme: app.prefs.theme,
    })
}

/// Cleanup pane forced by the tour, if any.
pub fn pane(ctx: &egui::Context) -> Option<usize> {
    ctx.data(|data| data.get_temp::<usize>(egui::Id::new(PANE_KEY)))
        .filter(|pane| *pane != SCROLL)
}

/// Detail scroll offset forced by the tour, if any.
pub fn scroll(ctx: &egui::Context) -> Option<f32> {
    ctx.data(|data| data.get_temp::<usize>(egui::Id::new(PANE_KEY)))
        .filter(|pane| *pane == SCROLL)
        .map(|_| 430.0)
}

/// Advance the tour by one frame. No-op unless `SWEEPLOOM_SHOTS` is set.
pub fn drive(ctx: &egui::Context, app: &mut SweepLoomApp) {
    let mut guard = TOUR
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let slot = guard.get_or_insert_with(|| {
        let tour = start(app);
        if tour.is_some() {
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(egui::vec2(1360.0, 860.0)));
        }
        tour
    });
    let Some(tour) = slot else {
        return;
    };
    if std::env::var_os("SWEEPLOOM_SHOTS_FPS").is_some() {
        static LAST: Mutex<Option<Instant>> = Mutex::new(None);
        let mut last = LAST.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(at) = last.replace(Instant::now()) {
            eprintln!("frame {:?}", at.elapsed());
        }
    }
    ctx.request_repaint_after(Duration::from_millis(60));
    let Some(shot) = tour.shots.get(tour.index) else {
        app.prefs.theme = tour.theme;
        app.force_quit = true;
        ctx.send_viewport_cmd(ViewportCommand::Close);
        return;
    };
    app.nav = shot.nav;
    app.prefs.theme = shot.theme;
    ctx.data_mut(|data| match shot.pane {
        Some(pane) => data.insert_temp(egui::Id::new(PANE_KEY), pane),
        None => data.remove::<usize>(egui::Id::new(PANE_KEY)),
    });
    if shot.nav == Nav::Sessions && app.selected_session.is_none() {
        app.selected_session = app
            .sessions
            .iter()
            .filter(|session| session.kind.is_known_dev())
            .max_by_key(|session| session.rss_bytes)
            .map(|session| session.id);
    }
    let settle = match (shot.nav, shot.pane) {
        (Nav::Ai, _) => 60,
        (Nav::Storage | Nav::Projects, _) => 8,
        (_, Some(_)) => 5,
        _ => 3,
    };
    if !tour.asked && tour.since.elapsed() >= Duration::from_secs(settle) {
        ctx.send_viewport_cmd(ViewportCommand::Screenshot(egui::UserData::new(tour.index)));
        tour.asked = true;
    }
    let image = ctx.input(|input| {
        input.raw.events.iter().find_map(|event| match event {
            egui::Event::Screenshot { image, .. } => Some(image.clone()),
            _ => None,
        })
    });
    if let Some(image) = image.filter(|_| tour.asked) {
        save(&tour.dir.join(format!("{}.png", shot.name)), &image);
        tour.index += 1;
        tour.asked = false;
        tour.since = Instant::now();
    }
}

fn save(path: &std::path::Path, image: &egui::ColorImage) {
    let bytes: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
    if let Some(buffer) =
        image::RgbaImage::from_raw(image.size[0] as u32, image.size[1] as u32, bytes)
    {
        let _ = buffer.save(path);
    }
}
