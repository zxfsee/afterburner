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
        "distributed-shard-lineage-bundle-reconcile receipt bundle:",
        "distributed-shard-lineage-bundle-history reconciliation event recorded_at_unix_ms:",
        "distributed-shard-lineage-handoff bundle:",
        "distributed-shard-lineage-handoff-reconcile bundle handoff:",
        "distributed-shard-lineage-handoff-history action +args:",
        "distributed-shard-lineage-locator action +args:",
        "distributed-shard-lineage-locator-transport action +args:",
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
        "distributed-shard-lineage-bundle-manage action +args:",
        "distributed-shard-lineage-handoff-manage action +args:",
        "distributed-shard-lineage-locator-manage action +args:",
    ] {
        assert!(
            !justfile.contains(forbidden),
            "justfile must not keep per-variant lineage recipe `{forbidden}` after grouping"
        );
    }

    let workflows = repo_file("docs/workflows.md");
    for needle in [
        "Distributed shard lineage stays grouped under these operator flows:",
        "Receipt flow: `just distributed-shard-lineage-receipt`.",
        "Bundle flow: `just distributed-shard-lineage-bundle`, `just distributed-shard-lineage-bundle-reconcile`, `just distributed-shard-lineage-bundle-history`.",
        "Handoff flow: `just distributed-shard-lineage-handoff`, `just distributed-shard-lineage-handoff-reconcile`, `just distributed-shard-lineage-handoff-history <record|reconciliation>`.",
        "Locator flow: `just distributed-shard-lineage-locator <point|history>`, `just distributed-shard-lineage-locator-transport <point|reconcile|history>`.",
        "`just workflow-surface-check-distributed-shard-lineage` guards the grouped lineage workflow map and recipe surface.",
    ] {
        assert!(
            workflows.contains(needle),
            "workflow reference must keep the grouped distributed-shard-lineage surface `{needle}`"
        );
    }
    for forbidden in [
        "just distributed-shard-lineage-bundle-manage",
        "just distributed-shard-lineage-handoff-manage",
        "just distributed-shard-lineage-locator-manage",
        "- `just distributed-shard-lineage-receipt` writes `distributed_shard_lineage_receipt.json`.",
        "- `just distributed-shard-lineage-bundle` writes `distributed_shard_lineage_evidence_bundle.json`.",
        "- `just distributed-shard-lineage-bundle-reconcile` writes `distributed_shard_lineage_evidence_bundle_reconciliation.json`.",
        "- `just distributed-shard-lineage-bundle-history` writes `distributed_shard_lineage_evidence_bundle_reconciliation_history.json`.",
        "- `just distributed-shard-lineage-handoff` writes `distributed_shard_lineage_evidence_handoff.json`.",
        "- `just distributed-shard-lineage-handoff-reconcile` writes `distributed_shard_lineage_evidence_handoff_reconciliation.json`.",
        "- `just distributed-shard-lineage-handoff-history <record|reconciliation>` covers `distributed_shard_lineage_evidence_handoff_history.json` and `distributed_shard_lineage_evidence_handoff_reconciliation_history.json`.",
        "- `just distributed-shard-lineage-locator <point|history>` covers `distributed_shard_lineage_locator_pointer.json` and `distributed_shard_lineage_locator_history.json`.",
        "- `just distributed-shard-lineage-locator-transport <point|reconcile|history>` covers `distributed_shard_lineage_transport_locator.json`, `distributed_shard_lineage_transport_locator_reconciliation.json`, and `distributed_shard_lineage_transport_locator_reconciliation_history.json`.",
    ] {
        assert!(
            !workflows.contains(forbidden),
            "workflow reference must stay grouped instead of enumerating lineage variant `{forbidden}`"
        );
    }

    let reference = repo_file("docs/reference.md");
    for needle in [
        "Distributed shard lineage artifact groups:",
        "`distributed_shard_lineage_receipt.json`",
        "`distributed_shard_lineage_evidence_bundle.json`",
        "`distributed_shard_lineage_evidence_bundle_reconciliation.json`",
        "`distributed_shard_lineage_evidence_bundle_reconciliation_history.json`",
        "`distributed_shard_lineage_evidence_handoff.json`",
        "`distributed_shard_lineage_evidence_handoff_reconciliation.json`",
        "`distributed_shard_lineage_evidence_handoff_history.json`",
        "`distributed_shard_lineage_evidence_handoff_reconciliation_history.json`",
        "`distributed_shard_lineage_transport_locator.json`",
        "`distributed_shard_lineage_transport_locator_reconciliation.json`",
        "`distributed_shard_lineage_transport_locator_reconciliation_history.json`",
        "`distributed_shard_lineage_locator_pointer.json`",
        "`distributed_shard_lineage_locator_history.json`",
        "Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-distributed-shard-lineage` for mechanical coverage.",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must keep the grouped distributed-shard-lineage surface `{needle}`"
        );
    }
}
