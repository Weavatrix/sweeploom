//! Thin-mark charts: sparklines, meters and segmented breakdown bars.

use eframe::egui::{self, Color32, CornerRadius, RichText, Sense};

use crate::theme;

/// Tiny observed series scaled to its own range. Does not invent missing samples.
pub fn sparkline(ui: &mut egui::Ui, values: &[f32], size: egui::Vec2, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let min = values.iter().copied().fold(f32::MAX, f32::min);
    let max = values.iter().copied().fold(0.0_f32, f32::max);
    paint_series(ui, rect, values, min.min(max), max, color);
}

/// Series on a zero baseline. `max` fixes the top (e.g. 100 for percent).
pub fn sparkline_max(
    ui: &mut egui::Ui,
    values: &[f32],
    size: egui::Vec2,
    color: Color32,
    max: Option<f32>,
) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let top = max.unwrap_or_else(|| values.iter().copied().fold(0.0_f32, f32::max));
    paint_series(ui, rect, values, 0.0, top, color);
}

/// Line + soft area + end dot inside `rect`.
pub fn paint_series(
    ui: &egui::Ui,
    rect: egui::Rect,
    values: &[f32],
    min: f32,
    max: f32,
    color: Color32,
) {
    if values.len() < 2 {
        return;
    }
    let span = (max - min).max(0.01);
    let last = (values.len() - 1) as f32;
    let points: Vec<egui::Pos2> = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let x = rect.left() + rect.width() * (index as f32 / last);
            let t = ((*value - min) / span).clamp(0.0, 1.0);
            egui::pos2(x, rect.bottom() - 1.0 - (rect.height() - 2.0) * t)
        })
        .collect();
    let fill = color.gamma_multiply(if ui.visuals().dark_mode { 0.16 } else { 0.12 });
    let mut mesh = egui::Mesh::default();
    for pair in points.windows(2) {
        let base = mesh.vertices.len() as u32;
        for pos in [
            pair[0],
            egui::pos2(pair[0].x, rect.bottom()),
            pair[1],
            egui::pos2(pair[1].x, rect.bottom()),
        ] {
            mesh.colored_vertex(pos, fill);
        }
        mesh.add_triangle(base, base + 1, base + 2);
        mesh.add_triangle(base + 1, base + 3, base + 2);
    }
    let painter = ui.painter_at(rect.expand(3.0));
    painter.add(egui::Shape::mesh(mesh));
    let end = *points.last().unwrap_or(&rect.right_bottom());
    painter.add(egui::Shape::line(points, egui::Stroke::new(1.6_f32, color)));
    painter.circle_filled(end, 2.6, color);
}

/// Rounded 0–1 bar on the inset track.
pub fn meter(ui: &mut egui::Ui, fill: f32, color: Color32, width: f32, height: f32) {
    let fill = if fill.is_finite() {
        fill.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width.max(8.0), height), Sense::hover());
    let radius = (height / 2.0).min(4.0);
    ui.painter().rect_filled(rect, radius, theme::inset(ui));
    if fill > 0.005 {
        let mut used = rect;
        used.set_width((rect.width() * fill).max(height));
        ui.painter().rect_filled(used, radius, color);
    }
}

/// One slice of a breakdown bar.
#[derive(Clone, Debug)]
pub struct Segment {
    /// Legend label.
    pub label: String,
    /// Raw magnitude (bytes, percent…).
    pub value: f64,
    /// Formatted value for legend and tooltip.
    pub detail: String,
    /// Fill.
    pub color: Color32,
}

/// Horizontal stacked bar with 2 px surface gaps and hover tooltips.
pub fn segment_bar(ui: &mut egui::Ui, segments: &[Segment], height: f32) {
    let width = ui.available_width().max(40.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), Sense::hover());
    let total: f64 = segments.iter().map(|s| s.value.max(0.0)).sum();
    let radius = (height / 2.0).min(5.0) as u8;
    if total <= 0.0 {
        ui.painter().rect_filled(rect, radius, theme::inset(ui));
        return;
    }
    let visible: Vec<&Segment> = segments.iter().filter(|s| s.value > 0.0).collect();
    let gap = 2.0;
    let usable = rect.width() - gap * (visible.len().saturating_sub(1)) as f32;
    let mut x = rect.left();
    let hover = response.hover_pos();
    for (index, segment) in visible.iter().enumerate() {
        let w = (usable * (segment.value / total) as f32).max(2.0);
        let part = egui::Rect::from_min_size(egui::pos2(x, rect.top()), egui::vec2(w, height));
        let first = index == 0;
        let last = index + 1 == visible.len();
        let corner = CornerRadius {
            nw: if first { radius } else { 1 },
            sw: if first { radius } else { 1 },
            ne: if last { radius } else { 1 },
            se: if last { radius } else { 1 },
        };
        let hot = hover.is_some_and(|pos| part.contains(pos));
        let color = if hot {
            theme::lerp(segment.color, ui.visuals().text_color(), 0.18)
        } else {
            segment.color
        };
        ui.painter().rect_filled(part, corner, color);
        if hot {
            let share = 100.0 * segment.value / total;
            response.clone().on_hover_ui_at_pointer(|ui| {
                ui.strong(&segment.label);
                ui.label(format!("{} · {share:.0}%", segment.detail));
            });
        }
        x += w + gap;
    }
}

/// Wrapped legend: swatch, label, value. Text stays in ink, color only on the swatch.
pub fn legend(ui: &mut egui::Ui, segments: &[Segment]) {
    let font = egui::FontId::proportional(12.5);
    let ink = ui.visuals().text_color();
    let muted = theme::muted(ui);
    let edge = theme::border_strong(ui).gamma_multiply(0.5);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(theme::LG, 6.0);
        for segment in segments {
            let painter = ui.painter();
            let label = painter.layout_no_wrap(segment.label.clone(), font.clone(), ink);
            let detail = painter.layout_no_wrap(segment.detail.clone(), font.clone(), muted);
            let width = 16.0 + label.size().x + 6.0 + detail.size().x;
            let height = label.size().y.max(14.0);
            let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), Sense::hover());
            let swatch = egui::Rect::from_center_size(
                egui::pos2(rect.left() + 5.0, rect.center().y),
                egui::vec2(10.0, 10.0),
            );
            let painter = ui.painter();
            painter.rect(
                swatch,
                3.0,
                segment.color,
                egui::Stroke::new(1.0_f32, edge),
                egui::StrokeKind::Inside,
            );
            let y = rect.center().y - label.size().y / 2.0;
            let detail_x = rect.left() + 16.0 + label.size().x + 6.0;
            painter.galley(egui::pos2(rect.left() + 16.0, y), label, ink);
            painter.galley(egui::pos2(detail_x, y), detail, muted);
        }
    });
}

/// Titled breakdown: header with total, bar, legend.
pub fn breakdown(ui: &mut egui::Ui, title: &str, total: &str, segments: &[Segment]) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(title).size(13.5).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(total).size(13.0).color(theme::muted(ui)));
        });
    });
    ui.add_space(6.0);
    segment_bar(ui, segments, 12.0);
    ui.add_space(theme::SM);
    legend(ui, segments);
}
