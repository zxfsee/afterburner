#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

#[test]
fn cleanup_family_uses_shared_command_input_helpers() {
    for path in [
        "src/cmd_cleanup_dry_run.rs",
        "src/cmd_cleanup_evidence_bundle.rs",
        "src/cmd_cleanup_execute.rs",
        "src/cmd_cleanup_inventory.rs",
        "src/cmd_cleanup_policy.rs",
    ] {
        let text = repo_file(path);
        for forbidden in [
            "fn load_json_object(",
            "fn read_string(",
            "fn read_u64(",
            "fn read_array(",
            "fn read_object(",
            "fn load_history(",
            "fn history_entries_mut(",
            "fn write_json(",
        ] {
            assert!(
                !text.contains(forbidden),
                "{path} must use shared command-input helpers instead of local `{forbidden}`"
            );
        }
    }
}

#[test]
fn provenance_families_use_shared_command_input_helpers() {
    for path in [
        "src/cmd_pretraining_source_provenance_receipt.rs",
        "src/cmd_pretraining_source_provenance_evidence_bundle.rs",
        "src/cmd_profiling_provenance_receipt.rs",
        "src/cmd_profiling_provenance_bundle.rs",
    ] {
        let text = repo_file(path);
        for forbidden in [
            "fn load_json_object(",
            "fn read_string(",
            "fn read_u64(",
            "fn read_array(",
            "fn read_object(",
            "fn read_string_array(",
            "fn load_history(",
            "fn history_entries_mut(",
            "fn write_json(",
        ] {
            assert!(
                !text.contains(forbidden),
                "{path} must use shared command-input helpers instead of local `{forbidden}`"
            );
        }
    }
}

#[test]
fn drift_baseline_family_uses_shared_command_input_helpers() {
    for path in [
        "src/cmd_drift_baseline.rs",
        "src/cmd_drift_baseline_approval.rs",
        "src/cmd_drift_baseline_bundle.rs",
        "src/cmd_drift_baseline_checkpoint.rs",
        "src/cmd_drift_baseline_handoff.rs",
        "src/cmd_drift_baseline_history.rs",
        "src/cmd_drift_baseline_pointer.rs",
        "src/cmd_drift_baseline_refresh.rs",
        "src/cmd_drift_baseline_rollback.rs",
        "src/cmd_drift_baseline_supersession.rs",
        "src/cmd_drift_baseline_transport_locator.rs",
    ] {
        let text = repo_file(path);
        for forbidden in [
            "fn load_json_object(",
            "fn read_string(",
            "fn read_u64(",
            "fn load_history(",
            "fn history_entries_mut(",
            "fn write_json(",
        ] {
            assert!(
                !text.contains(forbidden),
                "{path} must use shared command-input helpers instead of local `{forbidden}`"
            );
        }
    }
}

#[test]
fn deployment_stack_launch_family_uses_shared_command_input_helpers() {
    for path in [
        "src/cmd_deployment_stack_launch_bundle.rs",
        "src/cmd_deployment_stack_launch_bundle_reconcile.rs",
        "src/cmd_deployment_stack_launch_bundle_reconciliation_history.rs",
        "src/cmd_deployment_stack_launch_handoff.rs",
        "src/cmd_deployment_stack_launch_handoff_history.rs",
        "src/cmd_deployment_stack_launch_handoff_reconcile.rs",
        "src/cmd_deployment_stack_launch_locator_history.rs",
        "src/cmd_deployment_stack_launch_locator_pointer.rs",
        "src/cmd_deployment_stack_launch_locator_reconcile.rs",
        "src/cmd_deployment_stack_launch_locator_reconciliation_history.rs",
        "src/cmd_deployment_stack_launch_receipt.rs",
        "src/cmd_deployment_stack_launch_transport_locator.rs",
        "src/cmd_deployment_stack_launch_transport_locator_history.rs",
        "src/cmd_deployment_stack_launch_transport_locator_reconcile.rs",
        "src/cmd_deployment_stack_launch_transport_locator_reconciliation_history.rs",
    ] {
        let text = repo_file(path);
        for forbidden in [
            "fn load_json_object(",
            "fn read_string(",
            "fn read_u64(",
            "fn load_history(",
            "fn history_entries_mut(",
            "fn write_json(",
        ] {
            assert!(
                !text.contains(forbidden),
                "{path} must use shared command-input helpers instead of local `{forbidden}`"
            );
        }
    }
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
