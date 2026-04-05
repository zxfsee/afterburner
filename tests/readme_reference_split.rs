use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
}

#[test]
fn readme_points_to_reference_docs_and_stays_navigation_first() {
    let readme = repo_file("README.md");
    assert!(
        readme.contains("[Reference index](./docs/reference.md)"),
        "README frontpage must point readers to the reference index"
    );
    assert!(
        readme.contains(
            "Deeper contract and capability detail now lives in [docs/reference.md](./docs/reference.md)."
        ),
        "README must keep the deeper reference split explicit"
    );

    let reference = repo_file("docs/reference.md");
    for heading in [
        "## Training and inference contracts",
        "## Workflow and artifact families",
        "## Capability and stance summary",
    ] {
        assert!(
            reference.contains(heading),
            "reference index must contain `{heading}`"
        );
    }
    for needle in [
        "Deployment:",
        "Training-side artifacts and events:",
        "`training_scalability_contract.json`",
        "`kernel_adoption_thresholds.json`",
        "`artifacts/train/observability.jsonl`",
        "`train_start`",
        "`train_done`",
        "`artifact_exported`",
        "`artifact_exported` event fields: `backend`, `artifact_version`, `artifact_path`, `manifest_path`, `current_path`",
        "Inference-side contract surfaces:",
        "versioned artifacts under `artifacts/inference/<version>/`",
        "`manifest.toml`",
        "optional `calibration_artifact_metadata.json`",
        "`AFTERBURNER_OBS_JSONL_PATH`",
        "normalized stderr event envelopes",
        "artifact_upload_request.json",
        "`kube_rs_gpu_lease_reconciliation.json`",
        "`kube_rs_gpu_lease_reconciliation_history.json`",
        "`kube_rs_gpu_lease_pointer.json`",
        "`kube_rs_gpu_lease_history.json`",
        "Scheduler heartbeat artifact groups:",
        "`gpu_scheduler_heartbeat*.json`",
        "`gpu_scheduler_heartbeat_pointer*.json`",
        "`gpu_scheduler_heartbeat_pointer_rollback*.json`",
        "Deployment stack artifact groups:",
        "`deployment_stack_check.json`",
        "`deployment_stack_launch_plan.json`",
        "`deployment_stack_launch_receipt.json`",
        "`deployment_stack_launch_evidence_bundle.json`",
        "`deployment_stack_launch_evidence_bundle_reconciliation.json`",
        "`deployment_stack_launch_evidence_bundle_reconciliation_history.json`",
        "`deployment_stack_launch_evidence_handoff.json`",
        "`deployment_stack_launch_evidence_handoff_reconciliation.json`",
        "`deployment_stack_launch_evidence_handoff_history.json`",
        "`deployment_stack_launch_evidence_handoff_reconciliation_history.json`",
        "`deployment_stack_launch_transport_locator.json`",
        "`deployment_stack_launch_transport_locator_history.json`",
        "`deployment_stack_launch_transport_locator_reconciliation.json`",
        "`deployment_stack_launch_transport_locator_reconciliation_history.json`",
        "`deployment_stack_launch_locator_pointer.json`",
        "`deployment_stack_launch_locator_history.json`",
        "`deployment_stack_launch_locator_reconciliation.json`",
        "`deployment_stack_launch_locator_reconciliation_history.json`",
        "Deployment verification artifact groups:",
        "`deployment_verification_receipt*.json`",
        "Receipt support flows: history/reconciliation and transport stay grouped under the receipt operator flow in [docs/workflows.md](./workflows.md).",
        "Receipt support flows: locator, rollback, and rollback supersession stay grouped under the receipt operator flow in [docs/workflows.md](./workflows.md).",
        "Bundle support flows: history/reconciliation and transport stay grouped under the bundle operator flow in [docs/workflows.md](./workflows.md).",
        "Bundle support flows: locator, rollback, and rollback supersession stay grouped under the bundle operator flow in [docs/workflows.md](./workflows.md).",
        "`deployment_verification_evidence_handoff*.json`",
        "Handoff support flows: history/reconciliation and transport stay grouped under the handoff operator flow in [docs/workflows.md](./workflows.md).",
        "`scheduler_runtime_simulation_report.json`",
        "`burn_bpk_migration_surface_inventory.json`",
        "`distributed_load_profile.json`",
        "`huggingface_publish_receipt.json`",
        "Rollout verification and pointer state:",
        "`rollout_check.json`",
        "`rollout_verify.json`",
        "`inference_current_pointer_promotion.json`",
        "`inference_current_pointer_rollback.json`",
        "Cleanup:",
        "`artifact_cleanup_inventory.json`",
        "`artifact_cleanup_policy.json`",
        "`artifact_cleanup_dry_run_receipt.json`",
        "`artifact_cleanup_execution_receipt.json`",
        "`artifact_cleanup_evidence_bundle.json`",
        "Profiling:",
        "`infer_hotspot_summary.json`",
        "`profiling_environment_snapshot.json`",
        "`profiling_environment_snapshot_refresh.json`",
        "`profiling_provenance_receipt.json`",
        "Drift and rollback:",
        "`infer_output_drift_summary.json`",
        "`infer_output_drift_receipt.json`",
        "`infer_output_drift_baseline.json`",
        "`infer_output_drift_baseline_approval.json`",
        "`infer_output_drift_baseline_history.json`",
        "`infer_output_drift_baseline_checkpoint.json`",
        "`infer_output_drift_baseline_bundle.json`",
        "`infer_output_drift_baseline_handoff.json`",
        "`infer_output_drift_baseline_transport_locator.json`",
        "`infer_output_drift_baseline_rollback.json`",
        "`infer_output_drift_baseline_supersession.json`",
        "`infer_output_drift_baseline_refresh.json`",
        "Source and lineage:",
        "Text-pretraining path:",
        "bounded MacBook target",
        "decoder-only language model",
        "`fineweb-edu/slice`",
        "`text_token_cache.schema.json`",
        "`artifacts/text_inference/<version>/`",
        "`text_pretraining_run.json`",
        "`artifacts/eval/text_pretraining_eval_summary.json`",
        "`just train-text-smoke`",
        "tokenizer and packing side",
        "Text-trained models should not reuse the current MNIST/logits infer surface",
        "source provenance receipt",
        "pretraining source registry contract",
        "Pretraining source artifact groups:",
        "Distributed shard lineage artifact groups:",
        "`deployment_verification_evidence_bundle*.json`",
        "infer_output_drift_baseline_pointer.json",
        "profiling_provenance_evidence_bundle.json",
        "distributed_shard_lineage_evidence_handoff.json",
        "remote locator contract",
        "deployment verification evidence provenance",
        "artifact retention envelope",
        "afterburner deploy <subcommand>",
    ] {
        assert!(
            reference.contains(needle),
            "reference index must contain `{needle}`"
        );
    }
}
