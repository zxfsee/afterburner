use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn artifact_cleanup_dry_run_receipt_contract_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-024"),
        "architecture decisions index must link ADR-024"
    );
    assert!(
        architecture.contains("cleanup dry-run receipt"),
        "architecture must mention the cleanup dry-run receipt contract"
    );
    assert!(
        architecture.contains("artifact_cleanup_inventory.json"),
        "architecture must anchor the dry-run receipt to the cleanup inventory artifact"
    );

    let adr = repo_file("docs/adr/024-artifact-cleanup-dry-run-receipt.md");
    for needle in [
        "artifact_cleanup_dry_run_receipt.json",
        "artifact_cleanup_inventory.json",
        "planned_removals",
        "retained_count",
        "prune_candidate_count",
        "generated_at_unix_ms",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-024 must mention `{needle}` as part of the dry-run receipt contract"
        );
    }

    let readme = repo_file("README.md");
    assert!(
        readme.contains("artifact_cleanup_dry_run_receipt.json"),
        "README must mention the cleanup dry-run receipt artifact"
    );
}
