#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

#[test]
fn distributed_checkpoint_index_contract_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-018"),
        "architecture decisions index must link ADR-018"
    );

    let adr = repo_file("docs/adr/018-distributed-checkpoint-index.md");
    assert!(
        adr.contains("distributed_shard_metadata.schema.json"),
        "ADR-018 must reference the shard metadata contract"
    );
    assert!(
        adr.contains("artifact_version"),
        "ADR-018 must mention artifact_version as part of the checkpoint index"
    );
    assert!(
        adr.contains("checkpoint_root"),
        "ADR-018 must mention checkpoint_root as part of the checkpoint index"
    );
    assert!(
        adr.contains("index-only"),
        "ADR-018 must make the index-only stance explicit"
    );

    let readme = repo_file("docs/reference.md");
    for needle in ["checkpoint index", "distributed_shard_metadata"] {
        assert!(
            readme.contains(needle),
            "reference index must mention the distributed checkpoint index anchor `{needle}`"
        );
    }
}
