use std::fs;
use std::path::PathBuf;

mod support;

use support::repo_file;

#[test]
fn deployment_verification_receipt_locator_and_rollback_surface_stays_grouped() {
    let justfile = repo_file("justfile");
    for recipe in [
        "deployment-verification-receipt-locator-pointer locator:",
        "deployment-verification-receipt-locator-history pointer event recorded_at_unix_ms:",
        "deployment-verification-receipt-locator-reconcile locator pointer:",
        "deployment-verification-receipt-locator-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-receipt-rollback-pointer current_pointer restored_pointer rolled_back_at_unix_ms:",
        "deployment-verification-receipt-rollback-pointer-history rollback event recorded_at_unix_ms:",
        "deployment-verification-receipt-rollback-reconcile rollback current_rollback:",
        "deployment-verification-receipt-rollback-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-receipt-rollback-supersede previous_rollback next_rollback superseded_at_unix_ms:",
        "deployment-verification-receipt-rollback-supersession-history supersession event recorded_at_unix_ms:",
        "deployment-verification-receipt-rollback-supersession-reconcile supersession current_supersession:",
        "deployment-verification-receipt-rollback-supersession-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "workflow-surface-check-deployment-verification:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    for forbidden in [
        "deployment-verification-receipt-locator action +args:",
        "deployment-verification-receipt-rollback action +args:",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "justfile must not keep per-variant receipt locator/rollback recipe `{forbidden}` after grouping"
        );
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Receipt support flows: history/reconciliation, transport, locator, and rollback stay grouped under the receipt flow.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped receipt locator/rollback surface `{needle}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "`deployment_verification_receipt*.json`",
        "Receipt support flows: locator, rollback, and rollback supersession stay grouped under the receipt operator flow in [docs/workflows.md](./workflows.md).",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped receipt locator/rollback surface `{needle}`"
        );
    }
}
