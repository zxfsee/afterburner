use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn distributed_shard_lineage_family_stays_grouped_and_surface_complete() {
    let justfile = repo_file("justfile");
    for recipe in [
        "distributed-shard-lineage-receipt metadata shard_id source source_revision checkpoint_group checkpoint_root checked_at_unix_ms:",
        "distributed-shard-lineage-bundle receipt:",
        "distributed-shard-lineage-bundle-manage action +args:",
        "distributed-shard-lineage-handoff bundle:",
        "distributed-shard-lineage-handoff-manage action +args:",
        "distributed-shard-lineage-locator-manage action +args:",
        "workflow-surface-check-distributed-shard-lineage:",
    ] {
        assert!(justfile.contains(recipe), "justfile must expose `{recipe}`");
    }

    for forbidden in [
        "distributed-shard-lineage-evidence-bundle receipt:",
        "distributed-shard-lineage-reconcile-evidence-bundle receipt bundle:",
        "distributed-shard-lineage-record-evidence-bundle-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "distributed-shard-lineage-reconcile-handoff bundle handoff:",
        "distributed-shard-lineage-record-handoff-history handoff event recorded_at_unix_ms:",
        "distributed-shard-lineage-record-handoff-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "distributed-shard-lineage-point-transport-locator handoff:",
        "distributed-shard-lineage-reconcile-transport-locator handoff locator:",
        "distributed-shard-lineage-record-transport-locator-reconciliation-history reconciliation event recorded_at_unix_ms:",
        "distributed-shard-lineage-point-locator locator:",
        "distributed-shard-lineage-record-locator-history pointer event recorded_at_unix_ms:",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "justfile must not keep per-variant lineage recipe `{forbidden}` after grouping"
        );
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Distributed shard lineage stays grouped under six workflow entrypoints:",
        "`just distributed-shard-lineage-receipt` writes `distributed_shard_lineage_receipt.json`.",
        "`just distributed-shard-lineage-bundle` writes `distributed_shard_lineage_evidence_bundle.json`.",
        "`just distributed-shard-lineage-bundle-manage <reconcile|record-reconciliation-history>`",
        "`just distributed-shard-lineage-handoff` writes `distributed_shard_lineage_evidence_handoff.json`.",
        "`just distributed-shard-lineage-handoff-manage <reconcile|record-history|record-reconciliation-history>`",
        "`just distributed-shard-lineage-locator-manage <point-transport|reconcile-transport|record-transport-reconciliation-history|point|record-history>`",
        "`just workflow-surface-check-distributed-shard-lineage` keeps the grouped CLI and recipe surface checked.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped distributed-shard-lineage surface `{needle}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "Distributed shard lineage family groups:",
        "`distributed_shard_lineage_receipt.json`",
        "`distributed_shard_lineage_evidence_bundle*.json`",
        "`distributed_shard_lineage_evidence_handoff*.json`",
        "`distributed_shard_lineage_transport_locator*.json`",
        "`distributed_shard_lineage_locator*.json`",
        "Use [docs/workflows.md](./workflows.md) for the grouped entrypoint map and `workflow-surface-check-distributed-shard-lineage` for mechanical coverage.",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped distributed-shard-lineage surface `{needle}`"
        );
    }
}
