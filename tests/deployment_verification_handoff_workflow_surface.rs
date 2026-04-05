use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn deployment_verification_handoff_surface_stays_grouped() {
    let justfile = repo_file("justfile");
    for recipe in [
        "deployment-verification-handoff bundle:",
        "deployment-verification-handoff-history action +args:",
        "deployment-verification-handoff-reconcile bundle handoff:",
        "deployment-verification-handoff-transport action +args:",
        "workflow-surface-check-deployment-verification:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    for forbidden in [
        "deployment-verification-record-handoff-history handoff event recorded_at_unix_ms:",
        "deployment-verification-point-handoff-transport-locator handoff:",
        "deployment-verification-record-handoff-transport-locator-history locator event recorded_at_unix_ms:",
        "deployment-verification-reconcile-handoff-transport-locator handoff locator:",
        "deployment-verification-reconcile-handoff bundle handoff:",
        "deployment-verification-record-handoff-reconciliation-history reconciliation event recorded_at_unix_ms:",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "justfile must not keep per-variant handoff recipe `{forbidden}` after grouping"
        );
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Handoff flow: `just deployment-verification-handoff`.",
        "Handoff support flows: history/reconciliation and transport stay grouped under the handoff flow.",
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
