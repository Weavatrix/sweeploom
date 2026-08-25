//! First-class navigation. Sessions are not buried in Settings.

/// Primary navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nav {
    /// Machine pressure cards.
    Overview,
    /// Disk candidates.
    Storage,
    /// Logical sessions.
    Sessions,
    /// Project heat.
    Projects,
    /// Browser companion.
    Browser,
    /// Folder inspector.
    Explorer,
    /// AI storage.
    Ai,
    /// Declarative rules.
    Rules,
    /// Observed history.
    History,
    /// Local settings.
    Settings,
}

impl Nav {
    /// Sidebar order.
    pub const ALL: [Self; 10] = [
        Self::Overview,
        Self::Sessions,
        Self::History,
        Self::Storage,
        Self::Explorer,
        Self::Projects,
        Self::Browser,
        Self::Ai,
        Self::Rules,
        Self::Settings,
    ];

    /// Short label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Storage => "Review",
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

    /// Sidebar section heading.
    #[must_use]
    pub const fn section(self) -> &'static str {
        match self {
            Self::Overview | Self::Sessions | Self::History => "LIVE",
            Self::Storage | Self::Explorer | Self::Projects => "DISK",
            Self::Browser | Self::Ai | Self::Rules => "WORKSPACE",
            Self::Settings => "APP",
        }
    }

    /// Sidebar glyph.
    #[must_use]
    pub const fn glyph(self) -> crate::icons::Glyph {
        match self {
            Self::Overview => crate::icons::Glyph::Overview,
            Self::Storage => crate::icons::Glyph::Review,
            Self::Sessions => crate::icons::Glyph::Sessions,
            Self::Projects => crate::icons::Glyph::Projects,
            Self::Browser => crate::icons::Glyph::Browser,
            Self::Explorer => crate::icons::Glyph::Explorer,
            Self::Ai => crate::icons::Glyph::Ai,
            Self::Rules => crate::icons::Glyph::Rules,
            Self::History => crate::icons::Glyph::History,
            Self::Settings => crate::icons::Glyph::Settings,
        }
    }
}
