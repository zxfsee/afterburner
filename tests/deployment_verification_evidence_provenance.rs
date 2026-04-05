#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

#[test]
fn deployment_verification_evidence_provenance_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-027"),
        "architecture decisions index must link ADR-027"
    );
    let adr = repo_file("docs/adr/027-deployment-verification-evidence-provenance.md");
    for needle in [
        "deployment_verification_receipt.json",
        "evidence_sources",
        "artifact_path",
        "event_name",
        "observed_at_unix_ms",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-027 must mention `{needle}` as part of evidence provenance"
        );
    }

    let readme = repo_file("docs/reference.md");
    for needle in [
        "deployment verification evidence provenance",
        "deployment verification receipt",
    ] {
        assert!(
            readme.contains(needle),
            "reference index must mention the deployment verification provenance anchor `{needle}`"
        );
    }
}
