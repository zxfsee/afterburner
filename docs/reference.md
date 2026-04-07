# Reference Index

Contract and capability detail lives here so [README.md](../README.md) can stay a frontpage and navigation document.

This index is broader than the public operator CLI. Many artifact families listed here are
inspectable implementation surfaces that should remain behind `just` or `afterburner debug ...`
unless they graduate into a clear operator intent.

## Training and inference contracts

- Training-side artifacts and events:
  - `training_scalability_contract.json`
  - `kernel_adoption_thresholds.json`
  - `artifacts/train/observability.jsonl`
  - `train_start`
  - `train_done`
  - `artifact_exported`
  - `artifact_exported` event fields: `backend`, `artifact_version`, `artifact_path`, `manifest_path`, `current_path`
- Inference-side contract surfaces:
  - versioned artifacts under `artifacts/inference/<version>/`
  - `manifest.toml`
  - optional `calibration_artifact_metadata.json`
  - `AFTERBURNER_OBS_JSONL_PATH`
  - normalized stderr event envelopes
- Text-pretraining path:
  - bounded MacBook target
  - decoder-only language model
  - `fineweb-edu/slice`
  - single-node, single-device execution
  - `50M` to `300M` parameters
  - `1024` token context length
  - `50M` to `200M` token budgets
  - `AdamW`
  - `text_token_cache.schema.json`
  - `artifacts/text_inference/<version>/`
  - `text_pretraining_run.json`
  - `artifacts/eval/text_pretraining_eval_summary.json`
  - `just train-text-smoke`
  - tokenizer and packing side
  - Text-trained models should not reuse the current MNIST/logits infer surface
  - source provenance receipt
  - pretraining source registry contract

## Workflow and artifact families

- Deployment:
  - Deployment stack artifact groups:
  - `deployment_stack_profile.example.json`
  - `deployment_stack_check.json`
  - `deployment_stack_launch_plan.json`
  - `deployment_stack_launch_receipt.json`
  - `deployment_stack_launch_evidence_bundle.json`
  - `deployment_stack_launch_evidence_bundle_reconciliation.json`
  - `deployment_stack_launch_evidence_bundle_reconciliation_history.json`
  - `deployment_stack_launch_evidence_handoff.json`
  - `deployment_stack_launch_evidence_handoff_reconciliation.json`
  - `deployment_stack_launch_evidence_handoff_history.json`
  - `deployment_stack_launch_evidence_handoff_reconciliation_history.json`
  - `deployment_stack_launch_transport_locator.json`
  - `deployment_stack_launch_transport_locator_history.json`
  - `deployment_stack_launch_transport_locator_reconciliation.json`
  - `deployment_stack_launch_transport_locator_reconciliation_history.json`
  - `deployment_stack_launch_locator_pointer.json`
  - `deployment_stack_launch_locator_history.json`
  - `deployment_stack_launch_locator_reconciliation.json`
  - `deployment_stack_launch_locator_reconciliation_history.json`
  - `kube_rs_gpu_lease_reconciliation.json`
  - `kube_rs_gpu_lease_reconciliation_history.json`
  - `kube_rs_gpu_lease_pointer.json`
  - `kube_rs_gpu_lease_history.json`
  - Low-level kube lease writers stay behind `afterburner debug deploy lease ...`; use `just kube-rs-lease-reconcile` for the operator-facing lease flow.
  - Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-deployment-stack` for mechanical coverage.
  - Scheduler heartbeat artifact groups:
  - `gpu_scheduler_heartbeat*.json`
  - `gpu_scheduler_heartbeat_pointer*.json`
  - `gpu_scheduler_heartbeat_pointer_rollback*.json`
  - Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-scheduler-heartbeat` for mechanical coverage.
  - `scheduler_runtime_simulation_report.json`
  - `burn_bpk_migration_surface_inventory.json`
  - Deployment verification artifact groups:
  - `deployment_verification_receipt*.json`
  - Receipt support flows: history/reconciliation and transport stay grouped under the receipt operator flow in [docs/workflows.md](./workflows.md).
  - Receipt support flows: locator, rollback, and rollback supersession stay grouped under the receipt operator flow in [docs/workflows.md](./workflows.md).
  - `deployment_verification_evidence_bundle*.json`
  - Bundle support flows: history/reconciliation and transport stay grouped under the bundle operator flow in [docs/workflows.md](./workflows.md).
  - Bundle support flows: locator, rollback, and rollback supersession stay grouped under the bundle operator flow in [docs/workflows.md](./workflows.md).
  - `deployment_verification_evidence_handoff*.json`
  - Handoff support flows: history/reconciliation and transport stay grouped under the handoff operator flow in [docs/workflows.md](./workflows.md).
  - Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-deployment-verification` for mechanical coverage.
  - `distributed_load_profile.json`
  - `artifact_upload_request.json`
  - `huggingface_publish_receipt.json`
  - Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-deployment-utility` for mechanical coverage.
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
  - Low-level baseline writers stay behind `afterburner debug drift baseline ...`; use the grouped `just drift-*` recipes for operator-facing drift flows.
  - Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-drift` for mechanical coverage.
- Cleanup:
  - `artifact_cleanup_inventory.json`
  - `artifact_cleanup_policy.json`
  - `artifact_cleanup_dry_run_receipt.json`
  - `artifact_cleanup_execution_receipt.json`
  - `artifact_cleanup_evidence_bundle.json`
  - Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-cleanup` for mechanical coverage.
- Profiling:
  - `infer_hotspot_summary.json`
  - `profiling_environment_snapshot.json`
  - `profiling_environment_snapshot_refresh.json`
  - `profiling_provenance_receipt.json`
  - `profiling_provenance_evidence_bundle.json`
  - Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-profiling` for mechanical coverage.
- Rollout verification and pointer state:
  - `rollout_check.json`
  - `rollout_verify.json`
  - `inference_current_pointer_promotion.json`
  - `inference_current_pointer_rollback.json`
  - Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map.
- Source and lineage:
  - Pretraining source artifact groups:
  - `pretraining_source_approval_receipt.json`
  - `pretraining_source_provenance_receipt.json`
  - `pretraining_source_provenance_evidence_bundle.json`
  - Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-pretraining-source` for mechanical coverage.
  - Distributed shard lineage artifact groups:
  - `distributed_shard_lineage_receipt.json`
  - `distributed_shard_lineage_evidence_bundle.json`
  - `distributed_shard_lineage_evidence_bundle_reconciliation.json`
  - `distributed_shard_lineage_evidence_bundle_reconciliation_history.json`
  - `distributed_shard_lineage_evidence_handoff.json`
  - `distributed_shard_lineage_evidence_handoff_reconciliation.json`
  - `distributed_shard_lineage_evidence_handoff_history.json`
  - `distributed_shard_lineage_evidence_handoff_reconciliation_history.json`
  - `distributed_shard_lineage_transport_locator.json`
  - `distributed_shard_lineage_transport_locator_reconciliation.json`
  - `distributed_shard_lineage_transport_locator_reconciliation_history.json`
  - `distributed_shard_lineage_locator_pointer.json`
  - `distributed_shard_lineage_locator_history.json`
  - Low-level lineage writers stay behind `afterburner debug lineage ...`; use the grouped `just distributed-shard-lineage-*` recipes for operator-facing flows.
  - Use [docs/workflows.md](./workflows.md) for the grouped operator-flow map and `workflow-surface-check-distributed-shard-lineage` for mechanical coverage.

For command entrypoints, use [docs/workflows.md](./workflows.md).

## Capability and stance summary

- Distributed runtime and scheduler:
  - Current distributed capability surface keeps single-device execution by default and adds a guarded explicit single-node DP train path through `afterburner train --world-size <N> --device-group <name> --participant-devices <ref[,ref...]>`.
  - That DP path currently validates explicit topology inputs and rejects unsupported or ambiguous configurations such as CPU explicit DP and non-concrete participant refs. `ZeRO-1/2/3` and `TP/PP/SP-CP/EP` remain unsupported.
  - The distributed-training roadmap still targets the minimum useful capability subset first rather than broad framework-parity work.
  - `world_size`, ranks, `device_group`, and participant-device refs stay explicit so requested shard topology is inspectable without guessing from scheduler placement.
  - Distributed runtime remains workload-driven DP-first growth; `worker_parallelism` is only a local throughput knob; `distributed_runtime_execution.json`, `distributed_runtime_profile.json`, `distributed_runtime_benchmark_run.json`, and the distributed runtime layout feasibility artifact stay explicit.
  - Scheduler/control plane keeps the lifecycle boundary explicit: `START`, `STOP`, `KILL`, `READY`, `CHECKPOINTED`, `FAILED`, and `HEARTBEAT`. Lease-owned resources and rank assignments stay explicit. Supported preemption is cooperative checkpoint/resume, with constrained GPU colocation only for known low-saturation workloads.
- Operator surface and backend fit:
  - Operator surfaces stay grouped under `afterburner deploy <subcommand>`, `afterburner verify <subcommand>`, `afterburner rollback <subcommand>`, `afterburner drift <subcommand>`, `afterburner cleanup <subcommand>`, `afterburner profile <subcommand>`, and `afterburner source <subcommand>`.
  - Backend/runtime evolution stays measured: `backend_performance_profile.json`, `CubeCL`, `CubeK`, `cutile-rs`, later NVIDIA-specific backend-extension candidate, explicit Metal backend support via `BACKEND=cpu|wgpu|metal`, `process-compose-flake`, and the local process-compose fit.
  - `Parquet`, `DataFusion`, and `Ballista` remain data-infra fit decisions rather than active architecture boundaries.
  - The separate post-training pipeline remains explicit through `model_optimization_profile.schema.json`, quantization/compression/export packaging work, and the output-constraints contract.
  - Burn dependency refresh stays explicit through ADR-033, the current Burn dependency refresh pin `0.20.1`, Burn 0.20.1, `burn_bpk_migration_surface_inventory.json`, and the `burn_stable_release_availability` gate.
- Profiling, provenance, and retention:
  - Profiling remains local-first and adapter-only. `OpenTelemetry` stays parked, the hotspot taxonomy remains `execution`, `framework`, `compiler`, and `incidental`, and current pointer resolution plus profiling environment/provenance artifacts stay explicit.
  - Provenance and retention stay explicit: deployment verification evidence provenance, distributed shard lineage evidence provenance, profiling environment provenance, artifact retention envelope, profiling retention policy, `artifacts/inference/current`, and `artifacts/profiling/`.
  - The deployment verification receipt remains explicit in the reference-owned contract catalog, stays anchored to `artifact_version`, and deployment verification evidence provenance remains anchored to that receipt.
- Longer-horizon anchors:
  - remote locator contract, separate post-training pipeline, distributed checkpoint index contract, distributed runtime profile trials, distributed optimizer-state recovery, `checkpoint_group`, RL rollout metadata contract, `rl_rollout_metadata.schema.json`, RL, vectorized-environment stance, multibillion-scale target envelope, shard metadata contract, `distributed_shard_metadata.schema.json`, `worker_parallelism`, the distributed shard lineage receipt, and the distributed shard lineage evidence provenance contract remain explicit anchors.
  - Source/data anchors remain explicit: pretraining source registry contract, source approval receipt, source provenance receipt, `approval_status`, `upstream_locator`, `source_revision`, `fineweb-edu/slice`, tokenizer and packing surface, and text inference profile sidecar.
