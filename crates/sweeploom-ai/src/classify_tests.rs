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
