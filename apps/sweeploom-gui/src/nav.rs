//! First-class navigation. Sessions are not buried in Settings.

/// Primary navigation (sidebar).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nav {
    /// Machine pressure cards.
    Overview,
    /// Logical sessions.
    Sessions,
    /// Process activity and saved disk scans.
    History,
    /// Project output and native tool storage.
    Cleanup,
    /// Project heat.
    Projects,
    /// Folder inspector.
    Explorer,
    /// Browser companion.
    Browser,
    /// AI storage.
    Ai,
    /// Local settings.
    Settings,
}

/// Where a deep link lands: a screen, or a screen plus its sub-view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dest {
    /// A screen as last left.
    Screen(Nav),
    /// Cleanup → Project output (the former Review screen).
    ProjectOutput,
    /// History → Disk scans (the former Scan history screen).
    DiskScans,
}

impl Nav {
    /// Sidebar order.
    pub const ALL: [Self; 9] = [
        Self::Overview,
        Self::Sessions,
        Self::History,
        Self::Cleanup,
        Self::Projects,
        Self::Explorer,
        Self::Browser,
        Self::Ai,
        Self::Settings,
    ];

    /// Short label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Sessions => "Sessions",
            Self::History => "History",
            Self::Cleanup => "Cleanup",
            Self::Projects => "Projects",
            Self::Explorer => "Explorer",
            Self::Browser => "Browser",
            Self::Ai => "AI",
            Self::Settings => "Settings",
        }
    }

    /// Sidebar section heading.
    #[must_use]
    pub const fn section(self) -> &'static str {
        match self {
            Self::Overview | Self::Sessions | Self::History => "LIVE",
            Self::Cleanup | Self::Projects | Self::Explorer => "DISK",
            Self::Browser | Self::Ai => "WORKSPACE",
            Self::Settings => "APP",
        }
    }

    /// Sidebar glyph.
    #[must_use]
    pub const fn glyph(self) -> crate::icons::Glyph {
        match self {
            Self::Overview => crate::icons::Glyph::Overview,
            Self::Sessions => crate::icons::Glyph::Sessions,
            Self::History => crate::icons::Glyph::History,
            Self::Cleanup => crate::icons::Glyph::Review,
            Self::Projects => crate::icons::Glyph::Projects,
            Self::Explorer => crate::icons::Glyph::Explorer,
            Self::Browser => crate::icons::Glyph::Browser,
            Self::Ai => crate::icons::Glyph::Ai,
            Self::Settings => crate::icons::Glyph::Settings,
        }
    }

    /// Screens that manage their own scrolling (tables pinned to the window).
    #[must_use]
    pub const fn owns_scroll(self) -> bool {
        !matches!(self, Self::Overview | Self::Browser | Self::Settings)
    }
}

impl Dest {
    /// Sidebar item this destination highlights.
    #[must_use]
    pub const fn nav(self) -> Nav {
        match self {
            Self::Screen(nav) => nav,
            Self::ProjectOutput => Nav::Cleanup,
            Self::DiskScans => Nav::History,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merged_screens_route_to_their_parent() {
        assert_eq!(Nav::ALL.len(), 9);
        assert_eq!(Dest::ProjectOutput.nav(), Nav::Cleanup);
        assert_eq!(Dest::DiskScans.nav(), Nav::History);
        assert_eq!(Dest::Screen(Nav::Ai).nav(), Nav::Ai);
        let sections: Vec<_> = Nav::ALL.iter().map(|nav| nav.section()).collect();
        assert_eq!(sections.first(), Some(&"LIVE"));
        assert_eq!(sections.last(), Some(&"APP"));
    }
}
