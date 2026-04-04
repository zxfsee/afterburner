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
        "deployment-verification-receipt-history action +args:",
        "deployment-verification-receipt-reconcile receipt artifact_version profile_name verification_status verified_at_unix_ms evidence evidence_source_1 evidence_source_2 +evidence_sources:",
        "deployment-verification-receipt-transport action +args:",
        "deployment-verification-receipt-locator action +args:",
        "deployment-verification-receipt-rollback action +args:",
        "deployment-verification-bundle-history action +args:",
        "deployment-verification-bundle-reconcile receipt bundle:",
        "deployment-verification-bundle-transport action +args:",
        "deployment-verification-bundle-locator action +args:",
        "deployment-verification-bundle-rollback action +args:",
        "deployment-verification-handoff-history action +args:",
        "deployment-verification-handoff-reconcile bundle handoff:",
        "deployment-verification-handoff-transport action +args:",
    ] {
        assert!(
            justfile.contains(required),
            "canonical deployment verification surface must expose `{required}`"
        );
    }

    for forbidden in [
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
