use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn artifact_cleanup_execution_receipt_contract_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-029"),
        "architecture decisions index must link ADR-029"
    );
    assert!(
        architecture.contains("cleanup execution receipt"),
        "architecture must mention the cleanup execution receipt contract"
    );
    assert!(
        architecture.contains("artifact_cleanup_dry_run_receipt.json"),
        "architecture must anchor the execution receipt to the cleanup dry-run receipt"
    );

    let adr = repo_file("docs/adr/029-artifact-cleanup-execution-receipt.md");
    for needle in [
        "artifact_cleanup_execution_receipt.json",
        "artifact_cleanup_dry_run_receipt.json",
        "removed_paths",
        "skipped_paths",
        "executed_at_unix_ms",
        "policy_profile",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-029 must mention `{needle}` as part of the cleanup execution receipt contract"
        );
    }

    let readme = repo_file("README.md");
    assert!(
        readme.contains("artifact_cleanup_execution_receipt.json"),
        "README must mention the cleanup execution receipt artifact"
    );
}
