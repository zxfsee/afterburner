use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn deployment_verification_bundle_core_surface_stays_grouped() {
    let justfile = repo_file("justfile");
    for recipe in [
        "deployment-verification-bundle receipt:",
        "deployment-verification-bundle-history action +args:",
        "deployment-verification-bundle-reconcile receipt bundle:",
        "deployment-verification-bundle-transport action +args:",
        "workflow-surface-check-deployment-verification:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    for forbidden in [
        "deployment-verification-record-bundle-history bundle event recorded_at_unix_ms:",
        "deployment-verification-point-bundle-transport-locator bundle:",
        "deployment-verification-record-bundle-transport-locator-history locator event recorded_at_unix_ms:",
        "deployment-verification-reconcile-bundle-transport-locator bundle locator:",
        "deployment-verification-record-bundle-transport-locator-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-reconcile-bundle receipt bundle:",
        "deployment-verification-record-bundle-reconciliation-history reconciliation event recorded_at_unix_ms:",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "justfile must not keep per-variant bundle core recipe `{forbidden}` after grouping"
        );
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Bundle flow: `just deployment-verification-bundle`.",
        "Bundle support flows: history/reconciliation, transport, locator, and rollback stay grouped under the bundle flow.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped bundle core surface `{needle}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "`deployment_verification_evidence_bundle*.json`",
        "Bundle core history, reconciliation, and transport-locator flows stay grouped under the bundle entry surface in [docs/workflows.md](./workflows.md).",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped bundle core surface `{needle}`"
        );
    }
}
