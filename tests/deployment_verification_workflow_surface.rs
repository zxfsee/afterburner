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
    let main_rs = repo_file("src/main.rs");
    for cli in [
        "verification-receipt",
        "verification-bundle",
        "reconcile-verification-bundle",
        "verification-handoff",
        "reconcile-verification-handoff",
        "record-verification-handoff-reconciliation-history",
    ] {
        assert!(
            main_rs.contains(&format!("\"{cli}\"")),
            "src/main.rs must expose deploy subcommand `{cli}`"
        );
    }

    let justfile = repo_file("justfile");
    for recipe in [
        "deployment-verification-receipt artifact_version profile_name verification_status verified_at_unix_ms evidence evidence_source_1 evidence_source_2:",
        "deployment-verification-bundle receipt:",
        "deployment-verification-reconcile-bundle receipt bundle:",
        "deployment-verification-handoff bundle:",
        "deployment-verification-reconcile-handoff bundle handoff:",
        "deployment-verification-record-handoff-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "workflow-surface-check-deployment-verification:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    for fixture in [
        "deployment_verification_receipt.schema.json",
        "deployment_verification_receipt.example.json",
        "deployment_verification_evidence_bundle.schema.json",
        "deployment_verification_evidence_bundle.example.json",
        "deployment_verification_evidence_bundle_reconciliation.schema.json",
        "deployment_verification_evidence_bundle_reconciliation.example.json",
        "deployment_verification_evidence_handoff.schema.json",
        "deployment_verification_evidence_handoff.example.json",
        "deployment_verification_evidence_handoff_reconciliation.schema.json",
        "deployment_verification_evidence_handoff_reconciliation.example.json",
        "deployment_verification_evidence_handoff_reconciliation_history.schema.json",
        "deployment_verification_evidence_handoff_reconciliation_history.example.json",
    ] {
        assert!(fixture_exists(fixture), "missing fixture `{fixture}`");
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "just deployment-verification-receipt",
        "just deployment-verification-bundle",
        "just deployment-verification-reconcile-bundle",
        "just deployment-verification-handoff",
        "just deployment-verification-reconcile-handoff",
        "just deployment-verification-record-handoff-reconciliation-history",
        "just workflow-surface-check-deployment-verification",
        "deployment_verification_receipt.json",
        "deployment_verification_evidence_bundle.json",
        "deployment_verification_evidence_bundle_reconciliation.json",
        "deployment_verification_evidence_handoff.json",
        "deployment_verification_evidence_handoff_reconciliation.json",
        "deployment_verification_evidence_handoff_reconciliation_history.json",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must mention `{needle}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "deployment_verification_receipt.json",
        "deployment_verification_evidence_bundle.json",
        "deployment_verification_evidence_bundle_reconciliation.json",
        "deployment_verification_evidence_handoff.json",
        "deployment_verification_evidence_handoff_reconciliation.json",
        "deployment_verification_evidence_handoff_reconciliation_history.json",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must mention `{needle}`"
        );
    }
}
