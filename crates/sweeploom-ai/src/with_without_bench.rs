//! How an agent copes **without** SweepLoom vs **with** it.

use crate::classify::{
    AiClass, ContextAdvice, advise_context, classify_name, estimated_prompt_tokens_with_files,
    leaf_name, naive_is_secret,
};
use crate::gold::{GOLD_DUMP, GOLD_HISTORY_AS_PROMPT, GOLD_STORE, GOLD_WITH};

/// Substring classifier an agent improvises without SweepLoom.
///
/// Shares the naive Secret rule with `classify_legacy`. Cache / history stay
/// thinner on purpose: no sqlite, log, settings, or archived_sessions.
pub(crate) fn classify_without(name: &str) -> AiClass {
    let leaf = leaf_name(name);
    let leaf = leaf.as_str();
    if naive_is_secret(leaf) {
        AiClass::Secret
    } else if leaf.contains("cache") {
        AiClass::Cache
    } else if leaf.ends_with(".jsonl") || matches!(leaf, "history" | "projects" | "sessions") {
        AiClass::History
    } else {
        AiClass::Other
    }
}

#[test]
fn bench_ai_with_and_without_sweeploom() {
    let mut dump = 0_u64;
    let mut history_as_prompt = 0_u64;
    let mut with = 0_u64;
    let mut without_wrong_class = 0_u32;
    let mut without_would_clean_context = 0_u32;
    let mut without_false_secret = 0_u32;
    let mut with_wrong_class = 0_u32;

    eprintln!("=== AI WITHOUT SweepLoom vs WITH SweepLoom (token + class) ===");
    eprintln!(
        "  {:<22} {:>12} {:>12} {:>12}  without-class  with-class",
        "leaf", "dump/4", "hist+ctx/4", "with"
    );
    for leaf in GOLD_STORE {
        let dumped = leaf.bytes / 4;
        let naive = if matches!(leaf.class, AiClass::Context | AiClass::History) {
            leaf.bytes / 4
        } else {
            0
        };
        let now = classify_name(leaf.name);
        let taxed = estimated_prompt_tokens_with_files(now, leaf.name, leaf.bytes, leaf.files);
        let guessed = classify_without(leaf.name);
        dump += dumped;
        history_as_prompt += naive;
        with += taxed;
        if guessed != leaf.class {
            without_wrong_class += 1;
        }
        if now != leaf.class {
            with_wrong_class += 1;
        }
        if leaf.class.is_prompt_context()
            && matches!(guessed, AiClass::Other | AiClass::Cache | AiClass::Log)
        {
            without_would_clean_context += 1;
        }
        if leaf.class == AiClass::Other && guessed == AiClass::Secret {
            without_false_secret += 1;
        }
        eprintln!(
            "  {:<22} {dumped:>12} {naive:>12} {taxed:>12}  {guessed:?}  {now:?}",
            leaf.name
        );
    }

    let agents_md = advise_context("AGENTS.md", AiClass::Context, None);
    let stale_skills = advise_context(
        "skills",
        AiClass::Context,
        Some(std::time::Duration::from_secs(40 * 24 * 3600)),
    );

    eprintln!("--- sums ---");
    eprintln!("WITHOUT (dump whole store /4):     {dump}");
    eprintln!("WITHOUT (History billed as ctx):   {history_as_prompt}");
    eprintln!("WITH    (SweepLoom always-on):     {with}");
    eprintln!(
        "WITHOUT wrong class={without_wrong_class}  false-secret={without_false_secret}  would-clean-Context={without_would_clean_context}"
    );
    eprintln!("WITH    wrong class={with_wrong_class}");
    eprintln!(
        "WITH    AGENTS.md advice={:?}  stale skills={:?}",
        agents_md, stale_skills
    );

    assert_eq!(with_wrong_class, 0);
    assert_eq!(
        without_would_clean_context, 5,
        "without SweepLoom, AGENTS.md / mdc / skills / plugins / rules look like Other junk"
    );
    assert!(
        without_wrong_class > 0,
        "without SweepLoom must miss Context"
    );
    assert!(
        without_false_secret > 0,
        "without SweepLoom password-reset.md is Secret"
    );
    assert_eq!(dump, GOLD_DUMP);
    assert_eq!(history_as_prompt, GOLD_HISTORY_AS_PROMPT);
    assert_eq!(with, GOLD_WITH);
    assert_eq!(agents_md, ContextAdvice::Keep);
    assert_eq!(stale_skills, ContextAdvice::SuggestPark);
    assert_ne!(classify_without("AGENTS.md"), AiClass::Context);
    assert_eq!(classify_without("password-reset.md"), AiClass::Secret);
}
