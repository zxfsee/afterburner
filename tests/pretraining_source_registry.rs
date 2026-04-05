use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn pretraining_source_registry_contract_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-021"),
        "architecture decisions index must link ADR-021"
    );
    let adr = repo_file("docs/adr/021-pretraining-source-registry.md");
    assert!(
        adr.contains("pretraining_dataset_manifest.schema.json"),
        "ADR-021 must reference the dataset manifest contract"
    );
    for needle in [
        "`source`",
        "`source_revision`",
        "`upstream_locator`",
        "`license`",
        "`approval_status`",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-021 must mention {needle} as part of the source registry decision"
        );
    }

    let readme = repo_file("docs/reference.md");
    for needle in ["source registry", "source_revision"] {
        assert!(
            readme.contains(needle),
            "reference index must mention the source registry anchor `{needle}`"
        );
    }
}
