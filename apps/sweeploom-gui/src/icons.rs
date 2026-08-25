//! Stroke icons. No emoji font, so sort arrows cannot become tofu squares.

use eframe::egui::{self, Color32, Pos2, Stroke};

/// Small geometric glyph used in chrome and cards.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {
    /// Live overview.
    Overview,
    /// Session list.
    Sessions,
    /// History clock.
    History,
    /// Storage review.
    Review,
    /// Folder inspector.
    Explorer,
    /// Project crate.
    Projects,
    /// Browser window.
    Browser,
    /// AI spark.
    Ai,
    /// Rules sliders.
    Rules,
    /// Settings gear.
    Settings,
    /// Memory chip.
    Memory,
    /// CPU bars.
    Cpu,
    /// Disk platter.
    Disk,
    /// Volume / free space.
    Volume,
    /// Sort descending.
    SortDesc,
    /// Sort ascending.
    SortAsc,
}

/// Allocate a square and paint `glyph`.
pub fn show(ui: &mut egui::Ui, glyph: Glyph, size: f32, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    paint(ui.painter(), glyph, rect.shrink(size * 0.12), color);
}

/// Paint `glyph` into `rect`.
pub fn paint(painter: &egui::Painter, glyph: Glyph, rect: egui::Rect, color: Color32) {
    let stroke = Stroke::new((rect.width() * 0.09).clamp(1.1, 1.8), color);
    match glyph {
        Glyph::Overview => tiles(painter, rect, stroke),
        Glyph::Sessions => lines(painter, rect, stroke),
        Glyph::History => clock(painter, rect, stroke, color),
        Glyph::Review => stack(painter, rect, stroke),
        Glyph::Explorer => folder(painter, rect, stroke),
        Glyph::Projects => crate_box(painter, rect, stroke),
        Glyph::Browser => window(painter, rect, stroke, color),
        Glyph::Ai => spark(painter, rect, stroke),
        Glyph::Rules => sliders(painter, rect, stroke, color),
        Glyph::Settings => gear(painter, rect, stroke, color),
        Glyph::Memory => chip(painter, rect, stroke),
        Glyph::Cpu => bars(painter, rect, color),
        Glyph::Disk => platter(painter, rect, stroke, color),
        Glyph::Volume => drive(painter, rect, stroke),
        Glyph::SortDesc => triangle(painter, rect, color, true),
        Glyph::SortAsc => triangle(painter, rect, color, false),
    }
}

fn tiles(painter: &egui::Painter, rect: egui::Rect, stroke: Stroke) {
    let gap = rect.width() * 0.12;
    let w = (rect.width() - gap) * 0.5;
    for (dx, dy) in [
        (0.0, 0.0),
        (w + gap, 0.0),
        (0.0, w + gap),
        (w + gap, w + gap),
    ] {
        painter.rect_stroke(
            egui::Rect::from_min_size(rect.min + egui::vec2(dx, dy), egui::vec2(w, w)),
            2.0,
            stroke,
            egui::StrokeKind::Middle,
        );
    }
}

fn lines(painter: &egui::Painter, rect: egui::Rect, stroke: Stroke) {
    let y0 = rect.center().y - rect.height() * 0.28;
    for i in 0..3 {
        let y = y0 + rect.height() * 0.28 * i as f32;
        painter.line_segment(
            [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
            stroke,
        );
    }
}

fn clock(painter: &egui::Painter, rect: egui::Rect, stroke: Stroke, color: Color32) {
    painter.circle_stroke(rect.center(), rect.width() * 0.42, stroke);
    painter.line_segment(
        [
            rect.center(),
            rect.center() + egui::vec2(0.0, -rect.height() * 0.28),
        ],
        stroke,
    );
    painter.line_segment(
        [
            rect.center(),
            rect.center() + egui::vec2(rect.width() * 0.22, 0.08),
        ],
        stroke,
    );
    painter.circle_filled(rect.center(), 1.6, color);
}

fn stack(painter: &egui::Painter, rect: egui::Rect, stroke: Stroke) {
    let h = rect.height() * 0.22;
    for i in 0..3 {
        let y = rect.top() + rect.height() * 0.12 + (h + rect.height() * 0.1) * i as f32;
        painter.rect_stroke(
            egui::Rect::from_min_max(
                Pos2::new(rect.left() + rect.width() * 0.08 * i as f32, y),
                Pos2::new(rect.right() - rect.width() * 0.08 * i as f32, y + h),
            ),
            2.0,
            stroke,
            egui::StrokeKind::Middle,
        );
    }
}

fn folder(painter: &egui::Painter, rect: egui::Rect, stroke: Stroke) {
    let tab = egui::Rect::from_min_size(
        rect.min,
        egui::vec2(rect.width() * 0.42, rect.height() * 0.22),
    );
    painter.rect_stroke(tab, 2.0, stroke, egui::StrokeKind::Middle);
    painter.rect_stroke(
        egui::Rect::from_min_max(
            Pos2::new(rect.left(), rect.top() + rect.height() * 0.18),
            rect.max,
        ),
        3.0,
        stroke,
        egui::StrokeKind::Middle,
    );
}

fn crate_box(painter: &egui::Painter, rect: egui::Rect, stroke: Stroke) {
    painter.rect_stroke(
        rect.shrink(rect.width() * 0.08),
        3.0,
        stroke,
        egui::StrokeKind::Middle,
    );
    painter.line_segment([rect.left_center(), rect.right_center()], stroke);
}

fn window(painter: &egui::Painter, rect: egui::Rect, stroke: Stroke, color: Color32) {
    painter.rect_stroke(rect.shrink(1.0), 3.0, stroke, egui::StrokeKind::Middle);
    let y = rect.top() + rect.height() * 0.28;
    painter.line_segment(
        [
            Pos2::new(rect.left() + 1.0, y),
            Pos2::new(rect.right() - 1.0, y),
        ],
        stroke,
    );
    painter.circle_filled(
        Pos2::new(
            rect.left() + rect.width() * 0.22,
            rect.top() + rect.height() * 0.14,
        ),
        1.6,
        color,
    );
}

fn spark(painter: &egui::Painter, rect: egui::Rect, stroke: Stroke) {
    let c = rect.center();
    painter.line_segment(
        [Pos2::new(c.x, rect.top()), Pos2::new(c.x, rect.bottom())],
        stroke,
    );
    painter.line_segment(
        [Pos2::new(rect.left(), c.y), Pos2::new(rect.right(), c.y)],
        stroke,
    );
    painter.line_segment(
        [
            Pos2::new(rect.left() + 2.0, rect.top() + 2.0),
            Pos2::new(rect.right() - 2.0, rect.bottom() - 2.0),
        ],
        stroke,
    );
}

fn sliders(painter: &egui::Painter, rect: egui::Rect, stroke: Stroke, color: Color32) {
    for (i, t) in [0.28_f32, 0.62].into_iter().enumerate() {
        let x = rect.left() + rect.width() * (0.32 + 0.36 * i as f32);
        painter.line_segment(
            [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
            stroke,
        );
        painter.circle_filled(Pos2::new(x, rect.top() + rect.height() * t), 2.4, color);
    }
}

fn gear(painter: &egui::Painter, rect: egui::Rect, stroke: Stroke, color: Color32) {
    painter.circle_stroke(rect.center(), rect.width() * 0.22, stroke);
    painter.circle_filled(rect.center(), 1.5, color);
    for i in 0..6 {
        let a = (i as f32) * std::f32::consts::TAU / 6.0;
        let inner = rect.center() + egui::vec2(a.cos(), a.sin()) * rect.width() * 0.22;
        let outer = rect.center() + egui::vec2(a.cos(), a.sin()) * rect.width() * 0.46;
        painter.line_segment([inner, outer], stroke);
    }
}

fn chip(painter: &egui::Painter, rect: egui::Rect, stroke: Stroke) {
    painter.rect_stroke(
        rect.shrink(rect.width() * 0.16),
        2.0,
        stroke,
        egui::StrokeKind::Middle,
    );
}

fn bars(painter: &egui::Painter, rect: egui::Rect, color: Color32) {
    let w = rect.width() * 0.18;
    let gap = rect.width() * 0.1;
    let heights = [0.45, 0.85, 0.6, 0.95];
    for (i, h) in heights.iter().enumerate() {
        let x = rect.left() + (w + gap) * i as f32;
        let top = rect.bottom() - rect.height() * h;
        painter.rect_filled(
            egui::Rect::from_min_max(Pos2::new(x, top), Pos2::new(x + w, rect.bottom())),
            1.0,
            color,
        );
    }
}

fn platter(painter: &egui::Painter, rect: egui::Rect, stroke: Stroke, color: Color32) {
    painter.circle_stroke(rect.center(), rect.width() * 0.42, stroke);
    painter.circle_stroke(rect.center(), rect.width() * 0.22, stroke);
    painter.circle_filled(rect.center(), 1.8, color);
}

fn drive(painter: &egui::Painter, rect: egui::Rect, stroke: Stroke) {
    painter.rect_stroke(
        rect.shrink2(egui::vec2(1.0, rect.height() * 0.18)),
        3.0,
        stroke,
        egui::StrokeKind::Middle,
    );
}

fn triangle(painter: &egui::Painter, rect: egui::Rect, color: Color32, down: bool) {
    let c = rect.center();
    let w = rect.width() * 0.36;
    let h = rect.height() * 0.32;
    let (a, b, d) = if down {
        (
            Pos2::new(c.x, c.y + h),
            Pos2::new(c.x - w, c.y - h),
            Pos2::new(c.x + w, c.y - h),
        )
    } else {
        (
            Pos2::new(c.x, c.y - h),
            Pos2::new(c.x - w, c.y + h),
            Pos2::new(c.x + w, c.y + h),
        )
    };
    painter.add(egui::Shape::convex_polygon(
        vec![a, b, d],
        color,
        Stroke::NONE,
    ));
}
