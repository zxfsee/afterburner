#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

#[test]
fn deployment_verification_canonical_surface_moves_past_manage_buckets() {
    let justfile = repo_file("justfile");
    let deploy_dispatch = repo_file("src/cli_dispatch/deploy.rs");

    for required in [
        "deployment-verification-receipt artifact_version profile_name verification_status verified_at_unix_ms evidence evidence_source_1 evidence_source_2:",
        "deployment-verification-receipt-reconcile receipt artifact_version profile_name verification_status verified_at_unix_ms evidence evidence_source_1 evidence_source_2 +evidence_sources:",
        "deployment-verification-bundle receipt:",
        "deployment-verification-bundle-reconcile receipt bundle:",
        "deployment-verification-handoff bundle:",
        "deployment-verification-handoff-reconcile bundle handoff:",
        "workflow-surface-check-deployment-verification:",
    ] {
        assert!(
            justfile.contains(required),
            "canonical deployment verification surface must expose `{required}`"
        );
    }

    for required in [
        "\"verification-receipt\"",
        "\"verification-bundle\"",
        "\"verification-handoff\"",
        "\"verification-receipt-locator\"",
        "\"verification-bundle-locator\"",
        "\"verification-bundle\"",
        "\"verification-handoff\"",
        "\"rollback-verification-receipt-locator\"",
    ] {
        assert!(
            deploy_dispatch.contains(required),
            "debug deploy taxonomy must keep grouped family-first dispatch `{required}`"
        );
    }

    for forbidden in [
        "deployment-verification-receipt-history receipt event recorded_at_unix_ms:",
        "deployment-verification-receipt-transport-locator receipt:",
        "deployment-verification-receipt-locator-pointer locator:",
        "deployment-verification-receipt-rollback-pointer current_pointer restored_pointer rolled_back_at_unix_ms:",
        "deployment-verification-bundle-history bundle event recorded_at_unix_ms:",
        "deployment-verification-bundle-transport-locator bundle:",
        "deployment-verification-bundle-locator-pointer locator:",
        "deployment-verification-bundle-rollback-pointer current_pointer restored_pointer rolled_back_at_unix_ms:",
        "deployment-verification-bundle-rollback-apply current_bundle restored_bundle rolled_back_at_unix_ms:",
        "deployment-verification-handoff-history handoff event recorded_at_unix_ms:",
        "deployment-verification-handoff-transport-locator handoff:",
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
