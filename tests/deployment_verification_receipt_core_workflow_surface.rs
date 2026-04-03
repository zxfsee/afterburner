use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn deployment_verification_receipt_core_surface_stays_grouped() {
    let justfile = repo_file("justfile");
    for recipe in [
        "deployment-verification-receipt artifact_version profile_name verification_status verified_at_unix_ms evidence evidence_source_1 evidence_source_2:",
        "deployment-verification-receipt-manage action +args:",
        "workflow-surface-check-deployment-verification:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    for forbidden in [
        "deployment-verification-point-receipt-transport-locator receipt:",
        "deployment-verification-record-receipt-transport-locator-history locator event recorded_at_unix_ms:",
        "deployment-verification-reconcile-receipt-transport-locator receipt locator:",
        "deployment-verification-record-receipt-transport-locator-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-record-receipt-history receipt event recorded_at_unix_ms:",
        "deployment-verification-reconcile-receipt receipt artifact_version profile_name verification_status verified_at_unix_ms evidence evidence_source_1 evidence_source_2:",
        "deployment-verification-record-receipt-reconciliation-history reconciliation event recorded_at_unix_ms:",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "justfile must not keep per-variant receipt core recipe `{forbidden}` after grouping"
        );
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Receipt family: `just deployment-verification-receipt` anchors `deployment_verification_receipt*.json`.",
        "Core receipt sidecars: `just deployment-verification-receipt-manage <record-history|reconcile|point-transport-locator|record-transport-locator-history|reconcile-transport-locator|record-transport-locator-reconciliation-history>` covers the receipt history, reconciliation, and transport-locator artifacts.",
        "Receipt locator and rollback variants remain explicit until the dedicated locator-and-rollback grouping lands.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped receipt core surface `{needle}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "`deployment_verification_receipt*.json`",
        "Receipt core history, reconciliation, and transport-locator flows stay grouped under the receipt entry surface in [docs/workflows.md](./workflows.md); receipt locator and rollback variants remain explicit until their follow-on grouping lands.",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped receipt core surface `{needle}`"
        );
    }
}
