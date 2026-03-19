use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn deployment_verification_evidence_provenance_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-027"),
        "architecture decisions index must link ADR-027"
    );
    assert!(
        architecture.contains("deployment verification evidence provenance"),
        "architecture must mention the deployment verification evidence provenance contract"
    );
    assert!(
        architecture.contains("deployment verification receipt"),
        "architecture must anchor evidence provenance to the deployment verification receipt"
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

    let readme = repo_file("README.md");
    assert!(
        readme.contains("evidence provenance"),
        "README must mention the deployment verification evidence provenance contract"
    );
}
