use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn distributed_shard_lineage_receipt_contract_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-028"),
        "architecture decisions index must link ADR-028"
    );
    assert!(
        architecture.contains("distributed shard lineage receipt"),
        "architecture must mention the distributed shard lineage receipt contract"
    );
    assert!(
        architecture.contains("checkpoint_group"),
        "architecture must anchor the lineage receipt to checkpoint_group"
    );

    let adr = repo_file("docs/adr/028-distributed-shard-lineage-receipt.md");
    for needle in [
        "distributed_shard_lineage_receipt.json",
        "shard_id",
        "source",
        "source_revision",
        "checkpoint_group",
        "checked_at_unix_ms",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-028 must mention `{needle}` as part of the lineage receipt contract"
        );
    }

    let readme = repo_file("README.md");
    assert!(
        readme.contains("distributed_shard_lineage_receipt.json"),
        "README must mention the distributed shard lineage receipt artifact"
    );
}
