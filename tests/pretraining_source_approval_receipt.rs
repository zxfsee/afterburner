use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn pretraining_source_approval_receipt_contract_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-026"),
        "architecture decisions index must link ADR-026"
    );
    assert!(
        architecture.contains("source approval receipt"),
        "architecture must mention the pretraining source approval receipt contract"
    );
    assert!(
        architecture.contains("approval_status"),
        "architecture must anchor the receipt to approval_status"
    );

    let adr = repo_file("docs/adr/026-pretraining-source-approval-receipt.md");
    for needle in [
        "pretraining_source_approval_receipt.json",
        "source",
        "source_revision",
        "approval_status",
        "approved_by",
        "approval_ticket",
        "approved_at_unix_ms",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-026 must mention `{needle}` as part of the source approval receipt contract"
        );
    }

    let readme = repo_file("README.md");
    assert!(
        readme.contains("pretraining_source_approval_receipt.json"),
        "README must mention the pretraining source approval receipt artifact"
    );
}
