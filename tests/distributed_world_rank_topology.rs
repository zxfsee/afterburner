use std::fs;
use std::path::PathBuf;

mod support;

use support::repo_file;

#[test]
fn distributed_world_rank_topology_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-039"),
        "architecture decisions index must link ADR-039"
    );
    for needle in [
        "training_scalability_contract.json",
        "distributed_shard_metadata.schema.json",
        "not the topology source of truth",
        "world-size",
        "device-group",
    ] {
        assert!(
            architecture.contains(needle),
            "architecture must mention `{needle}` as part of the topology stance"
        );
    }

    let adr = repo_file("docs/adr/039-distributed-world-rank-topology.md");
    for needle in [
        "`world_size`",
        "`global_rank`",
        "`local_rank`",
        "`node_rank`",
        "`device_group`",
        "`owner_worker_id`",
        "`owner_worker_rank`",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-039 must mention `{needle}` as part of the topology decision"
        );
    }

    let readme = repo_file("docs/reference.md");
    assert!(
        readme.contains("`world_size`, ranks, and `device_group` metadata"),
        "reference index must document the explicit topology stance"
    );
}
