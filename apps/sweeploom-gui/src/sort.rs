//! Clickable column sort used by Sessions, Review, and Explorer.

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

    /// Header caption with an arrow when this column is active.
    #[must_use]
    pub fn caption(self, col: Col, name: &str) -> String {
        if self.col != col {
            name.to_owned()
        } else if self.desc {
            format!("{name}  ↓")
        } else {
            format!("{name}  ↑")
        }
    }
}

/// Draw a header button. Returns true when the sort changed.
pub fn header_button(ui: &mut egui::Ui, sort: &mut Sort, col: Col, name: &str) -> bool {
    let text = RichText::new(sort.caption(col, name)).strong();
    if ui.button(text).clicked() {
        sort.toggle(col);
        true
    } else {
        false
    }
}
