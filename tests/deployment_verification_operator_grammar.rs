use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn deployment_verification_canonical_surface_moves_past_manage_buckets() {
    let justfile = repo_file("justfile");

    for required in [
        "deployment-verification-receipt-history receipt event recorded_at_unix_ms:",
        "deployment-verification-receipt-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-receipt-reconcile receipt artifact_version profile_name verification_status verified_at_unix_ms evidence evidence_source_1 evidence_source_2 +evidence_sources:",
        "deployment-verification-receipt-transport-locator receipt:",
        "deployment-verification-receipt-transport-locator-history locator event recorded_at_unix_ms:",
        "deployment-verification-receipt-transport-locator-reconcile receipt locator:",
        "deployment-verification-receipt-transport-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-receipt-locator-pointer locator:",
        "deployment-verification-receipt-locator-history pointer event recorded_at_unix_ms:",
        "deployment-verification-receipt-locator-reconcile locator pointer:",
        "deployment-verification-receipt-locator-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-receipt-rollback-locator current_pointer restored_pointer rolled_back_at_unix_ms:",
        "deployment-verification-receipt-rollback-locator-history rollback event recorded_at_unix_ms:",
        "deployment-verification-receipt-rollback-reconcile rollback current_rollback:",
        "deployment-verification-receipt-rollback-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-receipt-rollback-supersede previous_rollback next_rollback superseded_at_unix_ms:",
        "deployment-verification-receipt-rollback-supersession-history supersession event recorded_at_unix_ms:",
        "deployment-verification-receipt-rollback-supersession-reconcile supersession current_supersession:",
        "deployment-verification-receipt-rollback-supersession-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-bundle-history bundle event recorded_at_unix_ms:",
        "deployment-verification-bundle-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-bundle-reconcile receipt bundle:",
        "deployment-verification-bundle-transport-locator bundle:",
        "deployment-verification-bundle-transport-locator-history locator event recorded_at_unix_ms:",
        "deployment-verification-bundle-transport-locator-reconcile bundle locator:",
        "deployment-verification-bundle-transport-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-bundle-locator-pointer locator:",
        "deployment-verification-bundle-locator-history pointer event recorded_at_unix_ms:",
        "deployment-verification-bundle-locator-reconcile locator pointer:",
        "deployment-verification-bundle-locator-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-bundle-rollback-locator current_pointer restored_pointer rolled_back_at_unix_ms:",
        "deployment-verification-bundle-rollback-locator-history rollback event recorded_at_unix_ms:",
        "deployment-verification-bundle-rollback-apply current_bundle restored_bundle rolled_back_at_unix_ms:",
        "deployment-verification-bundle-rollback-history rollback event recorded_at_unix_ms:",
        "deployment-verification-bundle-rollback-reconcile rollback current_rollback:",
        "deployment-verification-bundle-rollback-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-bundle-rollback-supersede previous_rollback next_rollback superseded_at_unix_ms:",
        "deployment-verification-bundle-rollback-supersession-history supersession event recorded_at_unix_ms:",
        "deployment-verification-bundle-rollback-supersession-reconcile supersession current_supersession:",
        "deployment-verification-bundle-rollback-supersession-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-handoff-history handoff event recorded_at_unix_ms:",
        "deployment-verification-handoff-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-handoff-reconcile bundle handoff:",
        "deployment-verification-handoff-transport-locator handoff:",
        "deployment-verification-handoff-transport-locator-history locator event recorded_at_unix_ms:",
        "deployment-verification-handoff-transport-locator-reconcile handoff locator:",
    ] {
        assert!(
            justfile.contains(required),
            "canonical deployment verification surface must expose `{required}`"
        );
    }

    for forbidden in [
        "deployment-verification-receipt-history action +args:",
        "deployment-verification-receipt-transport action +args:",
        "deployment-verification-receipt-locator action +args:",
        "deployment-verification-receipt-rollback action +args:",
        "deployment-verification-bundle-history action +args:",
        "deployment-verification-bundle-transport action +args:",
        "deployment-verification-bundle-locator action +args:",
        "deployment-verification-bundle-rollback action +args:",
        "deployment-verification-handoff-history action +args:",
        "deployment-verification-handoff-transport action +args:",
        "deployment-verification-receipt-manage action +args:",
        "deployment-verification-receipt-locator-manage action +args:",
        "deployment-verification-receipt-rollback-manage action +args:",
        "deployment-verification-bundle-manage action +args:",
        "deployment-verification-bundle-locator-manage action +args:",
        "deployment-verification-bundle-rollback-manage action +args:",
        "deployment-verification-handoff-manage action +args:",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "canonical deployment verification surface must not keep legacy manage bucket `{forbidden}`"
        );
    }
}
