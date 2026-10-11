use super::*;

#[test]
fn mcp_apply_without_confirm_does_not_run() {
    let report = apply_cleanup(ApplyRequest {
        confirm: false,
        root: std::env::temp_dir(),
        ids: Vec::new(),
    });
    assert!(!report.ok);
    assert_eq!(report.deleted, 0);
    assert!(
        report
            .error
            .as_deref()
            .unwrap_or("")
            .contains("confirm=true")
    );
}

#[test]
fn advise_does_not_claim_history() {
    assert_eq!(
        advise_one("history.jsonl", AiClass::History, None).label(),
        ""
    );
    assert_eq!(
        advise_one("AGENTS.md", AiClass::Context, None).label(),
        "keep"
    );
}
