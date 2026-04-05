mod support;

use support::repo_file;

#[test]
fn deployment_verification_receipt_core_surface_stays_grouped() {
    let justfile = repo_file("justfile");
    for recipe in [
        "deployment-verification-receipt artifact_version profile_name verification_status verified_at_unix_ms evidence evidence_source_1 evidence_source_2:",
        "deployment-verification-receipt-reconcile receipt artifact_version profile_name verification_status verified_at_unix_ms evidence evidence_source_1 evidence_source_2 +evidence_sources:",
        "workflow-surface-check-deployment-verification:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    for forbidden in [
        "deployment-verification-receipt-history action +args:",
        "deployment-verification-receipt-transport action +args:",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "justfile must not keep per-variant receipt core recipe `{forbidden}` after grouping"
        );
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Receipt flow: `just deployment-verification-receipt`, `just deployment-verification-receipt-reconcile`.",
        "Receipt support flows: history/reconciliation, transport, locator, and rollback stay grouped under the receipt flow, but the detailed support artifacts stay behind `afterburner verify receipt ...` and `afterburner rollback verification-receipt ...`.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped receipt core surface `{needle}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "`deployment_verification_receipt*.json`",
        "Receipt support flows: history/reconciliation and transport stay grouped under the receipt operator flow in [docs/workflows.md](./workflows.md).",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped receipt core surface `{needle}`"
        );
    }
}
