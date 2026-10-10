//! Metric cards, stat tiles and clickable list rows.

use eframe::egui::{self, CornerRadius, Margin, RichText, Sense};

use super::{Tone, hover_stroke, meter, pointer, sparkline_max};
use crate::icons::{self, Glyph};
use crate::nav::Nav;
use crate::theme;

/// Small chart under a tile's number.
#[derive(Clone, Debug, Default)]
pub enum TileViz {
    /// Text only.
    #[default]
    None,
    /// 0–1 ratio bar.
    Meter(f32),
    /// Observed series with a fixed top (e.g. 100 for percent).
    Trend(Vec<f32>, Option<f32>),
}

/// One stat tile: caption, big number, supporting line, optional chart.
#[derive(Clone, Debug)]
pub struct StatTile {
    /// Optional stroke icon before the caption.
    pub icon: Option<Glyph>,
    /// Short caption.
    pub label: String,
    /// Primary value.
    pub value: String,
    /// Supporting line.
    pub sub: String,
    /// Value coloring.
    pub tone: Tone,
    /// Chart under the number.
    pub viz: TileViz,
}

/// One overview metric that opens a screen.
pub struct Metric {
    /// Tile contents.
    pub tile: StatTile,
    /// Screen opened when the card is clicked.
    pub open: Nav,
}

/// Lay out equal tiles in as many columns as fit. Returns the clicked index.
pub fn tile_grid<T>(
    ui: &mut egui::Ui,
    items: &[T],
    min_width: f32,
    height: f32,
    mut draw: impl FnMut(&mut egui::Ui, &T, egui::Vec2) -> egui::Response,
) -> Option<usize> {
    if items.is_empty() {
        return None;
    }
    let gap = theme::MD;
    let width = ui.available_width();
    let fit = (((width + gap) / (min_width + gap)).floor() as usize).max(1);
    let cols = fit.min(items.len());
    let tile_w = ((width - gap * (cols - 1) as f32) / cols as f32).floor();
    let mut clicked = None;
    for (row, chunk) in items.chunks(cols).enumerate() {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for (offset, item) in chunk.iter().enumerate() {
                if draw(ui, item, egui::vec2(tile_w, height)).clicked() {
                    clicked = Some(row * cols + offset);
                }
            }
        });
        ui.add_space(gap);
    }
    clicked
}

/// Overview metric cards. Returns the screen to open.
pub fn metric_grid(ui: &mut egui::Ui, cards: &[Metric]) -> Option<Nav> {
    tile_grid(ui, cards, 220.0, 138.0, |ui, card, size| {
        let response = pointer(stat_tile(ui, &card.tile, size, true));
        hover_stroke(
            ui,
            ui.id().with(&card.tile.label),
            response.rect,
            response.hovered(),
            10.0,
        );
        response
    })
    .map(|index| cards[index].open)
}

/// Static stat tiles (no navigation).
pub fn stat_row(ui: &mut egui::Ui, tiles: &[StatTile], min_width: f32) {
    let height = if tiles.iter().any(|t| !matches!(t.viz, TileViz::None)) {
        126.0
    } else {
        96.0
    };
    let _ = tile_grid(ui, tiles, min_width, height, |ui, tile, size| {
        stat_tile(ui, tile, size, false)
    });
}

/// Draw one tile in a fixed-size card.
pub fn stat_tile(
    ui: &mut egui::Ui,
    tile: &StatTile,
    size: egui::Vec2,
    click: bool,
) -> egui::Response {
    let color = theme::tone(ui, tile.tone);
    card_box(ui, size, click, |ui| {
        ui.spacing_mut().item_spacing.y = 2.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            if let Some(icon) = tile.icon {
                icons::show(ui, icon, 14.0, theme::muted(ui));
            }
            ui.add(
                egui::Label::new(
                    RichText::new(&tile.label)
                        .size(12.5)
                        .color(theme::muted(ui)),
                )
                .selectable(false)
                .truncate(),
            );
        });
        ui.add_space(4.0);
        ui.add(
            egui::Label::new(RichText::new(&tile.value).size(24.0).strong().color(color))
                .selectable(false)
                .truncate(),
        );
        ui.add(
            egui::Label::new(RichText::new(&tile.sub).size(12.5).color(theme::muted(ui)))
                .selectable(false)
                .truncate(),
        );
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            let width = ui.available_width().max(24.0);
            let accent = if tile.tone == Tone::Neutral {
                theme::series(ui, 0)
            } else {
                color
            };
            match &tile.viz {
                TileViz::None => {}
                TileViz::Meter(fill) => meter(ui, *fill, accent, width, 6.0),
                TileViz::Trend(values, max) => {
                    sparkline_max(ui, values, egui::vec2(width, 24.0), accent, *max);
                }
            }
        });
    })
}

/// One heavy session for the Overview grid.
pub struct PressureItem {
    /// Display name.
    pub title: String,
    /// Formatted RSS.
    pub rss: String,
    /// Combined CPU percent.
    pub cpu: f32,
    /// Share of in-use RAM, 0–1.
    pub ram_share: f32,
}

/// Heavy sessions: equal-height cards, as many per row as fit.
pub fn pressure_grid(ui: &mut egui::Ui, items: &[PressureItem]) -> Option<usize> {
    tile_grid(ui, items, 300.0, 82.0, pressure_card)
}

fn pressure_card(ui: &mut egui::Ui, item: &PressureItem, size: egui::Vec2) -> egui::Response {
    let id = ui.id().with(&item.title);
    let cpu_tone = if item.cpu >= 80.0 {
        Tone::Warn
    } else {
        Tone::Muted
    };
    let clicked = pointer(card_box(ui, size, true, |ui| {
        ui.horizontal(|ui| {
            ui.add(
                egui::Label::new(RichText::new(&item.title).size(14.5).strong())
                    .selectable(false)
                    .truncate(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(format!("{:.0}% CPU", item.cpu))
                            .size(12.5)
                            .color(theme::tone(ui, cpu_tone)),
                    )
                    .selectable(false),
                );
                ui.add_space(theme::SM);
                ui.add(
                    egui::Label::new(RichText::new(&item.rss).size(14.5).strong())
                        .selectable(false),
                );
            });
        });
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(format!("{:.0}% of RAM in use", item.ram_share * 100.0))
                            .size(12.0)
                            .color(theme::muted(ui)),
                    )
                    .selectable(false),
                );
                let width = (ui.available_width() - 4.0).max(48.0);
                meter(ui, item.ram_share, theme::series(ui, 0), width, 6.0);
            });
        });
    }));
    hover_stroke(ui, id, clicked.rect, clicked.hovered(), 10.0);
    clicked
}

/// Fixed-size card. `click` makes the whole card a button.
pub fn card_box(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    click: bool,
    add_contents: impl FnOnce(&mut egui::Ui),
) -> egui::Response {
    let sense = if click {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(size, sense);
    ui.painter().rect(
        rect,
        CornerRadius::same(theme::RADIUS_LG),
        theme::card_fill(ui),
        theme::card_stroke(ui),
        egui::StrokeKind::Inside,
    );
    let inner = rect.shrink2(egui::vec2(theme::LG, 14.0));
    // A detached child: content never moves the parent's cursor past `size`.
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    child.set_clip_rect(inner.expand(4.0).intersect(ui.clip_rect()));
    add_contents(&mut child);
    response
}

/// Clickable listing row. Returns the row response.
pub fn list_row_at(ui: &mut egui::Ui, title: &str, meta: &str, detail: &str) -> egui::Response {
    let id = ui.id().with(title);
    let inner = egui::Frame::default()
        .fill(theme::card_fill(ui))
        .stroke(theme::card_stroke(ui))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::Label::new(RichText::new(title).strong())
                        .selectable(false)
                        .truncate(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if !meta.is_empty() {
                        ui.add(egui::Label::new(RichText::new(meta).strong()).selectable(false));
                    }
                    if !detail.is_empty() {
                        ui.add(
                            egui::Label::new(
                                RichText::new(detail).size(12.5).color(theme::muted(ui)),
                            )
                            .selectable(false)
                            .truncate(),
                        );
                    }
                });
            });
        });
    let response = pointer(inner.response.interact(Sense::click()));
    hover_stroke(ui, id, response.rect, response.hovered(), 8.0);
    ui.add_space(6.0);
    response
}
