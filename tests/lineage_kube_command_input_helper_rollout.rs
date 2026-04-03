use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn lineage_and_kube_lease_families_use_shared_command_input_helpers() {
    for path in [
        "src/cmd_distributed_shard_lineage_evidence_bundle.rs",
        "src/cmd_distributed_shard_lineage_evidence_bundle_reconcile.rs",
        "src/cmd_distributed_shard_lineage_evidence_bundle_reconciliation_history.rs",
        "src/cmd_distributed_shard_lineage_handoff.rs",
        "src/cmd_distributed_shard_lineage_handoff_history.rs",
        "src/cmd_distributed_shard_lineage_handoff_reconcile.rs",
        "src/cmd_distributed_shard_lineage_locator_history.rs",
        "src/cmd_distributed_shard_lineage_locator_pointer.rs",
        "src/cmd_distributed_shard_lineage_receipt.rs",
        "src/cmd_distributed_shard_lineage_transport_locator.rs",
        "src/cmd_distributed_shard_lineage_transport_locator_reconcile.rs",
        "src/cmd_distributed_shard_lineage_transport_locator_reconciliation_history.rs",
        "src/cmd_kube_rs_lease_history.rs",
        "src/cmd_kube_rs_lease_pointer.rs",
        "src/cmd_kube_rs_lease_reconcile.rs",
        "src/cmd_kube_rs_lease_reconciliation_history.rs",
    ] {
        let text = repo_file(path);
        for forbidden in [
            "fn load_json_object(",
            "fn read_string(",
            "fn read_u64(",
            "fn load_history(",
            "fn history_entries_mut(",
            "fn write_json(",
            "fn read_string_array(",
        ] {
            assert!(
                !text.contains(forbidden),
                "{path} must use shared command-input helpers instead of local `{forbidden}`"
            );
        }
    }
}
