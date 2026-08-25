//! Clickable column sort used by Sessions, Review, and Explorer.

use crate::icons::{self, Glyph};
use crate::theme;
use crate::widgets;
use eframe::egui::{self, RichText};

/// Which column is driving the current order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Col {
    /// Label / path / process name.
    Name,
    /// Logical bytes or RSS.
    Size,
    /// CPU percent.
    Cpu,
    /// Process count.
    Procs,
    /// Recommendation / rebuild / safety.
    Status,
}

/// Current sort.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sort {
    /// Column.
    pub col: Col,
    /// True when high values come first.
    pub desc: bool,
}

impl Sort {
    /// Default: largest first.
    #[must_use]
    pub const fn size_desc() -> Self {
        Self {
            col: Col::Size,
            desc: true,
        }
    }

    /// Toggle the same column, or switch to a new one.
    pub fn toggle(&mut self, col: Col) {
        if self.col == col {
            self.desc = !self.desc;
        } else {
            self.col = col;
            self.desc = !matches!(col, Col::Name | Col::Status);
        }
    }
}

/// Sortable table header cell. Looks like a column title, not a toolbar button.
pub fn header_cell(ui: &mut egui::Ui, sort: &mut Sort, col: Col, name: &str) {
    let active = sort.col == col;
    let mut text = RichText::new(name).strong();
    if active {
        text = text.color(theme::accent());
    }
    let inner = egui::Frame::new().show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            ui.add(egui::Label::new(text).selectable(false));
            if active {
                let glyph = if sort.desc {
                    Glyph::SortDesc
                } else {
                    Glyph::SortAsc
                };
                icons::show(ui, glyph, 12.0, theme::accent());
            }
        });
    });
    if widgets::pointer(inner.response.interact(egui::Sense::click())).clicked() {
        sort.toggle(col);
    }
}
