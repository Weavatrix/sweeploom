//! Recording finished measurements and saving the archive off the UI thread.

use super::*;

impl ScanHistory {
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
