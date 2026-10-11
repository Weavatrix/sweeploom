//! Per-core CPU infographic. Samples ride the existing refresh; nothing extra polls.

use std::collections::VecDeque;
use std::time::SystemTime;

use eframe::egui::{self, RichText, Sense};
use sweeploom_process::{CoreSplit, ProcessSnapshotSet};

use crate::theme;

/// Samples kept per core (~1 per second).
pub const HISTORY: usize = 60;
const KEY: &str = "sweeploom-cpu-cores";

/// Rolling per-core history, newest last.
#[derive(Clone, Debug, Default)]
pub struct CoreHistory {
    stamp: Option<SystemTime>,
    /// Whole-machine load.
    pub total: VecDeque<f32>,
    /// One series per logical core, OS order.
    pub cores: Vec<VecDeque<f32>>,
    /// P/E split when known.
    pub split: Option<CoreSplit>,
    /// Physical cores when known.
    pub physical: Option<usize>,
}

impl CoreHistory {
    fn push(&mut self, snapshot: &ProcessSnapshotSet) {
        if self.stamp == Some(snapshot.captured_at) {
            return;
        }
        self.stamp = Some(snapshot.captured_at);
        let cpu = &snapshot.cpu;
        if self.cores.len() != cpu.cores.len() {
            self.cores = vec![VecDeque::with_capacity(HISTORY); cpu.cores.len()];
        }
        for (series, value) in self.cores.iter_mut().zip(&cpu.cores) {
            push(series, *value);
        }
        push(&mut self.total, cpu.usage_percent);
        self.split = cpu.split;
        self.physical = cpu.physical_cores;
    }

    /// Latest value for a core.
    #[must_use]
    pub fn now(&self, index: usize) -> f32 {
        self.cores
            .get(index)
            .and_then(|series| series.back().copied())
            .unwrap_or(0.0)
    }

    /// Cores in display order: performance first, then efficiency.
    #[must_use]
    pub fn groups(&self) -> Vec<(&'static str, Vec<(String, usize)>)> {
        let count = self.cores.len();
        match self.split.filter(|s| s.performance + s.efficiency == count) {
            Some(split) => vec![
                (
                    "Performance",
                    (split.efficiency..count)
                        .enumerate()
                        .map(|(n, i)| (format!("P{}", n + 1), i))
                        .collect(),
                ),
                (
                    "Efficiency",
                    (0..split.efficiency)
                        .map(|i| (format!("E{}", i + 1), i))
                        .collect(),
                ),
            ],
            None => vec![(
                "Cores",
                (0..count).map(|i| (format!("{}", i + 1), i)).collect(),
            )],
        }
    }

    /// "10 cores · 4P + 6E".
    #[must_use]
    pub fn summary(&self) -> String {
        let mut text = format!("{} logical cores", self.cores.len());
        if let Some(split) = self.split {
            text = format!(
                "{} cores · {}P + {}E",
                self.cores.len(),
                split.performance,
                split.efficiency
            );
        } else if let Some(physical) = self.physical.filter(|p| *p != self.cores.len()) {
            text.push_str(&format!(" · {physical} physical"));
        }
        text
    }
}

fn push(series: &mut VecDeque<f32>, value: f32) {
    if series.len() == HISTORY {
        series.pop_front();
    }
    series.push_back(if value.is_finite() { value } else { 0.0 });
}

/// Record the latest snapshot. Call once per frame; duplicates are ignored.
pub fn record(ctx: &egui::Context, snapshot: &ProcessSnapshotSet) {
    ctx.data_mut(|data| {
        data.get_temp_mut_or_default::<CoreHistory>(egui::Id::new(KEY))
            .push(snapshot);
    });
}

/// Copy of the current history.
#[must_use]
pub fn history(ctx: &egui::Context) -> CoreHistory {
    ctx.data(|data| data.get_temp::<CoreHistory>(egui::Id::new(KEY)))
        .unwrap_or_default()
}

/// Core cells grouped P, then E. Each group's cells share its row evenly.
pub fn grid(ui: &mut egui::Ui, history: &CoreHistory) {
    if history.cores.is_empty() {
        crate::widgets::caption(ui, "Per-core load appears after the second sample.");
        return;
    }
    let gap = theme::SM;
    let label_w = 96.0;
    for (name, cores) in &history.groups() {
        for chunk in cores.chunks(8) {
            let n = chunk.len() as f32;
            let cell_w = ((ui.available_width() - label_w - gap * n) / n).max(48.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                group_label(ui, name, chunk, history, label_w);
                for (label, index) in chunk {
                    cell(ui, history, label, *index, egui::vec2(cell_w, 64.0));
                }
            });
            ui.add_space(gap);
        }
    }
}

fn group_label(
    ui: &mut egui::Ui,
    name: &str,
    cores: &[(String, usize)],
    history: &CoreHistory,
    width: f32,
) {
    let avg = cores.iter().map(|(_, i)| history.now(*i)).sum::<f32>() / cores.len().max(1) as f32;
    ui.allocate_ui_with_layout(
        egui::vec2(width, 40.0),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.label(RichText::new(name).size(12.5).strong());
            ui.label(
                RichText::new(format!("{} cores · {avg:.0}%", cores.len()))
                    .size(12.0)
                    .color(theme::muted(ui)),
            );
        },
    );
}

fn cell(ui: &mut egui::Ui, history: &CoreHistory, label: &str, index: usize, size: egui::Vec2) {
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    let now = history.now(index);
    let color = theme::load(ui, now);
    ui.painter()
        .rect_filled(rect, theme::RADIUS, theme::inset(ui));
    let inner = rect.shrink2(egui::vec2(10.0, 8.0));
    ui.painter().text(
        inner.left_top(),
        egui::Align2::LEFT_TOP,
        label,
        egui::FontId::proportional(12.0),
        theme::muted(ui),
    );
    ui.painter().text(
        inner.right_top(),
        egui::Align2::RIGHT_TOP,
        format!("{now:.0}%"),
        egui::FontId::proportional(13.0),
        if now >= 85.0 {
            theme::tone(ui, theme::Tone::Warn)
        } else {
            ui.visuals().text_color()
        },
    );
    let chart = egui::Rect::from_min_max(egui::pos2(inner.left(), inner.top() + 22.0), inner.max);
    let values: Vec<f32> = history
        .cores
        .get(index)
        .map(|series| series.iter().copied().collect())
        .unwrap_or_default();
    if values.len() < 2 {
        let bar = egui::Rect::from_min_max(
            egui::pos2(chart.left(), chart.bottom() - 3.0),
            egui::pos2(chart.left() + chart.width() * now / 100.0, chart.bottom()),
        );
        ui.painter().rect_filled(bar, 1.5, color);
    } else {
        crate::widgets::paint_series(ui, chart, &values, 0.0, 100.0, color);
    }
    response.on_hover_text(core_tip(label, history.cores.get(index)));
}

fn core_tip(label: &str, series: Option<&VecDeque<f32>>) -> String {
    let Some(series) = series.filter(|s| !s.is_empty()) else {
        return format!("Core {label}: no samples yet");
    };
    let avg = series.iter().sum::<f32>() / series.len() as f32;
    let peak = series.iter().copied().fold(0.0_f32, f32::max);
    format!(
        "Core {label}\nNow {:.0}% · avg {avg:.0}% · peak {peak:.0}%\nLast {} samples (~1 s each)",
        series.back().copied().unwrap_or(0.0),
        series.len()
    )
}

/// Compact heat strip: one row per core, time runs left to right (~1 s per column).
pub fn heat_strip(ui: &mut egui::Ui, history: &CoreHistory) {
    if history.cores.is_empty() {
        crate::widgets::caption(ui, "Per-core load appears after the second sample.");
        return;
    }
    let (row_h, gap, group_gap) = (10.0, 4.0, 10.0);
    let groups = history.groups();
    let rows: usize = groups.iter().map(|(_, cores)| cores.len()).sum();
    let height = rows as f32 * (row_h + gap) + (groups.len() - 1) as f32 * group_gap;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), height), Sense::hover());
    let font = egui::FontId::proportional(11.0);
    let muted = theme::muted(ui);
    let mut y = rect.top();
    let mut tip = None;
    for (_, cores) in &groups {
        for (label, index) in cores {
            let row = egui::Rect::from_min_size(
                egui::pos2(rect.left(), y),
                egui::vec2(rect.width(), row_h),
            );
            let painter = ui.painter();
            painter.text(
                row.left_center(),
                egui::Align2::LEFT_CENTER,
                label,
                font.clone(),
                muted,
            );
            painter.text(
                row.right_center(),
                egui::Align2::RIGHT_CENTER,
                format!("{:.0}%", history.now(*index)),
                font.clone(),
                muted,
            );
            let bar = egui::Rect::from_min_max(
                egui::pos2(row.left() + 28.0, row.top()),
                egui::pos2(row.right() - 40.0, row.bottom()),
            );
            strip(ui, bar, history.cores.get(*index));
            if response
                .hover_pos()
                .is_some_and(|pos| row.expand2(egui::vec2(0.0, gap / 2.0)).contains(pos))
            {
                tip = Some(core_tip(label, history.cores.get(*index)));
            }
            y += row_h + gap;
        }
        y += group_gap;
    }
    if let Some(tip) = tip {
        response.on_hover_text(tip);
    }
}

fn strip(ui: &egui::Ui, rect: egui::Rect, series: Option<&VecDeque<f32>>) {
    ui.painter().rect_filled(rect, 2.0, theme::inset(ui));
    let Some(series) = series else {
        return;
    };
    let step = rect.width() / HISTORY as f32;
    let offset = HISTORY - series.len().min(HISTORY);
    for (n, value) in series.iter().enumerate() {
        let x = rect.left() + (offset + n) as f32 * step;
        let cell = egui::Rect::from_min_max(
            egui::pos2(x, rect.top()),
            egui::pos2(x + step + 0.5, rect.bottom()),
        );
        ui.painter().rect_filled(cell, 0.0, theme::load(ui, *value));
    }
}
