//! Application state and screens.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, RichText};
use egui_extras::{Column, TableBuilder};

use sweeploom_core::{LiveSession, ProcessSnapshot, Recommendation, SessionKind};
use sweeploom_network::enrich_network;
use sweeploom_platform::UserLocations;
use sweeploom_process::{HostMemory, ProcessSampler, ProcessSnapshotSet};
use sweeploom_session::{AttributionRoots, sessions_from_snapshot};
use sweeploom_storage::{DirectoryNode, InventoryLimits, InventoryReport, scan_inventory};

/// Primary navigation. Sessions are first-class, not buried in Settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Nav {
    Overview,
    Storage,
    Sessions,
    Projects,
    Browser,
    Explorer,
    Ai,
    Rules,
    History,
    Settings,
}

impl Nav {
    const ALL: [Self; 10] = [
        Self::Overview,
        Self::Storage,
        Self::Sessions,
        Self::Projects,
        Self::Browser,
        Self::Explorer,
        Self::Ai,
        Self::Rules,
        Self::History,
        Self::Settings,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Storage => "Storage",
            Self::Sessions => "Sessions",
            Self::Projects => "Projects",
            Self::Browser => "Browser",
            Self::Explorer => "Explorer",
            Self::Ai => "AI",
            Self::Rules => "Rules",
            Self::History => "History",
            Self::Settings => "Settings",
        }
    }
}

/// Live UI application.
pub struct SweepLoomApp {
    nav: Nav,
    sampler: ProcessSampler,
    last_sample: Instant,
    snapshot: Option<ProcessSnapshotSet>,
    sessions: Vec<LiveSession>,
    selected_session: Option<usize>,
    group_raw: bool,
    inventory: Option<InventoryReport>,
    inventory_error: Option<String>,
    scan_root: String,
    locations: UserLocations,
}

impl SweepLoomApp {
    /// Construct and take the first process sample.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        cc.egui_ctx.set_style(style);

        let locations = UserLocations::current();
        let mut sampler = ProcessSampler::new();
        let mut snapshot = sampler.refresh(Duration::from_millis(200));
        snapshot.resolve_parents();
        let _ = enrich_network(&mut snapshot.processes);
        let sessions = sessions_from_snapshot(
            &mut snapshot,
            &AttributionRoots {
                projects: vec![locations.home.clone()],
                current_project: None,
            },
        );
        Self {
            nav: Nav::Overview,
            sampler,
            last_sample: Instant::now(),
            snapshot: Some(snapshot),
            sessions,
            selected_session: None,
            group_raw: false,
            inventory: None,
            inventory_error: None,
            scan_root: locations.home.display().to_string(),
            locations,
        }
    }

    fn refresh_live(&mut self) {
        if self.last_sample.elapsed() < Duration::from_secs(1) {
            return;
        }
        let mut snapshot = self.sampler.refresh(Duration::ZERO);
        snapshot.resolve_parents();
        let _ = enrich_network(&mut snapshot.processes);
        self.sessions = sessions_from_snapshot(
            &mut snapshot,
            &AttributionRoots {
                projects: vec![self.locations.home.clone()],
                current_project: None,
            },
        );
        self.snapshot = Some(snapshot);
        self.last_sample = Instant::now();
    }

    fn run_scan(&mut self) {
        let root = PathBuf::from(self.scan_root.trim());
        match scan_inventory(&root, InventoryLimits::default()) {
            Ok(report) => {
                self.inventory = Some(report);
                self.inventory_error = None;
            }
            Err(error) => {
                self.inventory_error = Some(error.to_string());
            }
        }
    }
}

impl eframe::App for SweepLoomApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.refresh_live();
        ctx.request_repaint_after(Duration::from_secs(1));

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.heading(RichText::new("SweepLoom").strong());
                ui.label(RichText::new("by Weavatrix").weak());
                ui.separator();
                ui.label("Reclaim your workstation without losing your workspace");
            });
            ui.add_space(4.0);
        });

        egui::SidePanel::left("nav")
            .resizable(false)
            .exact_width(168.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                for nav in Nav::ALL {
                    let selected = self.nav == nav;
                    if ui.selectable_label(selected, nav.label()).clicked() {
                        self.nav = nav;
                    }
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| match self.nav {
            Nav::Overview => self.ui_overview(ui),
            Nav::Sessions => self.ui_sessions(ui),
            Nav::Storage | Nav::Explorer => self.ui_storage(ui),
            Nav::Projects => self.ui_projects(ui),
            Nav::Browser => placeholder(ui, "Browser", "Optional companion for lastAccessed / Discard / Bookmark+Close. Native browser process totals already appear under Sessions."),
            Nav::Ai => placeholder(ui, "AI", "Inspect-first Claude/Codex storage. Search-before-delete is feature-gated later via weavatrix-search."),
            Nav::Rules => placeholder(ui, "Rules", "Declarative TOML cleaners. No shell from downloaded rules."),
            Nav::History => placeholder(ui, "History", "Observed history only — SweepLoom does not invent activity from before it started."),
            Nav::Settings => self.ui_settings(ui),
        });
    }
}

impl SweepLoomApp {
    fn ui_overview(&self, ui: &mut egui::Ui) {
        ui.heading("Overview");
        ui.add_space(8.0);
        let memory = self
            .snapshot
            .as_ref()
            .map(|item| item.memory)
            .unwrap_or_default();
        let cpu = self
            .snapshot
            .as_ref()
            .map(|item| item.cpu.usage_percent)
            .unwrap_or(0.0);
        let stale = self
            .sessions
            .iter()
            .filter(|session| {
                matches!(
                    session.recommendation.recommendation,
                    Recommendation::Recommended | Recommendation::StronglyRecommended
                )
            })
            .count();
        let reclaimable = self
            .sessions
            .iter()
            .filter(|session| session.recommendation.recommendation != Recommendation::Keep)
            .map(|session| session.recommendation.estimated_reclaimable_rss)
            .sum::<u64>();
        ui.horizontal(|ui| {
            metric_card(
                ui,
                "MEMORY",
                &format_bytes(memory.used_bytes),
                &format!("of {}", format_bytes(memory.total_bytes)),
            );
            metric_card(
                ui,
                "RECLAIMABLE SESSIONS",
                &format_bytes(reclaimable),
                &format!("{stale} stale candidates"),
            );
            metric_card(ui, "CPU", &format!("{cpu:.0}%"), "background load now");
            let disk = self.inventory.as_ref().map_or_else(
                || "scan Storage".to_owned(),
                |item| format_bytes(item.tree.logical_bytes),
            );
            metric_card(ui, "SCANNED DISK", &disk, "Folder Inspector");
        });
        ui.add_space(16.0);
        ui.label(RichText::new("Top opportunities").strong());
        ui.add_space(6.0);
        for session in self
            .sessions
            .iter()
            .filter(|item| item.recommendation.recommendation != Recommendation::Keep)
            .take(8)
        {
            ui.label(format!(
                "• {}  {}  {}  {:?}",
                session.label(),
                format_bytes(session.rss_bytes),
                session
                    .project
                    .as_ref()
                    .map(|item| item.0.display().to_string())
                    .unwrap_or_default(),
                session.recommendation.recommendation
            ));
        }
        if self
            .sessions
            .iter()
            .all(|item| item.recommendation.recommendation == Recommendation::Keep)
        {
            ui.label("No forgotten-session candidates in this sample. Keep watching.");
        }
    }

    fn ui_sessions(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Sessions");
            ui.separator();
            ui.checkbox(&mut self.group_raw, "Raw process tree");
        });
        ui.label(
            RichText::new(
                "Logical sessions sit on top of the OS process tree. Terminate is never automatic.",
            )
            .weak(),
        );
        ui.add_space(8.0);

        if self.group_raw {
            self.ui_process_table(ui);
            return;
        }

        let row_count = self.sessions.len();
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto().at_least(140.0))
            .column(Column::auto().at_least(70.0))
            .column(Column::auto().at_least(80.0))
            .column(Column::auto().at_least(70.0))
            .column(Column::auto().at_least(140.0))
            .column(Column::remainder())
            .header(22.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Session");
                });
                header.col(|ui| {
                    ui.strong("Procs");
                });
                header.col(|ui| {
                    ui.strong("RSS");
                });
                header.col(|ui| {
                    ui.strong("CPU");
                });
                header.col(|ui| {
                    ui.strong("Recommendation");
                });
                header.col(|ui| {
                    ui.strong("Project");
                });
            })
            .body(|mut body| {
                body.rows(20.0, row_count, |mut row| {
                    let index = row.index();
                    if let Some(session) = self.sessions.get(index) {
                        let selected = self.selected_session == Some(index);
                        row.col(|ui| {
                            if ui.selectable_label(selected, session.label()).clicked() {
                                self.selected_session = Some(index);
                            }
                        });
                        row.col(|ui| {
                            ui.label(session.processes.len().to_string());
                        });
                        row.col(|ui| {
                            ui.label(format_bytes(session.rss_bytes));
                        });
                        row.col(|ui| {
                            ui.label(format!("{:.1}%", session.cpu_percent));
                        });
                        row.col(|ui| {
                            ui.label(format!("{:?}", session.recommendation.recommendation));
                        });
                        row.col(|ui| {
                            ui.label(
                                session
                                    .project
                                    .as_ref()
                                    .map(|item| item.0.display().to_string())
                                    .unwrap_or_else(|| "Unknown".to_owned()),
                            );
                        });
                    }
                });
            });

        if let Some(index) = self.selected_session
            && let Some(session) = self.sessions.get(index)
        {
            ui.separator();
            session_details(ui, session, self.snapshot.as_ref());
        }
    }

    fn ui_process_table(&self, ui: &mut egui::Ui) {
        let Some(snapshot) = &self.snapshot else {
            return;
        };
        let rows = snapshot.processes.len();
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .column(Column::auto().at_least(70.0))
            .column(Column::auto().at_least(160.0))
            .column(Column::auto().at_least(80.0))
            .column(Column::auto().at_least(70.0))
            .column(Column::remainder())
            .header(22.0, |mut header| {
                header.col(|ui| {
                    ui.strong("PID");
                });
                header.col(|ui| {
                    ui.strong("Process");
                });
                header.col(|ui| {
                    ui.strong("RSS");
                });
                header.col(|ui| {
                    ui.strong("CPU");
                });
                header.col(|ui| {
                    ui.strong("Command");
                });
            })
            .body(|mut body| {
                body.rows(18.0, rows, |mut row| {
                    let index = row.index();
                    if let Some(process) = snapshot.processes.get(index) {
                        row.col(|ui| {
                            ui.label(process.pid.to_string());
                        });
                        row.col(|ui| {
                            ui.label(&process.name);
                        });
                        row.col(|ui| {
                            ui.label(format_bytes(process.rss_bytes));
                        });
                        row.col(|ui| {
                            ui.label(format!("{:.1}%", process.cpu_percent));
                        });
                        row.col(|ui| {
                            ui.monospace(process.command.join(" "));
                        });
                    }
                });
            });
    }

    fn ui_storage(&mut self, ui: &mut egui::Ui) {
        ui.heading("Storage / Explorer");
        ui.horizontal(|ui| {
            ui.label("Root");
            ui.add(egui::TextEdit::singleline(&mut self.scan_root).desired_width(480.0));
            if ui.button("Scan").clicked() {
                self.run_scan();
            }
        });
        if let Some(error) = &self.inventory_error {
            ui.colored_label(Color32::from_rgb(240, 120, 120), error);
        }
        if let Some(report) = &self.inventory {
            ui.label(format!(
                "entries {} · projects {} · logical {}{}",
                report.entries,
                report.projects.len(),
                format_bytes(report.tree.logical_bytes),
                if report.capped { " · capped" } else { "" }
            ));
            ui.add_space(8.0);
            folder_tree(ui, &report.tree, 0);
        } else {
            ui.label("Scan a folder to open the inspector. Symlinks are not followed.");
        }
    }

    fn ui_projects(&self, ui: &mut egui::Ui) {
        ui.heading("Projects");
        ui.label("Source Heat and Artifact Heat are independent. A fresh `target` does not make source HOT.");
        if let Some(report) = &self.inventory {
            for project in &report.projects {
                ui.label(project.display().to_string());
            }
        } else {
            ui.label("Run a Storage scan to discover project markers (Cargo.toml, package.json, pyproject.toml, …).");
        }
    }

    fn ui_settings(&self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.label("Telemetry: none.");
        ui.label("License: MPL-2.0 (SweepLoom). Weavatrix crates remain MIT.");
        ui.label(format!("Home: {}", self.locations.home.display()));
        ui.label(format!("Temp: {}", self.locations.temp.display()));
        if let Some(snapshot) = &self.snapshot {
            ui.label(format!("Observed processes: {}", snapshot.processes.len()));
        }
    }
}

fn session_details(
    ui: &mut egui::Ui,
    session: &LiveSession,
    snapshot: Option<&ProcessSnapshotSet>,
) {
    ui.label(RichText::new(session.label()).strong());
    ui.label(format!(
        "RAM {} · CPU {:.1}% · processes {} · {:?}",
        format_bytes(session.rss_bytes),
        session.cpu_percent,
        session.processes.len(),
        session.activity
    ));
    if session.safety.terminate_disabled {
        ui.colored_label(
            Color32::from_rgb(240, 160, 80),
            "Terminate disabled (system-critical).",
        );
    }
    if let Some(snapshot) = snapshot {
        for key in &session.processes {
            if let Some(process) = snapshot.processes.iter().find(|item| item.key == *key) {
                ui.monospace(format!(
                    "pid {}  {}  {}",
                    process.pid,
                    format_bytes(process.rss_bytes),
                    process.name
                ));
            }
        }
    }
    let _ = SessionKind::Unknown;
}

fn folder_tree(ui: &mut egui::Ui, node: &DirectoryNode, depth: usize) {
    if depth > 6 {
        return;
    }
    let label = format!(
        "{}  {}  {:?}",
        format_bytes(node.logical_bytes),
        node.path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("."),
        node.category
    );
    if node.children.is_empty() {
        ui.label(label);
        return;
    }
    egui::CollapsingHeader::new(label)
        .default_open(depth < 1)
        .show(ui, |ui| {
            for child in &node.children {
                folder_tree(ui, child, depth + 1);
            }
        });
}

fn metric_card(ui: &mut egui::Ui, title: &str, value: &str, sub: &str) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_min_width(200.0);
        ui.label(RichText::new(title).small().weak());
        ui.label(RichText::new(value).heading());
        ui.label(RichText::new(sub).weak());
    });
}

fn placeholder(ui: &mut egui::Ui, title: &str, body: &str) {
    ui.heading(title);
    ui.add_space(8.0);
    ui.label(body);
}

fn format_bytes(bytes: u64) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    let value = bytes as f64;
    if value >= GIB {
        format!("{:.1} GB", value / GIB)
    } else if value >= MIB {
        format!("{:.1} MB", value / MIB)
    } else {
        format!("{} KB", (value / 1024.0).round())
    }
}

#[allow(dead_code)]
fn _use_process_snapshot(process: &ProcessSnapshot) -> u32 {
    process.pid
}

#[allow(dead_code)]
fn _use_host_memory(memory: HostMemory) -> u64 {
    memory.total_bytes
}
