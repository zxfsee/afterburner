use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn deployment_verification_receipt_contract_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-022"),
        "architecture decisions index must link ADR-022"
    );
    assert!(
        architecture.contains("deployment verification receipt"),
        "architecture must mention the deployment verification receipt contract"
    );
    assert!(
        architecture.contains("artifact_version"),
        "architecture must anchor the receipt to artifact_version"
    );

    let adr = repo_file("docs/adr/022-deployment-verification-receipt.md");
    for needle in [
        "deployment_verification_receipt.json",
        "deployment_verification_receipt_written",
        "artifact_version",
        "profile_name",
        "verification_status",
        "verified_at_unix_ms",
        "evidence",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-022 must mention `{needle}` as part of the receipt contract"
        );
    }

    let readme = repo_file("README.md");
    assert!(
        readme.contains("deployment_verification_receipt.json"),
        "README must mention the deployment verification receipt artifact"
    );
    assert!(
        readme.contains("rollout-verify"),
        "README must anchor the receipt to rollout-verify"
    );
}
