use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn deployment_verification_receipt_locator_and_rollback_surface_stays_grouped() {
    let justfile = repo_file("justfile");
    for recipe in [
        "deployment-verification-receipt-locator action +args:",
        "deployment-verification-receipt-rollback action +args:",
        "workflow-surface-check-deployment-verification:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    for forbidden in [
        "deployment-verification-record-receipt-locator-history pointer event recorded_at_unix_ms:",
        "deployment-verification-reconcile-receipt-locator locator pointer:",
        "deployment-verification-record-receipt-locator-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-rollback-receipt-locator current_pointer restored_pointer rolled_back_at_unix_ms:",
        "deployment-verification-record-receipt-locator-rollback-history rollback event recorded_at_unix_ms:",
        "deployment-verification-reconcile-receipt-rollback rollback current_rollback:",
        "deployment-verification-record-receipt-rollback-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-supersede-receipt-rollback previous_rollback next_rollback superseded_at_unix_ms:",
        "deployment-verification-reconcile-receipt-rollback-supersession supersession current_supersession:",
        "deployment-verification-record-receipt-rollback-supersession-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-record-receipt-rollback-supersession-history supersession event recorded_at_unix_ms:",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "justfile must not keep per-variant receipt locator/rollback recipe `{forbidden}` after grouping"
        );
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Receipt support flows: history/reconciliation, transport, locator, and rollback stay grouped under the receipt family entrypoints.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped receipt locator/rollback surface `{needle}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "`deployment_verification_receipt*.json`",
        "Receipt locator, locator rollback, and rollback supersession flows stay grouped under the receipt entry surface in [docs/workflows.md](./workflows.md).",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped receipt locator/rollback surface `{needle}`"
        );
    }
}
