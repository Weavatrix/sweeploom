//! Local scan archive. Only finished measurements become historical samples.

use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use crossbeam_channel::Receiver;
use serde::{Deserialize, Serialize};
use sweeploom_storage::{DirectoryNode, DiskUsage, InventoryReport};

const VERSION: u32 = 1;
const MAX_SCANS: usize = 12;
const MAX_POINTS: usize = 32;
const MAX_PATHS: usize = 25_000;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Source {
    Explorer,
    Projects,
    Review,
    Ai,
    Native,
}

impl Source {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Explorer => "Explorer",
            Self::Projects => "Projects",
            Self::Review => "Review estimates",
            Self::Ai => "AI estimates",
            Self::Native => "Native cleanup",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Basis {
    Allocated,
    Logical,
    Native,
}
impl Basis {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Allocated => "Size on disk",
            Self::Logical => "Logical estimate",
            Self::Native => "Tool reported",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Point {
    pub at: u64,
    pub usage: DiskUsage,
    pub basis: Basis,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Series {
    pub source: Source,
    pub path: PathBuf,
    pub points: Vec<Point>,
}

impl Series {
    pub(crate) fn latest(&self) -> &Point {
        self.points.last().expect("nonempty series")
    }

    pub(crate) fn delta(&self) -> Option<(i128, &Point)> {
        let latest = self.latest();
        if !latest.usage.complete {
            return None;
        }
        let previous = self.points[..self.points.len() - 1]
            .iter()
            .rev()
            .find(|point| point.usage.complete && point.basis == latest.basis)?;
        Some((
            i128::from(latest.usage.bytes) - i128::from(previous.usage.bytes),
            previous,
        ))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct SavedScan {
    pub at: u64,
    pub report: InventoryReport,
}

#[derive(Clone, Serialize, Deserialize)]
struct Archive {
    version: u32,
    scans: Vec<SavedScan>,
    series: Vec<Series>,
    projects: Vec<PathBuf>,
}

impl Default for Archive {
    fn default() -> Self {
        Self {
            version: VERSION,
            scans: Vec::new(),
            series: Vec::new(),
            projects: Vec::new(),
        }
    }
}

pub(crate) struct ScanHistory {
    archive: Archive,
    lookup: HashMap<(Source, PathBuf), usize>,
    path: PathBuf,
    dirty: bool,
    writer: Option<Receiver<Result<(), String>>>,
    last_write: Instant,
    writable: bool,
    pub error: Option<String>,
}

impl ScanHistory {
    pub(crate) fn load(path: PathBuf) -> Self {
        let loaded = (|| -> Result<Archive, String> {
            let file = match File::open(&path) {
                Ok(file) => file,
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    return Ok(Archive::default());
                }
                Err(error) => return Err(error.to_string()),
            };
            let archive: Archive =
                serde_json::from_reader(BufReader::new(file)).map_err(|e| e.to_string())?;
            if archive.version != VERSION {
                return Err("Unsupported scan archive version; existing file was preserved".into());
            }
            if archive.series.iter().any(|series| series.points.is_empty()) {
                return Err(
                    "Invalid empty measurement history; existing file was preserved".into(),
                );
            }
            Ok(archive)
        })();
        let (archive, error) = match loaded {
            Ok(archive) => (archive, None),
            Err(error) => (Archive::default(), Some(error)),
        };
        let writable = error.is_none();
        let mut store = Self {
            archive,
            lookup: HashMap::new(),
            path,
            dirty: false,
            writer: None,
            last_write: Instant::now(),
            writable,
            error,
        };
        store.reindex();
        store
    }

    fn reindex(&mut self) {
        self.lookup = self
            .archive
            .series
            .iter()
            .enumerate()
            .map(|(index, series)| ((series.source, series.path.clone()), index))
            .collect();
    }

    pub(crate) fn scans(&self) -> &[SavedScan] {
        &self.archive.scans
    }
    pub(crate) fn series(&self) -> &[Series] {
        &self.archive.series
    }
    pub(crate) fn projects(&self) -> &[PathBuf] {
        &self.archive.projects
    }
    pub(crate) fn get(&self, source: Source, path: &Path) -> Option<&Series> {
        self.lookup
            .get(&(source, path.to_path_buf()))
            .map(|&index| &self.archive.series[index])
    }

    pub(crate) fn remember_projects(&mut self, projects: &[PathBuf]) {
        if self.archive.projects != projects {
            self.archive.projects = projects.to_vec();
            self.dirty = true;
        }
    }

    pub(crate) fn record_batch(
        &mut self,
        source: Source,
        values: Vec<(PathBuf, DiskUsage, Basis)>,
        at: u64,
    ) {
        // Alternative cleanup offers can share a path. They are one observation,
        // not multiple historical scans. Preserve the largest size for that path.
        let mut unique = HashMap::<PathBuf, (DiskUsage, Basis)>::new();
        for (path, usage, basis) in values {
            unique
                .entry(path)
                .and_modify(|value| {
                    if usage.bytes > value.0.bytes {
                        *value = (usage, basis);
                    }
                })
                .or_insert((usage, basis));
        }
        for (path, (usage, basis)) in unique {
            let index = if let Some(&index) = self.lookup.get(&(source, path.clone())) {
                index
            } else {
                let index = self.archive.series.len();
                self.archive.series.push(Series {
                    source,
                    path: path.clone(),
                    points: Vec::new(),
                });
                self.lookup.insert((source, path), index);
                index
            };
            let points = &mut self.archive.series[index].points;
            if points.last().is_some_and(|point| point.at == at) {
                points.pop();
            }
            points.push(Point { at, usage, basis });
            points.sort_by_key(|point| point.at);
            if points.len() > MAX_POINTS {
                points.remove(0);
            }
        }
        if self.archive.series.len() > MAX_PATHS {
            self.archive
                .series
                .sort_by_key(|series| std::cmp::Reverse(series.latest().at));
            self.archive.series.truncate(MAX_PATHS);
            self.reindex();
        }
        self.dirty = true;
    }

    pub(crate) fn record_scan(&mut self, report: &InventoryReport, at: u64) {
        let mut values = Vec::new();
        fn collect(
            node: &DirectoryNode,
            report_complete: bool,
            values: &mut Vec<(PathBuf, DiskUsage, Basis)>,
        ) {
            values.push((
                node.path.clone(),
                DiskUsage {
                    bytes: node.disk_bytes(),
                    logical_bytes: node.logical_bytes,
                    files: node.files,
                    errors: u64::from(!report_complete || node.incomplete),
                    complete: report_complete && !node.incomplete,
                },
                if node.allocated_bytes.is_some() {
                    Basis::Allocated
                } else {
                    Basis::Logical
                },
            ));
            for child in &node.children {
                collect(child, report_complete, values);
            }
        }
        collect(
            &report.tree,
            !report.capped && report.errors == 0,
            &mut values,
        );
        self.record_batch(Source::Explorer, values, at);
        self.archive.scans.push(SavedScan {
            at,
            report: report.clone(),
        });
        if self.archive.scans.len() > MAX_SCANS {
            self.archive.scans.remove(0);
        }
        self.dirty = true;
    }

    pub(crate) fn record_review(&mut self, rows: &[sweeploom_dev::ReviewRow]) {
        let values = rows
            .iter()
            .map(|row| {
                let candidate = &row.candidate;
                let complete = !row.title.contains('≥')
                    && !row.title.contains(">=")
                    && !candidate
                        .evidence
                        .iter()
                        .any(|e| e.key.ends_with("capped") || e.key == "size-incomplete");
                candidate_value(candidate, complete)
            })
            .collect();
        self.record_batch(Source::Review, values, now_ms());
    }

    pub(crate) fn poll_save(&mut self) {
        if let Some(rx) = &self.writer {
            match rx.try_recv() {
                Ok(result) => {
                    self.writer = None;
                    if let Err(error) = result {
                        self.error = Some(error);
                        self.dirty = true;
                    } else {
                        self.error = None;
                    }
                }
                Err(crossbeam_channel::TryRecvError::Empty) => return,
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    self.writer = None;
                    self.error = Some("Scan archive writer stopped unexpectedly".into());
                    self.dirty = true;
                }
            }
        }
        if !self.writable || !self.dirty || self.last_write.elapsed() < Duration::from_secs(2) {
            return;
        }
        let archive = self.archive.clone();
        let path = self.path.clone();
        let (tx, rx) = crossbeam_channel::bounded(1);
        self.writer = Some(rx);
        self.dirty = false;
        self.last_write = Instant::now();
        std::thread::spawn(move || {
            let _ = tx.send(write_atomic(&path, &archive).map_err(|e| e.to_string()));
        });
    }

    // Complete outstanding writes on normal quit, including a scan finished in
    // the last two seconds. The old archive remains intact if a write fails.
    pub(crate) fn flush(&mut self) {
        if let Some(rx) = self.writer.take() {
            match rx.recv() {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    self.error = Some(error);
                    self.dirty = true;
                }
                Err(error) => {
                    self.error = Some(error.to_string());
                    self.dirty = true;
                }
            }
        }
        if self.writable && self.dirty {
            match write_atomic(&self.path, &self.archive) {
                Ok(()) => {
                    self.dirty = false;
                    self.error = None;
                }
                Err(error) => self.error = Some(error.to_string()),
            }
        }
    }
}

fn write_atomic(path: &Path, archive: &Archive) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("json.tmp");
    let mut writer = BufWriter::new(File::create(&temp)?);
    serde_json::to_writer(&mut writer, archive).map_err(io::Error::other)?;
    writer.flush()?;
    writer.get_ref().sync_all()?;
    drop(writer);
    fs::rename(temp, path)
}

pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

pub(crate) fn signed_bytes(delta: i128) -> String {
    if delta == 0 {
        return "Unchanged".into();
    }
    let bytes = delta.unsigned_abs().min(u128::from(u64::MAX)) as u64;
    let size = if bytes < 1000 {
        format!("{bytes} B")
    } else {
        crate::format::format_bytes(bytes)
    };
    format!("{}{size}", if delta > 0 { "+" } else { "-" })
}

pub(crate) fn timestamp(ms: u64) -> String {
    // Gregorian calendar conversion, UTC. No locale or time-zone ambiguity in
    // comparisons after travel or a daylight-saving change.
    let seconds = ms / 1000;
    let days = (seconds / 86400) as i64 + 719468;
    let era = days.div_euclid(146097);
    let doe = days - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02} UTC",
        seconds / 3600 % 24,
        seconds / 60 % 60,
        seconds % 60
    )
}

/// Consistent visible trend in every disk table; full dates in the tooltip.
pub(crate) fn trend(ui: &mut eframe::egui::Ui, history: &ScanHistory, source: Source, path: &Path) {
    trend_at(ui, history, source, path, None);
}

pub(crate) fn trend_at(
    ui: &mut eframe::egui::Ui,
    history: &ScanHistory,
    source: Source,
    path: &Path,
    at: Option<u64>,
) {
    let Some(series) = history.get(source, path) else {
        ui.label(
            eframe::egui::RichText::new("No history")
                .small()
                .color(crate::theme::muted(ui)),
        );
        return;
    };
    let count = at.map_or(series.points.len(), |at| {
        series.points.partition_point(|point| point.at <= at)
    });
    if count == 0 {
        ui.label("No history");
        return;
    }
    let latest = &series.points[count - 1];
    let previous = series.points[..count - 1]
        .iter()
        .rev()
        .find(|point| point.usage.complete && point.basis == latest.basis);
    let delta = latest
        .usage
        .complete
        .then_some(previous)
        .flatten()
        .map(|before| {
            (
                i128::from(latest.usage.bytes) - i128::from(before.usage.bytes),
                before,
            )
        });
    let (caption, note) = match delta {
        Some((delta, before)) => {
            let percent = if before.usage.bytes > 0 {
                format!(
                    " ({:+.1}%)",
                    delta as f64 / before.usage.bytes as f64 * 100.0
                )
            } else {
                String::new()
            };
            (
                signed_bytes(delta),
                format!(
                    "{}{} since {}\nPrevious: {} bytes\nLatest: {} bytes at {}\nExact change: {delta:+} bytes\n{} saved measurements\n{}",
                    signed_bytes(delta),
                    percent,
                    timestamp(before.at),
                    before.usage.bytes,
                    latest.usage.bytes,
                    timestamp(latest.at),
                    count,
                    source.label()
                ),
            )
        }
        None if !latest.usage.complete => (
            "Partial".into(),
            format!(
                "Incomplete scan at {}. Growth/shrinkage cannot be determined.",
                timestamp(latest.at)
            ),
        ),
        None => (
            "First scan".into(),
            format!(
                "Measured at {}. Scan again to compare.\n{}",
                timestamp(latest.at),
                source.label()
            ),
        ),
    };
    let color = match delta.map(|(value, _)| value) {
        Some(value) if value > 0 => crate::theme::accent(),
        Some(value) if value < 0 => crate::theme::ok(),
        _ => crate::theme::muted(ui),
    };
    ui.label(eframe::egui::RichText::new(caption).small().color(color))
        .on_hover_text(note);
}

pub(crate) fn candidate_value(
    candidate: &sweeploom_core::Candidate,
    complete: bool,
) -> (PathBuf, DiskUsage, Basis) {
    (
        candidate.path.clone(),
        DiskUsage {
            bytes: candidate.estimated_reclaimable_bytes(),
            logical_bytes: candidate.logical_bytes,
            files: candidate.file_count,
            errors: 0,
            complete,
        },
        if candidate.allocated_bytes.is_some() {
            Basis::Allocated
        } else {
            Basis::Logical
        },
    )
}

#[cfg(test)]
#[path = "scan_history_tests.rs"]
mod tests;
