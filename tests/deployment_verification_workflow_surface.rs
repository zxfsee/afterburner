use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

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
        "deployment-verification-record-receipt-locator-history pointer event recorded_at_unix_ms:",
        "deployment-verification-point-receipt-locator locator:",
        "deployment-verification-reconcile-receipt-locator locator pointer:",
        "deployment-verification-record-receipt-locator-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-rollback-receipt-locator current_pointer restored_pointer rolled_back_at_unix_ms:",
        "deployment-verification-record-receipt-locator-rollback-history rollback event recorded_at_unix_ms:",
        "deployment-verification-reconcile-receipt-rollback rollback current_rollback:",
        "deployment-verification-record-receipt-rollback-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-supersede-receipt-rollback previous_rollback next_rollback superseded_at_unix_ms:",
        "deployment-verification-reconcile-receipt-rollback-supersession supersession current_supersession:",
        "deployment-verification-record-receipt-rollback-supersession-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-record-receipt-rollback-supersession-history supersession event recorded_at_unix_ms:",
        "deployment-verification-point-receipt-transport-locator receipt:",
        "deployment-verification-record-receipt-transport-locator-history locator event recorded_at_unix_ms:",
        "deployment-verification-reconcile-receipt-transport-locator receipt locator:",
        "deployment-verification-record-receipt-transport-locator-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-record-receipt-history receipt event recorded_at_unix_ms:",
        "deployment-verification-reconcile-receipt receipt artifact_version profile_name verification_status verified_at_unix_ms evidence evidence_source_1 evidence_source_2:",
        "deployment-verification-record-receipt-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-bundle receipt:",
        "deployment-verification-record-bundle-history bundle event recorded_at_unix_ms:",
        "deployment-verification-point-bundle-transport-locator bundle:",
        "deployment-verification-point-bundle-locator locator:",
        "deployment-verification-record-bundle-locator-history pointer event recorded_at_unix_ms:",
        "deployment-verification-reconcile-bundle-locator locator pointer:",
        "deployment-verification-record-bundle-locator-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-rollback-bundle-locator current_pointer restored_pointer rolled_back_at_unix_ms:",
        "deployment-verification-record-bundle-locator-rollback-history rollback event recorded_at_unix_ms:",
        "deployment-verification-rollback-bundle current_bundle restored_bundle rolled_back_at_unix_ms:",
        "deployment-verification-record-bundle-rollback-history rollback event recorded_at_unix_ms:",
        "deployment-verification-reconcile-bundle-rollback rollback current_rollback:",
        "deployment-verification-record-bundle-rollback-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-supersede-bundle-rollback previous_rollback next_rollback superseded_at_unix_ms:",
        "deployment-verification-reconcile-bundle-rollback-supersession supersession current_supersession:",
        "deployment-verification-record-bundle-rollback-supersession-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-record-bundle-rollback-supersession-history supersession event recorded_at_unix_ms:",
        "deployment-verification-record-bundle-transport-locator-history locator event recorded_at_unix_ms:",
        "deployment-verification-reconcile-bundle-transport-locator bundle locator:",
        "deployment-verification-record-bundle-transport-locator-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-reconcile-bundle receipt bundle:",
        "deployment-verification-record-bundle-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "deployment-verification-handoff bundle:",
        "deployment-verification-record-handoff-history handoff event recorded_at_unix_ms:",
        "deployment-verification-point-handoff-transport-locator handoff:",
        "deployment-verification-record-handoff-transport-locator-history locator event recorded_at_unix_ms:",
        "deployment-verification-reconcile-handoff-transport-locator handoff locator:",
        "deployment-verification-reconcile-handoff bundle handoff:",
        "deployment-verification-record-handoff-reconciliation-history reconciliation event recorded_at_unix_ms:",
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
        "Deployment verification stays grouped under three family maps",
        "Receipt family: `just deployment-verification-receipt`",
        "Bundle family: `just deployment-verification-bundle`",
        "Handoff family: `just deployment-verification-handoff`",
        "`just workflow-surface-check-deployment-verification` keeps the exhaustive CLI, fixture, and recipe surface checked",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped deployment verification surface `{needle}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "Deployment verification family groups:",
        "`deployment_verification_receipt*.json`",
        "`deployment_verification_evidence_bundle*.json`",
        "`deployment_verification_evidence_handoff*.json`",
        "Use [docs/workflows.md](./workflows.md) for the exhaustive recipe map and `workflow-surface-check-deployment-verification` for mechanical coverage.",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped deployment verification surface `{needle}`"
        );
    }
}
