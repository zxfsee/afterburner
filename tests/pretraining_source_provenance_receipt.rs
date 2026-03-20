use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn pretraining_source_provenance_receipt_contract_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-031"),
        "architecture decisions index must link ADR-031"
    );
    assert!(
        architecture.contains("source provenance receipt"),
        "architecture must mention the pretraining source provenance receipt contract"
    );
    assert!(
        architecture.contains("upstream_locator"),
        "architecture must anchor the provenance receipt to upstream_locator"
    );

    let adr = repo_file("docs/adr/031-pretraining-source-provenance-receipt.md");
    for needle in [
        "pretraining_source_approval_receipt.json",
        "registry_entry_path",
        "upstream_locator",
        "reviewed_metadata_sha256",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-031 must mention `{needle}` as part of source approval provenance"
        );
    }

    let readme = repo_file("README.md");
    assert!(
        readme.contains("source provenance receipt"),
        "README must mention the pretraining source provenance receipt contract"
    );
}
