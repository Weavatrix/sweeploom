//! Classify AI store names. Never opens file contents.

use std::time::Duration;

/// Kind of an AI-store child. Used to decide inspect vs explicit cache clean.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AiClass {
    /// Tokens, credentials, auth material.
    Secret,
    /// SQLite / DB files. Internals are never opened.
    Sqlite,
    /// Regenerable cache.
    Cache,
    /// Debug / log / last-run residue.
    Log,
    /// Session transcripts and history. Search-before-delete, not this slice.
    History,
    /// Tool settings.
    Settings,
    /// Always-on agent context (rules, skills, AGENTS.md). Inspect-only.
    Context,
    /// Everything else.
    Other,
}

/// What SweepLoom may *say* about always-on context. Never writes files.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContextAdvice {
    /// Project contract or still-warm context. Leave it loaded.
    Keep,
    /// Stale skill/plugin index. User reviews; SweepLoom does not disable it.
    SuggestPark,
    /// Not always-on prompt context.
    LeaveAlone,
}

impl ContextAdvice {
    /// CLI / hover label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Keep => "keep",
            Self::SuggestPark => "park?",
            Self::LeaveAlone => "",
        }
    }
}

impl AiClass {
    /// Toolbar / table label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Secret => "Secret",
            Self::Sqlite => "SQLite",
            Self::Cache => "Cache",
            Self::Log => "Log",
            Self::History => "History",
            Self::Settings => "Settings",
            Self::Context => "Context",
            Self::Other => "Other",
        }
    }

    /// Regenerable residue the user may trash one row at a time.
    #[must_use]
    pub const fn can_clean(self) -> bool {
        matches!(self, Self::Cache | Self::Log)
    }

    /// Always-on prompt files the host may inject every turn. History is archive.
    #[must_use]
    pub const fn is_prompt_context(self) -> bool {
        matches!(self, Self::Context)
    }
}

/// Per always-on file. Hosts truncate long rules; do not bill the whole file.
const ALWAYS_ON_FILE_CAP: u64 = 8_000;
/// Typical skill/plugin *description*, not the implementation tree.
const DESCRIPTION_TOKENS: u64 = 256;
/// Whole `skills` / `plugins` / `rules` directory: index tax, not payload.
const CONTEXT_DIR_CAP: u64 = 4_096;
/// Skill/plugin trees older than this may be parked. AGENTS.md / rules never are.
const PARK_AFTER: Duration = Duration::from_secs(30 * 24 * 3600);

/// Always-on prompt tokens from metadata. History/cache/secrets are 0.
///
/// Directory rows are capped: the host loads short descriptions, not the tree.
#[must_use]
pub fn estimated_prompt_tokens(class: AiClass, relative: &str, logical_bytes: u64) -> u64 {
    estimated_prompt_tokens_with_files(class, relative, logical_bytes, 1)
}

/// Same as [`estimated_prompt_tokens`], using the child file count for dir caps.
#[must_use]
pub fn estimated_prompt_tokens_with_files(
    class: AiClass,
    relative: &str,
    logical_bytes: u64,
    file_count: u64,
) -> u64 {
    if !class.is_prompt_context() {
        return 0;
    }
    let leaf = leaf_name(relative);
    let raw = logical_bytes / 4;
    if matches!(leaf.as_str(), "skills" | "plugins" | "rules") {
        let by_desc = file_count.max(1).saturating_mul(DESCRIPTION_TOKENS);
        return raw.min(by_desc).min(CONTEXT_DIR_CAP);
    }
    raw.min(ALWAYS_ON_FILE_CAP)
}

/// Advise whether always-on context is still earning its token tax.
///
/// Never flips `alwaysApply`, never deletes, never opens file contents.
/// `AGENTS.md`, `*.mdc`, and `rules` stay Keep — those are the project contract.
/// Only stale `skills` / `plugins` trees get SuggestPark.
#[must_use]
pub fn advise_context(relative: &str, class: AiClass, idle: Option<Duration>) -> ContextAdvice {
    if !class.is_prompt_context() {
        return ContextAdvice::LeaveAlone;
    }
    let leaf = leaf_name(relative);
    if leaf == "agents.md" || leaf.ends_with(".mdc") || leaf == "rules" {
        return ContextAdvice::Keep;
    }
    if matches!(leaf.as_str(), "skills" | "plugins") && idle.is_some_and(|age| age >= PARK_AFTER) {
        return ContextAdvice::SuggestPark;
    }
    ContextAdvice::Keep
}

/// Classify a relative path or file name. Metadata only.
#[must_use]
pub fn classify_name(name: &str) -> AiClass {
    let leaf = leaf_name(name);
    let leaf = leaf.as_str();
    if is_secret(leaf) {
        AiClass::Secret
    } else if is_sqlite(leaf) {
        AiClass::Sqlite
    } else if is_cache(leaf) {
        AiClass::Cache
    } else if is_log(leaf) {
        AiClass::Log
    } else if is_context(leaf) {
        AiClass::Context
    } else if is_history(leaf) {
        AiClass::History
    } else if is_settings(leaf) {
        AiClass::Settings
    } else {
        AiClass::Other
    }
}

fn is_secret(leaf: &str) -> bool {
    if leaf.contains("credential")
        || leaf.contains("secret")
        || leaf.contains(".auth")
        || leaf == "auth.json"
    {
        return true;
    }
    if has_segment(leaf, "token") {
        return !leaf.ends_with(".md");
    }
    if has_segment(leaf, "password") {
        return leaf.ends_with(".json")
            || leaf.ends_with(".env")
            || leaf.ends_with(".key")
            || leaf == "password";
    }
    false
}

pub(crate) fn is_sqlite(leaf: &str) -> bool {
    leaf.ends_with(".sqlite")
        || leaf.ends_with(".sqlite3")
        || leaf.ends_with(".db")
        || leaf.ends_with(".db-wal")
        || leaf.ends_with(".db-shm")
        || leaf.ends_with(".vscdb")
        || leaf.contains(".vscdb")
}

fn is_cache(leaf: &str) -> bool {
    matches!(
        leaf,
        "cache"
            | "caches"
            | "cacheddata"
            | "tmp"
            | "temp"
            | "statsig"
            | "mcp-needs-auth-cache.json"
    ) || has_segment(leaf, "cache")
        || has_segment(leaf, "caches")
        || has_segment(leaf, "cacheddata")
}

fn has_segment(leaf: &str, needle: &str) -> bool {
    leaf.split(|ch: char| !ch.is_ascii_alphanumeric())
        .any(|part| part == needle)
}

pub(crate) fn is_log(leaf: &str) -> bool {
    matches!(
        leaf,
        "log" | "logs" | "debug" | ".last-cleanup" | ".last-update-result.json"
    ) || leaf.ends_with(".log")
}

pub(crate) fn is_history(leaf: &str) -> bool {
    matches!(
        leaf,
        "history" | "history.jsonl" | "sessions" | "archived_sessions" | "transcripts" | "projects"
    ) || leaf.ends_with(".jsonl")
}

fn is_context(leaf: &str) -> bool {
    matches!(leaf, "agents.md" | "skills" | "plugins" | "rules") || leaf.ends_with(".mdc")
}

fn is_settings(leaf: &str) -> bool {
    leaf.contains("settings")
        || leaf.contains("config")
        || matches!(leaf, "argv.json" | "mcp.json" | ".mcp.json")
}

pub(crate) fn leaf_name(name: &str) -> String {
    name.replace('\\', "/")
        .rsplit('/')
        .next()
        .unwrap_or(name)
        .to_ascii_lowercase()
}

/// Old substring Secret rule copied by both bench baselines.
///
/// Production [`is_secret`] is tighter (`token` / `password` are segmented).
#[cfg(test)]
pub(crate) fn naive_is_secret(leaf: &str) -> bool {
    leaf.contains("credential")
        || leaf.contains("secret")
        || leaf.contains("token")
        || leaf.contains("password")
        || leaf.contains(".auth")
        || leaf == "auth.json"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn secrets_and_sqlite_cannot_clean() {
        assert_eq!(classify_name(".credentials.json"), AiClass::Secret);
        assert_eq!(classify_name("state.vscdb"), AiClass::Sqlite);
        assert_eq!(classify_name("foo.sqlite"), AiClass::Sqlite);
        assert!(!classify_name(".credentials.json").can_clean());
        assert!(!classify_name("foo.sqlite").can_clean());
    }

    #[test]
    fn caches_and_logs_can_clean() {
        assert_eq!(classify_name("cache"), AiClass::Cache);
        assert_eq!(classify_name("mcp-needs-auth-cache.json"), AiClass::Cache);
        assert_eq!(classify_name("debug"), AiClass::Log);
        assert!(classify_name("cache").can_clean());
        assert!(classify_name(".last-cleanup").can_clean());
    }

    #[test]
    fn history_and_settings_stay_inspect() {
        assert_eq!(classify_name("history.jsonl"), AiClass::History);
        assert_eq!(classify_name("projects"), AiClass::History);
        assert_eq!(classify_name("archived_sessions"), AiClass::History);
        assert_eq!(classify_name("remote-settings.json"), AiClass::Settings);
        assert!(!AiClass::History.can_clean());
        assert!(!AiClass::Settings.can_clean());
    }

    #[test]
    fn docs_are_not_secrets_and_cachet_is_not_cache() {
        assert_eq!(classify_name("password-reset.md"), AiClass::Other);
        assert_eq!(classify_name("token.txt"), AiClass::Secret);
        assert_eq!(classify_name("password.json"), AiClass::Secret);
        assert_eq!(classify_name("CachedData"), AiClass::Cache);
        assert_eq!(classify_name("cachet.json"), AiClass::Other);
        assert_eq!(classify_name("AGENTS.md"), AiClass::Context);
        assert_eq!(classify_name("always-on.mdc"), AiClass::Context);
        assert_eq!(classify_name("skills"), AiClass::Context);
        assert!(!AiClass::Context.can_clean());
        assert!(AiClass::Context.is_prompt_context());
        assert!(!AiClass::History.is_prompt_context());
        assert_eq!(
            estimated_prompt_tokens(AiClass::Context, "AGENTS.md", 4_000),
            1_000
        );
        assert_eq!(
            estimated_prompt_tokens(AiClass::History, "history.jsonl", 2_000_000),
            0
        );
        assert_eq!(
            estimated_prompt_tokens_with_files(AiClass::Context, "skills", 80_000, 200),
            CONTEXT_DIR_CAP
        );
        assert_eq!(classify_name("mcp.json"), AiClass::Settings);
        assert_eq!(
            advise_context("AGENTS.md", AiClass::Context, Some(PARK_AFTER)),
            ContextAdvice::Keep
        );
        assert_eq!(
            advise_context("always-on.mdc", AiClass::Context, Some(PARK_AFTER)),
            ContextAdvice::Keep
        );
        assert_eq!(
            advise_context("rules", AiClass::Context, Some(PARK_AFTER)),
            ContextAdvice::Keep
        );
        assert_eq!(
            advise_context(
                "skills",
                AiClass::Context,
                Some(Duration::from_secs(3 * 24 * 3600))
            ),
            ContextAdvice::Keep
        );
        assert_eq!(
            advise_context("skills", AiClass::Context, Some(PARK_AFTER)),
            ContextAdvice::SuggestPark
        );
        assert_eq!(
            advise_context("plugins", AiClass::Context, Some(PARK_AFTER)),
            ContextAdvice::SuggestPark
        );
        assert_eq!(
            advise_context("history.jsonl", AiClass::History, Some(PARK_AFTER)),
            ContextAdvice::LeaveAlone
        );
    }

    #[test]
    fn naive_secret_is_the_old_substring_rule_not_production() {
        for name in [
            ".credentials.json",
            "auth.json",
            "token.txt",
            "password.json",
            "password-reset.md",
        ] {
            assert!(
                naive_is_secret(&leaf_name(name)),
                "{name} must stay Secret on the shared bench rule"
            );
        }
        assert!(!naive_is_secret(&leaf_name("AGENTS.md")));
        assert!(!naive_is_secret(&leaf_name("cache")));
        assert_eq!(classify_name("password-reset.md"), AiClass::Other);
        assert_eq!(classify_name("token.txt"), AiClass::Secret);
    }
}
