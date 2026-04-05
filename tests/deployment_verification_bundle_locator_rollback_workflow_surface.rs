use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn deployment_verification_bundle_locator_and_rollback_surface_stays_grouped() {
    let justfile = repo_file("justfile");
    for recipe in [
        "deployment-verification-bundle-locator-pointer locator:",
        "deployment-verification-bundle-locator-history pointer event recorded_at_unix_ms:",
        "deployment-verification-bundle-locator-reconcile locator pointer:",
        "deployment-verification-bundle-locator-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-bundle-rollback-locator current_pointer restored_pointer rolled_back_at_unix_ms:",
        "deployment-verification-bundle-rollback-locator-history rollback event recorded_at_unix_ms:",
        "deployment-verification-bundle-rollback-apply current_bundle restored_bundle rolled_back_at_unix_ms:",
        "deployment-verification-bundle-rollback-apply-history rollback event recorded_at_unix_ms:",
        "deployment-verification-bundle-rollback-apply-reconcile rollback current_rollback:",
        "deployment-verification-bundle-rollback-apply-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-bundle-rollback-apply-supersede previous_rollback next_rollback superseded_at_unix_ms:",
        "deployment-verification-bundle-rollback-apply-supersession-history supersession event recorded_at_unix_ms:",
        "deployment-verification-bundle-rollback-apply-supersession-reconcile supersession current_supersession:",
        "deployment-verification-bundle-rollback-apply-supersession-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "workflow-surface-check-deployment-verification:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    for forbidden in [
        "deployment-verification-bundle-locator action +args:",
        "deployment-verification-bundle-rollback action +args:",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "justfile must not keep per-variant bundle locator/rollback recipe `{forbidden}` after grouping"
        );
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Bundle support flows: history/reconciliation, transport, locator, and rollback stay grouped under the bundle flow.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped bundle locator/rollback surface `{needle}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "`deployment_verification_evidence_bundle*.json`",
        "Bundle support flows: locator, rollback, and rollback supersession stay grouped under the bundle operator flow in [docs/workflows.md](./workflows.md).",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped bundle locator/rollback surface `{needle}`"
        );
    }
}
