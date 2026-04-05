use std::path::PathBuf;

mod support;

use support::repo_file;

fn fixture_exists(path: &str) -> bool {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(path)
        .exists()
}

#[test]
fn deployment_verification_family_stays_surface_complete() {
    let deploy_dispatch = repo_file("src/cli_dispatch/deploy.rs");
    for cli in [
        "verification-receipt",
        "record-verification-receipt-locator-history",
        "point-verification-receipt-locator",
        "reconcile-verification-receipt-locator",
        "record-verification-receipt-locator-reconciliation-history",
        "rollback-verification-receipt-locator",
        "record-verification-receipt-locator-rollback-history",
        "reconcile-verification-receipt-rollback",
        "record-verification-receipt-rollback-reconciliation-history",
        "supersede-verification-receipt-rollback",
        "reconcile-verification-receipt-rollback-supersession",
        "record-verification-receipt-rollback-supersession-reconciliation-history",
        "record-verification-receipt-rollback-supersession-history",
        "point-verification-receipt-transport-locator",
        "record-verification-receipt-transport-locator-history",
        "reconcile-verification-receipt-transport-locator",
        "record-verification-receipt-transport-locator-reconciliation-history",
        "record-verification-receipt-history",
        "reconcile-verification-receipt",
        "record-verification-receipt-reconciliation-history",
        "verification-bundle",
        "record-verification-bundle-history",
        "point-verification-bundle-transport-locator",
        "point-verification-bundle-locator",
        "record-verification-bundle-locator-history",
        "reconcile-verification-bundle-locator",
        "record-verification-bundle-locator-reconciliation-history",
        "rollback-verification-bundle-locator",
        "record-verification-bundle-locator-rollback-history",
        "rollback-verification-bundle",
        "record-verification-bundle-rollback-history",
        "reconcile-verification-bundle-rollback",
        "record-verification-bundle-rollback-reconciliation-history",
        "supersede-verification-bundle-rollback",
        "reconcile-verification-bundle-rollback-supersession",
        "record-verification-bundle-rollback-supersession-reconciliation-history",
        "record-verification-bundle-rollback-supersession-history",
        "record-verification-bundle-transport-locator-history",
        "reconcile-verification-bundle-transport-locator",
        "record-verification-bundle-transport-locator-reconciliation-history",
        "reconcile-verification-bundle",
        "record-verification-bundle-reconciliation-history",
        "verification-handoff",
        "record-verification-handoff-history",
        "point-verification-handoff-transport-locator",
        "record-verification-handoff-transport-locator-history",
        "reconcile-verification-handoff-transport-locator",
        "reconcile-verification-handoff",
        "record-verification-handoff-reconciliation-history",
    ] {
        assert!(
            deploy_dispatch.contains(&format!("\"{cli}\"")),
            "src/cli_dispatch/deploy.rs must expose deploy subcommand `{cli}`"
        );
    }

    let justfile = repo_file("justfile");
    for recipe in [
        "deployment-verification-receipt artifact_version profile_name verification_status verified_at_unix_ms evidence evidence_source_1 evidence_source_2:",
        "deployment-verification-receipt-reconcile receipt artifact_version profile_name verification_status verified_at_unix_ms evidence evidence_source_1 evidence_source_2 +evidence_sources:",
        "deployment-verification-bundle receipt:",
        "deployment-verification-bundle-reconcile receipt bundle:",
        "deployment-verification-handoff bundle:",
        "deployment-verification-handoff-reconcile bundle handoff:",
        "workflow-surface-check-deployment-verification:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    for fixture in [
        "deployment_verification_receipt.schema.json",
        "deployment_verification_receipt.example.json",
        "deployment_verification_receipt_locator_pointer_history.schema.json",
        "deployment_verification_receipt_locator_pointer_history.example.json",
        "deployment_verification_receipt_locator_pointer.schema.json",
        "deployment_verification_receipt_locator_pointer.example.json",
        "deployment_verification_receipt_locator_pointer_reconciliation.schema.json",
        "deployment_verification_receipt_locator_pointer_reconciliation.example.json",
        "deployment_verification_receipt_locator_pointer_reconciliation_history.schema.json",
        "deployment_verification_receipt_locator_pointer_reconciliation_history.example.json",
        "deployment_verification_receipt_locator_pointer_rollback.schema.json",
        "deployment_verification_receipt_locator_pointer_rollback.example.json",
        "deployment_verification_receipt_locator_pointer_rollback_history.schema.json",
        "deployment_verification_receipt_locator_pointer_rollback_history.example.json",
        "deployment_verification_receipt_rollback_reconciliation.schema.json",
        "deployment_verification_receipt_rollback_reconciliation.example.json",
        "deployment_verification_receipt_rollback_reconciliation_history.schema.json",
        "deployment_verification_receipt_rollback_reconciliation_history.example.json",
        "deployment_verification_receipt_rollback_supersession.schema.json",
        "deployment_verification_receipt_rollback_supersession.example.json",
        "deployment_verification_receipt_rollback_supersession_reconciliation.schema.json",
        "deployment_verification_receipt_rollback_supersession_reconciliation.example.json",
        "deployment_verification_receipt_rollback_supersession_reconciliation_history.schema.json",
        "deployment_verification_receipt_rollback_supersession_reconciliation_history.example.json",
        "deployment_verification_receipt_rollback_supersession_history.schema.json",
        "deployment_verification_receipt_rollback_supersession_history.example.json",
        "deployment_verification_receipt_transport_locator.schema.json",
        "deployment_verification_receipt_transport_locator.example.json",
        "deployment_verification_receipt_transport_locator_history.schema.json",
        "deployment_verification_receipt_transport_locator_history.example.json",
        "deployment_verification_receipt_transport_locator_reconciliation.schema.json",
        "deployment_verification_receipt_transport_locator_reconciliation.example.json",
        "deployment_verification_receipt_transport_locator_reconciliation_history.schema.json",
        "deployment_verification_receipt_transport_locator_reconciliation_history.example.json",
        "deployment_verification_receipt_history.schema.json",
        "deployment_verification_receipt_history.example.json",
        "deployment_verification_receipt_reconciliation.schema.json",
        "deployment_verification_receipt_reconciliation.example.json",
        "deployment_verification_receipt_reconciliation_history.schema.json",
        "deployment_verification_receipt_reconciliation_history.example.json",
        "deployment_verification_evidence_bundle.schema.json",
        "deployment_verification_evidence_bundle.example.json",
        "deployment_verification_evidence_bundle_history.schema.json",
        "deployment_verification_evidence_bundle_history.example.json",
        "deployment_verification_evidence_bundle_transport_locator.schema.json",
        "deployment_verification_evidence_bundle_transport_locator.example.json",
        "deployment_verification_evidence_bundle_locator_pointer.schema.json",
        "deployment_verification_evidence_bundle_locator_pointer.example.json",
        "deployment_verification_evidence_bundle_locator_pointer_history.schema.json",
        "deployment_verification_evidence_bundle_locator_pointer_history.example.json",
        "deployment_verification_evidence_bundle_locator_pointer_reconciliation.schema.json",
        "deployment_verification_evidence_bundle_locator_pointer_reconciliation.example.json",
        "deployment_verification_evidence_bundle_locator_pointer_reconciliation_history.schema.json",
        "deployment_verification_evidence_bundle_locator_pointer_reconciliation_history.example.json",
        "deployment_verification_evidence_bundle_locator_pointer_rollback.schema.json",
        "deployment_verification_evidence_bundle_locator_pointer_rollback.example.json",
        "deployment_verification_evidence_bundle_locator_pointer_rollback_history.schema.json",
        "deployment_verification_evidence_bundle_locator_pointer_rollback_history.example.json",
        "deployment_verification_evidence_bundle_rollback.schema.json",
        "deployment_verification_evidence_bundle_rollback.example.json",
        "deployment_verification_evidence_bundle_rollback_history.schema.json",
        "deployment_verification_evidence_bundle_rollback_history.example.json",
        "deployment_verification_evidence_bundle_rollback_reconciliation.schema.json",
        "deployment_verification_evidence_bundle_rollback_reconciliation.example.json",
        "deployment_verification_evidence_bundle_rollback_reconciliation_history.schema.json",
        "deployment_verification_evidence_bundle_rollback_reconciliation_history.example.json",
        "deployment_verification_evidence_bundle_rollback_supersession.schema.json",
        "deployment_verification_evidence_bundle_rollback_supersession.example.json",
        "deployment_verification_evidence_bundle_rollback_supersession_reconciliation.schema.json",
        "deployment_verification_evidence_bundle_rollback_supersession_reconciliation.example.json",
        "deployment_verification_evidence_bundle_rollback_supersession_reconciliation_history.schema.json",
        "deployment_verification_evidence_bundle_rollback_supersession_reconciliation_history.example.json",
        "deployment_verification_evidence_bundle_rollback_supersession_history.schema.json",
        "deployment_verification_evidence_bundle_rollback_supersession_history.example.json",
        "deployment_verification_evidence_bundle_transport_locator_history.schema.json",
        "deployment_verification_evidence_bundle_transport_locator_history.example.json",
        "deployment_verification_evidence_bundle_transport_locator_reconciliation.schema.json",
        "deployment_verification_evidence_bundle_transport_locator_reconciliation.example.json",
        "deployment_verification_evidence_bundle_transport_locator_reconciliation_history.schema.json",
        "deployment_verification_evidence_bundle_transport_locator_reconciliation_history.example.json",
        "deployment_verification_evidence_bundle_reconciliation.schema.json",
        "deployment_verification_evidence_bundle_reconciliation.example.json",
        "deployment_verification_evidence_bundle_reconciliation_history.schema.json",
        "deployment_verification_evidence_bundle_reconciliation_history.example.json",
        "deployment_verification_evidence_handoff.schema.json",
        "deployment_verification_evidence_handoff.example.json",
        "deployment_verification_evidence_handoff_history.schema.json",
        "deployment_verification_evidence_handoff_history.example.json",
        "deployment_verification_evidence_handoff_transport_locator.schema.json",
        "deployment_verification_evidence_handoff_transport_locator.example.json",
        "deployment_verification_evidence_handoff_transport_locator_history.schema.json",
        "deployment_verification_evidence_handoff_transport_locator_history.example.json",
        "deployment_verification_evidence_handoff_transport_locator_reconciliation.schema.json",
        "deployment_verification_evidence_handoff_transport_locator_reconciliation.example.json",
        "deployment_verification_evidence_handoff_reconciliation.schema.json",
        "deployment_verification_evidence_handoff_reconciliation.example.json",
        "deployment_verification_evidence_handoff_reconciliation_history.schema.json",
        "deployment_verification_evidence_handoff_reconciliation_history.example.json",
    ] {
        assert!(fixture_exists(fixture), "missing fixture `{fixture}`");
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "just workflow-surface-check-deployment-verification",
        "Deployment verification stays grouped under these operator flows:",
        "Receipt flow: `just deployment-verification-receipt`, `just deployment-verification-receipt-reconcile`.",
        "Receipt support flows: history/reconciliation, transport, locator, and rollback stay grouped under the receipt flow, but the detailed support artifacts stay behind `afterburner verify receipt ...` and `afterburner rollback verification-receipt ...`.",
        "Bundle flow: `just deployment-verification-bundle`, `just deployment-verification-bundle-reconcile`.",
        "Bundle support flows: history/reconciliation, transport, locator, and rollback stay grouped under the bundle flow, but the detailed support artifacts stay behind `afterburner verify bundle ...` and `afterburner rollback verification-bundle ...`.",
        "Handoff flow: `just deployment-verification-handoff`, `just deployment-verification-handoff-reconcile`.",
        "Handoff support flows: history/reconciliation and transport stay grouped under the handoff flow, but the detailed support artifacts stay behind `afterburner verify handoff ...`.",
        "`just workflow-surface-check-deployment-verification` guards the grouped deployment-verification workflow map and contract surface.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped deployment verification surface `{needle}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "Deployment verification artifact groups:",
        "`deployment_verification_receipt*.json`",
        "`deployment_verification_evidence_bundle*.json`",
        "`deployment_verification_evidence_handoff*.json`",
        "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-deployment-verification` for mechanical coverage.",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped deployment verification surface `{needle}`"
        );
    }

    for forbidden in [
        "Receipt sidecars:",
        "Bundle sidecars:",
        "Handoff sidecars:",
        "locator|locator-history|reconcile|reconciliation-history|supersede|supersession-history|supersession-reconcile|supersession-reconciliation-history",
    ] {
        assert!(
            !workflows.contains(forbidden),
            "workflow reference must not regress to taxonomy-heavy deployment verification wording `{forbidden}`"
        );
    }
}
