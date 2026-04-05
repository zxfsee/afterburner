use std::fs;
use std::path::PathBuf;

mod support;

use support::repo_file;

#[test]
fn distributed_shard_lineage_evidence_provenance_is_documented() {
    let architecture = repo_file("ARCHITECTURE.md");
    assert!(
        architecture.contains("ADR-032"),
        "architecture decisions index must link ADR-032"
    );

    let adr = repo_file("docs/adr/032-distributed-shard-lineage-evidence-provenance.md");
    for needle in [
        "distributed_shard_lineage_receipt.json",
        "evidence_sources",
        "metadata_path",
        "checkpoint_root",
        "observed_at_unix_ms",
    ] {
        assert!(
            adr.contains(needle),
            "ADR-032 must mention `{needle}` as part of lineage evidence provenance"
        );
    }

    let readme = repo_file("docs/reference.md");
    for needle in [
        "distributed shard lineage evidence provenance",
        "distributed shard lineage receipt",
    ] {
        assert!(
            readme.contains(needle),
            "reference index must mention the distributed shard lineage provenance anchor `{needle}`"
        );
    }
}
