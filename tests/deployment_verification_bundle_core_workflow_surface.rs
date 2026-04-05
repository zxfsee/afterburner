mod support;

use support::repo_file;

#[test]
fn deployment_verification_bundle_core_surface_stays_grouped() {
    let justfile = repo_file("justfile");
    for recipe in [
        "deployment-verification-bundle receipt:",
        "deployment-verification-bundle-reconcile receipt bundle:",
        "workflow-surface-check-deployment-verification:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    for forbidden in [
        "deployment-verification-bundle-history action +args:",
        "deployment-verification-bundle-transport action +args:",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "justfile must not keep per-variant bundle core recipe `{forbidden}` after grouping"
        );
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Bundle flow: `just deployment-verification-bundle`, `just deployment-verification-bundle-reconcile`.",
        "Bundle support flows: history/reconciliation, transport, locator, and rollback stay grouped under the bundle flow, but the detailed support artifacts stay behind `afterburner verify bundle ...` and `afterburner rollback verification-bundle ...`.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped bundle core surface `{needle}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "`deployment_verification_evidence_bundle*.json`",
        "Bundle support flows: history/reconciliation and transport stay grouped under the bundle operator flow in [docs/workflows.md](./workflows.md).",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped bundle core surface `{needle}`"
        );
    }
}
