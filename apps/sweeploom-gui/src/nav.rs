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

    /// Short label.
    #[must_use]
    pub const fn label(self) -> &'static str {
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
