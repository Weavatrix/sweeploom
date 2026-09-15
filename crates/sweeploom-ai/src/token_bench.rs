//! Always-on prompt tax vs the old "History + full bytes/4" overcount.

use crate::classify::{
    AiClass, classify_name, estimated_prompt_tokens, estimated_prompt_tokens_with_files,
};
use crate::gold::{GOLD_HISTORY_AS_PROMPT, GOLD_STORE, GOLD_WITH};

/// Old SweepLoom tax: History counted, no cap, every byte / 4.
fn legacy_tax(class: AiClass, bytes: u64) -> u64 {
    if matches!(class, AiClass::Context | AiClass::History) {
        bytes / 4
    } else {
        0
    }
}

#[test]
fn bench_token_context_weight() {
    let mut legacy_visible = 0_u64;
    let mut now_visible = 0_u64;
    let mut history_dropped = 0_u64;
    let mut still_hidden = 0_u32;
    let mut false_clean_context = 0_u32;

    eprintln!("--- always-on token tax (history is archive, dirs capped) ---");
    for leaf in GOLD_STORE {
        let now = classify_name(leaf.name);
        let old = legacy_tax(leaf.class, leaf.bytes);
        let new = estimated_prompt_tokens_with_files(now, leaf.name, leaf.bytes, leaf.files);
        legacy_visible += old;
        now_visible += new;
        if leaf.class == AiClass::History {
            history_dropped += old;
        }
        if leaf.class.is_prompt_context() && !now.is_prompt_context() {
            still_hidden += 1;
        }
        if leaf.class.is_prompt_context() && now.can_clean() {
            false_clean_context += 1;
        }
        eprintln!(
            "  {:<22} {:>10} B  files={:<4}  class={now:?}  old={old} tok  now={new} tok",
            leaf.name, leaf.bytes, leaf.files
        );
        assert_eq!(now, leaf.class, "{}", leaf.name);
    }

    eprintln!(
        "SUM always-on tokens: old(overcount)={legacy_visible}  now={now_visible}  history_dropped={history_dropped}"
    );
    eprintln!("false-clean on Context={false_clean_context}  still_hidden={still_hidden}");

    assert_eq!(false_clean_context, 0);
    assert_eq!(still_hidden, 0);
    assert_eq!(now_visible, GOLD_WITH);
    assert_eq!(legacy_visible, GOLD_HISTORY_AS_PROMPT);
    assert_eq!(
        estimated_prompt_tokens(AiClass::History, "history.jsonl", 2_000_000),
        0
    );
    assert_eq!(
        estimated_prompt_tokens(AiClass::Cache, "cache", 900_000_000),
        0
    );
    assert_eq!(
        estimated_prompt_tokens(AiClass::Secret, ".credentials.json", 2_000),
        0
    );
    assert!(now_visible <= 8_000 + 1_000 + 4_096 + 4_096 + 4_096);
    assert!(now_visible >= 3_000 + 1_000);
}
