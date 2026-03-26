# Reference Index

Contract and capability detail lives here so [README.md](../README.md) can stay a frontpage and navigation document.

## Training and inference contracts

- Training-side artifacts and events:
  - `training_scalability_contract.json`
  - `kernel_adoption_thresholds.json`
  - `artifacts/train/observability.jsonl`
  - `train_start`
  - `train_done`
  - `artifact_exported`
- Inference-side contract surfaces:
  - versioned artifacts under `artifacts/inference/<version>/`
  - `manifest.toml`
  - optional `calibration_artifact_metadata.json`
  - `AFTERBURNER_OBS_JSONL_PATH`
  - normalized stderr event envelopes
- Text-pretraining path:
  - bounded MacBook target
  - `fineweb-edu/slice`
  - `text_token_cache.schema.json`
  - `artifacts/text_inference/<version>/`
  - `text_pretraining_run.json`
  - `artifacts/eval/text_pretraining_eval_summary.json`
  - `just train-text-smoke`

## Workflow and artifact families

- Deployment:
  - `deployment_stack_check.json`
  - `deployment_stack_launch_plan.json`
  - `deployment_stack_launch_receipt.json`
  - `deployment_stack_launch_evidence_bundle.json`
  - `distributed_load_profile.json`
  - `artifact_upload_request.json`
  - `huggingface_publish_receipt.json`
- Drift and rollback:
  - `infer_output_drift_summary.json`
  - `infer_output_drift_receipt.json`
  - `infer_output_drift_baseline.json`
  - `infer_output_drift_baseline_approval.json`
  - `infer_output_drift_baseline_pointer.json`
  - `infer_output_drift_baseline_history.json`
  - `infer_output_drift_baseline_checkpoint.json`
  - `infer_output_drift_baseline_bundle.json`
  - `infer_output_drift_baseline_handoff.json`
  - `infer_output_drift_baseline_transport_locator.json`
  - `infer_output_drift_baseline_rollback.json`
  - `infer_output_drift_baseline_supersession.json`
  - `infer_output_drift_baseline_refresh.json`
- Cleanup:
  - `artifact_cleanup_inventory.json`
  - `artifact_cleanup_policy.json`
  - `artifact_cleanup_dry_run_receipt.json`
  - `artifact_cleanup_execution_receipt.json`
  - `artifact_cleanup_evidence_bundle.json`
- Profiling:
  - `infer_hotspot_summary.json`
  - `profiling_environment_snapshot.json`
  - `profiling_environment_snapshot_refresh.json`
  - `profiling_provenance_receipt.json`
  - `profiling_provenance_evidence_bundle.json`
- Source and lineage:
  - `pretraining_source_approval_receipt.json`
  - `pretraining_source_provenance_receipt.json`
  - `pretraining_source_provenance_evidence_bundle.json`
  - `distributed_shard_lineage_receipt.json`
  - `distributed_shard_lineage_evidence_bundle.json`
  - `distributed_shard_lineage_evidence_handoff.json`

For command entrypoints, use [docs/workflows.md](./workflows.md).

## Capability and stance summary

- Distributed runtime stays workload-driven: DP first, explicit `world_size`/rank/`device_group` topology, and a separate `distributed_runtime_profile.json` surface for executed trials.
- Scheduler policy stays conservative: priority and queueing first, cooperative checkpoint/resume preemption only, and constrained GPU colocation for known low-saturation workloads.
- Backend/runtime evolution stays measured:
  - `backend_performance_profile.json`
  - `CubeCL`
  - `CubeK`
  - `cutile-rs`
  - Burn 0.20.1
  - `.mpk` to `.bpk`
- Profiling remains local-first and adapter-only:
  - current pointer
  - `BACKEND`
  - `xcrun xctrace version`
  - full Xcode
  - `XCTRACE=/usr/bin/xctrace`
  - `DEVELOPER_DIR`
  - `SDKROOT`
  - OpenTelemetry fit stays parked
- Longer-horizon reference points remain explicit:
  - remote locator contract
  - separate post-training pipeline
  - distributed checkpoint index contract
  - RL rollout metadata contract
  - vectorized-environment stance
  - multibillion-scale target envelope
  - shard metadata contract
  - `Parquet`
  - `DataFusion`
