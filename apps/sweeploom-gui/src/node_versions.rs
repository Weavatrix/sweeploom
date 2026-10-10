//! Query running Node binaries once per installation, outside the UI thread.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crossbeam_channel::Receiver;
use sweeploom_core::ProcessSnapshot;

#[derive(Default)]
pub(crate) struct NodeVersions {
    versions: HashMap<PathBuf, String>,
    attempted: HashSet<PathBuf>,
    worker: Option<Receiver<Vec<(PathBuf, Option<String>)>>>,
}

impl NodeVersions {
    pub(crate) fn poll(&mut self, processes: &[ProcessSnapshot], ctx: &eframe::egui::Context) {
        if let Some(rx) = &self.worker {
            if let Ok(values) = rx.try_recv() {
                for (path, version) in values {
                    if let Some(version) = version {
                        self.versions.insert(path, version);
                    }
                }
                self.worker = None;
            } else {
                return;
            }
        }
        let mut paths = Vec::new();
        for process in processes.iter().filter(|p| sweeploom_session::is_node(p)) {
            let Some(path) = &process.exe else {
                continue;
            };
            if !path.is_absolute()
                || !path
                    .file_name()
                    .is_some_and(|name| name == "node" || name == "node.exe")
            {
                continue;
            }
            if self.attempted.insert(path.clone()) {
                paths.push(path.clone());
            }
            if paths.len() >= 8 {
                break;
            }
        }
        if paths.is_empty() {
            return;
        }
        let (tx, rx) = crossbeam_channel::bounded(1);
        self.worker = Some(rx);
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let values = paths
                .into_iter()
                .map(|path| {
                    let version = query_version(&path);
                    (path, version)
                })
                .collect();
            let _ = tx.send(values);
            ctx.request_repaint();
        });
    }

    pub(crate) fn get(&self, path: &Path) -> Option<&str> {
        self.versions.get(path).map(String::as_str)
    }
}

fn query_version(path: &Path) -> Option<String> {
    let mut child = Command::new(path)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return None;
                }
                let mut output = String::new();
                use std::io::Read;
                child
                    .stdout
                    .take()?
                    .take(128)
                    .read_to_string(&mut output)
                    .ok()?;
                return parse_version(&output);
            }
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
}

fn parse_version(output: &str) -> Option<String> {
    let version = output.trim();
    let numbers: Vec<_> = version.strip_prefix('v')?.split('.').collect();
    (numbers.len() == 3
        && numbers
            .iter()
            .all(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit())))
    .then(|| version.to_owned())
}

#[cfg(test)]
mod tests {
    #[test]
    fn only_actual_node_version_output_is_accepted() {
        assert_eq!(super::parse_version("v22.13.1\n"), Some("v22.13.1".into()));
        assert_eq!(super::parse_version("2026.09.26-dd393fe"), None);
        assert_eq!(super::parse_version("node server.js started"), None);
    }
}
