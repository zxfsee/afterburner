use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn distributed_shard_lineage_contract_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-023"),
        "architecture decisions index must link ADR-023"
    );

    let adr = repo_file("docs/adr/023-distributed-shard-lineage.md");
    for needle in [
        "distributed_shard_metadata.schema.json",
        "source",
        "source_revision",
        "checkpoint_group",
        "shard_id",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-023 must mention `{needle}` as part of shard lineage"
        );
    }

    let readme = repo_file("docs/reference.md");
    for needle in [
        "Distributed shard lineage artifact groups",
        "source_revision",
    ] {
        assert!(
            readme.contains(needle),
            "reference index must mention the distributed shard lineage anchor `{needle}`"
        );
    }
}
