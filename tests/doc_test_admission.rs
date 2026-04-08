use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

#[path = "support/repo.rs"]
mod repo_test_support;

use repo_test_support::repo_file;

const DOC_TEST_ADMISSION_ALLOWLIST: &[&str] = &[
    "tests/artifact_cleanup_dry_run_receipt.rs",
    "tests/artifact_cleanup_evidence_bundle.rs",
    "tests/artifact_cleanup_execution_receipt.rs",
    "tests/artifact_cleanup_inventory.rs",
    "tests/artifact_cleanup_policy.rs",
    "tests/artifact_upload_adapter.rs",
    "tests/burn_bpk_migration_surface_inventory.rs",
    "tests/canonical_just_surface.rs",
    "tests/deployment_stack_contract.rs",
    "tests/deployment_stack_launch_evidence_bundle.rs",
    "tests/deployment_stack_launch_evidence_bundle_reconciliation.rs",
    "tests/deployment_stack_launch_evidence_bundle_reconciliation_history.rs",
    "tests/deployment_stack_launch_evidence_handoff.rs",
    "tests/deployment_stack_launch_evidence_handoff_reconciliation.rs",
    "tests/deployment_stack_launch_evidence_handoff_reconciliation_history.rs",
    "tests/deployment_stack_launch_handoff_history.rs",
    "tests/deployment_stack_launch_locator_history.rs",
    "tests/deployment_stack_launch_locator_pointer.rs",
    "tests/deployment_stack_launch_locator_reconciliation.rs",
    "tests/deployment_stack_launch_locator_reconciliation_history.rs",
    "tests/deployment_stack_launch_plan.rs",
    "tests/deployment_stack_launch_receipt.rs",
    "tests/deployment_stack_launch_transport_locator.rs",
    "tests/deployment_stack_launch_transport_locator_history.rs",
    "tests/deployment_stack_launch_transport_locator_reconciliation.rs",
    "tests/deployment_stack_launch_transport_locator_reconciliation_history.rs",
    "tests/developer_workflows.rs",
    "tests/distributed_load_profile.rs",
    "tests/distributed_runtime_benchmark_harness.rs",
    "tests/distributed_runtime_checkpoint_state.rs",
    "tests/distributed_runtime_execution_contract.rs",
    "tests/distributed_runtime_layout_feasibility.rs",
    "tests/distributed_runtime_profile_schema.rs",
    "tests/distributed_shard_lineage_evidence_provenance.rs",
    "tests/distributed_shard_lineage_receipt.rs",
    "tests/gpu_scheduler_lifecycle_message.rs",
    "tests/huggingface_publish_adapter.rs",
    "tests/infer_drift_baseline.rs",
    "tests/infer_drift_baseline_approval.rs",
    "tests/infer_drift_baseline_bundle.rs",
    "tests/infer_drift_baseline_checkpoint.rs",
    "tests/infer_drift_baseline_handoff.rs",
    "tests/infer_drift_baseline_history.rs",
    "tests/infer_drift_baseline_pointer.rs",
    "tests/infer_drift_baseline_refresh.rs",
    "tests/infer_drift_baseline_rollback.rs",
    "tests/infer_drift_baseline_supersession.rs",
    "tests/infer_drift_baseline_transport_locator.rs",
    "tests/infer_drift_receipt.rs",
    "tests/infer_output_drift_summary.rs",
    "tests/kube_rs_gpu_lease_history.rs",
    "tests/kube_rs_gpu_lease_pointer.rs",
    "tests/kube_rs_gpu_lease_reconciliation.rs",
    "tests/kube_rs_gpu_lease_reconciliation_history.rs",
    "tests/model_optimization_profile.rs",
    "tests/optimized_model_capability_surface.rs",
    "tests/optimized_model_local_profile.rs",
    "tests/optimized_model_package_contract.rs",
    "tests/pretraining_source_approval_receipt.rs",
    "tests/pretraining_source_provenance_evidence_bundle.rs",
    "tests/pretraining_source_provenance_receipt.rs",
    "tests/profiling_environment_snapshot.rs",
    "tests/profiling_environment_snapshot_refresh.rs",
    "tests/profiling_hotspot_taxonomy.rs",
    "tests/profiling_provenance_evidence_bundle.rs",
    "tests/profiling_provenance_receipt.rs",
    "tests/profiling_recipe_current_pointer.rs",
    "tests/readme_reference_split.rs",
    "tests/scheduler_runtime_simulation.rs",
    "tests/text_model_artifact_inference_contract.rs",
    "tests/text_pretraining_adapter.rs",
    "tests/text_pretraining_smoke.rs",
    "tests/text_tokenizer_packing_profile.rs",
    "tests/workflow_reference.rs",
];

const DOC_TEST_STRING_POLICING_ALLOWLIST: &[&str] = &[
    "tests/artifact_cleanup_dry_run_receipt.rs",
    "tests/artifact_cleanup_evidence_bundle.rs",
    "tests/artifact_cleanup_execution_receipt.rs",
    "tests/artifact_cleanup_inventory.rs",
    "tests/artifact_cleanup_policy.rs",
    "tests/artifact_upload_adapter.rs",
    "tests/burn_bpk_migration_surface_inventory.rs",
    "tests/deployment_stack_contract.rs",
    "tests/deployment_stack_launch_evidence_bundle.rs",
    "tests/deployment_stack_launch_evidence_bundle_reconciliation.rs",
    "tests/deployment_stack_launch_evidence_bundle_reconciliation_history.rs",
    "tests/deployment_stack_launch_evidence_handoff.rs",
    "tests/deployment_stack_launch_evidence_handoff_reconciliation.rs",
    "tests/deployment_stack_launch_evidence_handoff_reconciliation_history.rs",
    "tests/deployment_stack_launch_handoff_history.rs",
    "tests/deployment_stack_launch_locator_history.rs",
    "tests/deployment_stack_launch_locator_pointer.rs",
    "tests/deployment_stack_launch_locator_reconciliation.rs",
    "tests/deployment_stack_launch_locator_reconciliation_history.rs",
    "tests/deployment_stack_launch_plan.rs",
    "tests/deployment_stack_launch_receipt.rs",
    "tests/deployment_stack_launch_transport_locator.rs",
    "tests/deployment_stack_launch_transport_locator_history.rs",
    "tests/deployment_stack_launch_transport_locator_reconciliation.rs",
    "tests/deployment_stack_launch_transport_locator_reconciliation_history.rs",
    "tests/distributed_load_profile.rs",
    "tests/distributed_runtime_benchmark_harness.rs",
    "tests/distributed_runtime_checkpoint_state.rs",
    "tests/distributed_runtime_execution_contract.rs",
    "tests/distributed_runtime_layout_feasibility.rs",
    "tests/distributed_runtime_profile_schema.rs",
    "tests/distributed_shard_lineage_evidence_provenance.rs",
    "tests/distributed_shard_lineage_receipt.rs",
    "tests/huggingface_publish_adapter.rs",
    "tests/infer_drift_baseline.rs",
    "tests/infer_drift_baseline_approval.rs",
    "tests/infer_drift_baseline_bundle.rs",
    "tests/infer_drift_baseline_checkpoint.rs",
    "tests/infer_drift_baseline_handoff.rs",
    "tests/infer_drift_baseline_history.rs",
    "tests/infer_drift_baseline_pointer.rs",
    "tests/infer_drift_baseline_refresh.rs",
    "tests/infer_drift_baseline_rollback.rs",
    "tests/infer_drift_baseline_supersession.rs",
    "tests/infer_drift_baseline_transport_locator.rs",
    "tests/infer_drift_receipt.rs",
    "tests/infer_output_drift_summary.rs",
    "tests/kube_rs_gpu_lease_history.rs",
    "tests/kube_rs_gpu_lease_pointer.rs",
    "tests/kube_rs_gpu_lease_reconciliation.rs",
    "tests/kube_rs_gpu_lease_reconciliation_history.rs",
    "tests/model_optimization_profile.rs",
    "tests/optimized_model_capability_surface.rs",
    "tests/optimized_model_local_profile.rs",
    "tests/optimized_model_package_contract.rs",
    "tests/pretraining_source_approval_receipt.rs",
    "tests/pretraining_source_provenance_evidence_bundle.rs",
    "tests/pretraining_source_provenance_receipt.rs",
    "tests/profiling_environment_snapshot.rs",
    "tests/profiling_environment_snapshot_refresh.rs",
    "tests/profiling_hotspot_taxonomy.rs",
    "tests/profiling_provenance_evidence_bundle.rs",
    "tests/profiling_provenance_receipt.rs",
    "tests/readme_reference_split.rs",
    "tests/scheduler_runtime_simulation.rs",
    "tests/text_model_artifact_inference_contract.rs",
    "tests/text_pretraining_adapter.rs",
    "tests/text_pretraining_smoke.rs",
    "tests/text_tokenizer_packing_profile.rs",
    "tests/workflow_reference.rs",
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn tracked_doc_surface_reads(text: &str) -> bool {
    text.contains("repo_file(\"README.md\")")
        || text.contains("repo_file(\"docs/reference.md\")")
        || text.contains("repo_file(\"docs/workflows.md\")")
}

fn brittle_doc_policing(text: &str) -> bool {
    text.contains("workflow reference must mention")
        || text.contains("reference index must mention")
        || text.contains("README must mention")
        || text.contains("README frontpage must")
        || text.contains("README must keep")
        || text.contains("README must point")
        || text.contains("reference index must keep the ownership/navigation fragment")
        || text.contains("training/inference section must keep the representative anchor")
        || text.contains(
            "workflow/artifact families section must keep the representative anchor",
        )
        || text.contains(
            "workflow/artifact families section must keep the grouped deployment-verification fragment",
        )
        || text.contains("capability summary must keep the representative anchor")
}

fn grouped_doc_surface_allowlist(file_name: &str) -> bool {
    matches!(
        file_name,
        "docs_surface_catalog.rs"
            | "readme_reference_split.rs"
            | "workflow_reference.rs"
            | "canonical_just_surface.rs"
            | "deployment_verification_workflow_surface.rs"
            | "deployment_verification_workflow_surface_catalog.rs"
            | "deployment_verification_receipt.rs"
            | "deployment_surface_catalog.rs"
            | "workflow_family_surface_catalog.rs"
            | "format_dependency_fit_catalog.rs"
            | "distributed_runtime_fit_catalog.rs"
            | "runtime_platform_fit_catalog.rs"
            | "workload_boundary_fit_catalog.rs"
            | "runtime_deployment_boundary_catalog.rs"
            | "event_contract_catalog.rs"
            | "contract_anchor_catalog.rs"
            | "cli_surface_catalog.rs"
            | "rollout_orchestration_catalog.rs"
            | "reference_doc_validation.rs"
            | "doc_test_admission.rs"
    )
}

fn load_snapshot(snapshot: &[&str]) -> BTreeSet<String> {
    snapshot.iter().map(|entry| (*entry).to_string()).collect()
}

#[test]
fn doc_test_admission_stays_on_the_known_legacy_surface() {
    let doc_reader_allowlist = load_snapshot(DOC_TEST_ADMISSION_ALLOWLIST);
    let string_policing_allowlist = load_snapshot(DOC_TEST_STRING_POLICING_ALLOWLIST);

    let mut unexpected_doc_readers = Vec::new();
    let mut unexpected_string_policing = Vec::new();

    for entry in fs::read_dir(repo_root().join("tests")).expect("read tests dir") {
        let entry = entry.expect("read test entry");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("test filename");
        let rel_path = format!("tests/{file_name}");
        let text =
            fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", rel_path));

        if tracked_doc_surface_reads(&text)
            && !grouped_doc_surface_allowlist(file_name)
            && !doc_reader_allowlist.contains(&rel_path)
        {
            unexpected_doc_readers.push(rel_path.clone());
        }
        if brittle_doc_policing(&text)
            && !grouped_doc_surface_allowlist(file_name)
            && !string_policing_allowlist.contains(&rel_path)
        {
            unexpected_string_policing.push(rel_path);
        }
    }

    assert!(
        unexpected_doc_readers.is_empty(),
        "new doc-reading tests must join the grouped-surface allowlist or the transitional snapshot: {:?}",
        unexpected_doc_readers
    );
    assert!(
        unexpected_string_policing.is_empty(),
        "new docs-string policing tests must join the grouped-surface allowlist or the transitional snapshot: {:?}",
        unexpected_string_policing
    );
}

#[test]
fn doc_test_snapshots_only_keep_still_relevant_legacy_entries() {
    let doc_reader_allowlist = load_snapshot(DOC_TEST_ADMISSION_ALLOWLIST);
    let string_policing_allowlist = load_snapshot(DOC_TEST_STRING_POLICING_ALLOWLIST);

    let stale_doc_readers = doc_reader_allowlist
        .iter()
        .filter(|rel_path| {
            let text = repo_file(rel_path);
            !tracked_doc_surface_reads(&text)
        })
        .cloned()
        .collect::<Vec<_>>();
    let stale_string_policing = string_policing_allowlist
        .iter()
        .filter(|rel_path| {
            let text = repo_file(rel_path);
            !brittle_doc_policing(&text)
        })
        .cloned()
        .collect::<Vec<_>>();

    assert!(
        stale_doc_readers.is_empty(),
        "doc-reader legacy snapshot must not keep stale entries: {:?}",
        stale_doc_readers
    );
    assert!(
        stale_string_policing.is_empty(),
        "docs-string legacy snapshot must not keep stale entries: {:?}",
        stale_string_policing
    );
}
