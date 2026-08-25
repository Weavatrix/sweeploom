//! User intent. The cleaner learns through policy, not an LLM.

/// Per-candidate or per-project user policy.
#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UserPolicy {
    /// Use the default planner.
    #[default]
    Default,
    /// Keep this item.
    Keep,
    /// Never suggest this item.
    NeverClean,
    /// Always clean when cold.
    AlwaysCleanWhenCold,
    /// Ask every time.
    AskEveryTime,
    /// Pin the owning project.
    PinProject,
}
