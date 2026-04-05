#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

#[test]
fn deployment_verification_handoff_surface_stays_grouped() {
    let justfile = repo_file("justfile");
    for recipe in [
        "deployment-verification-handoff bundle:",
        "deployment-verification-handoff-reconcile bundle handoff:",
        "workflow-surface-check-deployment-verification:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    for forbidden in [
        "deployment-verification-handoff-history action +args:",
        "deployment-verification-handoff-transport action +args:",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "justfile must not keep per-variant handoff recipe `{forbidden}` after grouping"
        );
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Handoff flow: `just deployment-verification-handoff`, `just deployment-verification-handoff-reconcile`.",
        "Handoff support flows: history/reconciliation and transport stay grouped under the handoff flow, but the detailed support artifacts stay behind `afterburner verify handoff ...`.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped handoff surface `{needle}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "`deployment_verification_evidence_handoff*.json`",
        "Handoff support flows: history/reconciliation and transport stay grouped under the handoff operator flow in [docs/workflows.md](./workflows.md).",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped handoff surface `{needle}`"
        );
    }
}
