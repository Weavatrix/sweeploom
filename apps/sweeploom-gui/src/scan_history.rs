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
            Self::Review => "Project output",
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
}

#[path = "scan_history_record.rs"]
mod record;
#[path = "scan_history_view.rs"]
mod view;
pub(crate) use view::{candidate_value, now_ms, signed_bytes, timestamp, trend, trend_at};

#[cfg(test)]
#[path = "scan_history_tests.rs"]
mod tests;
