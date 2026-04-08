#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

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

#[test]
fn deployment_verification_bundle_locator_and_rollback_surface_stays_grouped() {
    let justfile = repo_file("justfile");
    for recipe in [
        "deployment-verification-bundle receipt:",
        "deployment-verification-bundle-reconcile receipt bundle:",
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
    let needle = "Bundle support flows: history/reconciliation, transport, locator, and rollback stay grouped under the bundle flow, but the detailed support artifacts stay behind `afterburner verify bundle ...` and `afterburner rollback verification-bundle ...`.";
    assert!(
        workflows.contains(needle),
        "workflow reference must keep the grouped bundle locator/rollback surface `{needle}`"
    );

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

#[test]
fn deployment_verification_receipt_locator_and_rollback_surface_stays_grouped() {
    let justfile = repo_file("justfile");
    for recipe in [
        "deployment-verification-receipt artifact_version profile_name verification_status verified_at_unix_ms evidence evidence_source_1 evidence_source_2:",
        "deployment-verification-receipt-reconcile receipt artifact_version profile_name verification_status verified_at_unix_ms evidence evidence_source_1 evidence_source_2 +evidence_sources:",
        "workflow-surface-check-deployment-verification:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    for forbidden in [
        "deployment-verification-receipt-locator action +args:",
        "deployment-verification-receipt-rollback action +args:",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "justfile must not keep per-variant receipt locator/rollback recipe `{forbidden}` after grouping"
        );
    }

    let workflows = repo_file("docs/workflows.md");
    let needle = "Receipt support flows: history/reconciliation, transport, locator, and rollback stay grouped under the receipt flow, but the detailed support artifacts stay behind `afterburner verify receipt ...` and `afterburner rollback verification-receipt ...`.";
    assert!(
        workflows.contains(needle),
        "workflow reference must keep the grouped receipt locator/rollback surface `{needle}`"
    );

    let reference = repo_file("docs/reference.md");
    for needle in [
        "`deployment_verification_receipt*.json`",
        "Receipt support flows: locator, rollback, and rollback supersession stay grouped under the receipt operator flow in [docs/workflows.md](./workflows.md).",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped receipt locator/rollback surface `{needle}`"
        );
    }
}

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
