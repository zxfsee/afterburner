# Workflows

Operational workflow details live here so the README can stay focused on project shape and first-run orientation.

## Core repo workflows

- `just build` runs the reproducible Nix build.
- `just check` runs the full flake checks.
- `just fmt` runs the repo formatter.
- `just workflows` lists the canonical repo-managed entrypoints.
- `just test` runs the default fast test surface.

## Training and operator entrypoints

- `just train` runs the default training path.
- `just infer` runs inference through the current artifact pointer.
- `just eval` writes the deterministic eval summary artifact.
- `just eval-gate` runs the deterministic eval acceptance gate.
- `just backend-profile-gate` validates the backend performance decision profile.
- `just dashboard` opens the terminal observability dashboard.
- `afterburner train --task text --dataset-manifest <path> --tokenizer-profile <path> --token-cache <path>` is the text training adapter entrypoint.
- `just train-text <dataset-manifest> <tokenizer-profile> <token-cache>` runs the bounded text adapter path.
- `just train-text-smoke` exercises the bounded text smoke path and writes `text_pretraining_eval_summary.json`.

## Validation and hygiene

- `just workspace-gate` validates the workspace/core dependency boundaries.
- `just huggingface-publish <request>` shells the provider-neutral publish request through the Hugging Face adapter.

## Deployment

- `just deploy-check` validates `deployment_target_profile.example.json`, `deployment_stack_profile.example.json`, and writes `deployment_stack_check.json`.
- `just deploy-launch-plan <target-profile> <stack-profile>` writes `deployment_stack_launch_plan.json`.
- `just deploy-launch-receipt <plan> <port> <launched-at-unix-ms>` writes `deployment_stack_launch_receipt.json`.
- `just deploy-launch-bundle <receipt>` writes `deployment_stack_launch_evidence_bundle.json`.
- `just deploy-launch-handoff <bundle>` writes `deployment_stack_launch_evidence_handoff.json`.
- `just deploy-record-launch-handoff-history <handoff> <event> <recorded-at-unix-ms>` writes `deployment_stack_launch_evidence_handoff_history.json`.
- `just deploy-point-launch-transport-locator <handoff>` writes `deployment_stack_launch_transport_locator.json`.
- `just deploy-point-launch-locator <locator>` writes `deployment_stack_launch_locator_pointer.json`.
- `just deploy-reconcile-launch-locator <plan> <pointer>` writes `deployment_stack_launch_locator_reconciliation.json`.
- `just deploy-record-launch-locator-history <pointer> <event> <recorded-at-unix-ms>` writes `deployment_stack_launch_locator_history.json`.
- `just deploy-record-launch-locator-reconciliation-history <reconciliation> <event> <recorded-at-unix-ms>` writes `deployment_stack_launch_locator_reconciliation_history.json`.
- `just kube-rs-lease-reconcile <lease> <namespace> <resource-name>` writes `kube_rs_gpu_lease_reconciliation.json`.
- `just kube-rs-lease-point <reconciliation>` writes `kube_rs_gpu_lease_pointer.json`.
- `just kube-rs-lease-record-history <pointer> <event> <recorded-at-unix-ms>` writes `kube_rs_gpu_lease_history.json`.
- `just distributed-load-profile <addr> <requests> <concurrency> <latency-budget-ms-p99> <error-budget-ratio>` writes `distributed_load_profile.json`.
- `afterburner deploy hf-publish --request <path>` writes `huggingface_publish_receipt.json`.

## Rollout verification

- `just rollout-check <candidate-artifact> <candidate-manifest> <ownership> <provider> <destination>` runs eval, upload planning, and deploy-check together.
- `just rollout-promote <artifact-version>` advances the current inference pointer.
- `just rollout-verify <candidate-artifact>` exercises infer and eval on the promoted artifact.
- `just rollout-rollback` restores the previous current pointer.
- `just deployment-verification-receipt <artifact-version> <profile-name> <verification-status> <verified-at-unix-ms> <evidence> <evidence-source-1> <evidence-source-2>` writes `deployment_verification_receipt.json`.
- `just deployment-verification-bundle <receipt>` writes `deployment_verification_evidence_bundle.json`.
- `just deployment-verification-handoff <bundle>` writes `deployment_verification_evidence_handoff.json`.

## Cleanup

- `just cleanup-inventory` writes `artifact_cleanup_inventory.json`.
- `just cleanup-policy <profile>` writes `artifact_cleanup_policy.json`.
- `just cleanup-dry-run <inventory> <policy> <generated-at-unix-ms>` writes `artifact_cleanup_dry_run_receipt.json`.
- `just cleanup-execute <dry-run-receipt> <artifacts-root> <executed-at-unix-ms>` writes `artifact_cleanup_execution_receipt.json`.
- `just cleanup-evidence-bundle <execution-receipt>` writes `artifact_cleanup_evidence_bundle.json`.

## Drift

- `just drift-receipt <candidate-summary> <current-summary> <policy>` writes `infer_output_drift_receipt.json`.
- `just drift-baseline <summary> <receipt>` writes `infer_output_drift_baseline.json`.
- `just drift-approve-baseline <baseline> <approved-by> <approval-ticket> <approved-at-unix-ms>` writes `infer_output_drift_baseline_approval.json`.
- `just drift-point-approved-baseline <approval>` writes `infer_output_drift_baseline_pointer.json`.
- `just drift-record-approved-baseline-history <pointer> <event> <recorded-at-unix-ms>` writes `infer_output_drift_baseline_history.json`.
- `just drift-checkpoint-baseline <pointer> <history>` writes `infer_output_drift_baseline_checkpoint.json`.
- `just drift-export-baseline-bundle <pointer> <history>` writes `infer_output_drift_baseline_bundle.json`.
- `just drift-export-baseline-handoff <bundle>` writes `infer_output_drift_baseline_handoff.json`.
- `just drift-point-baseline-transport-locator <handoff>` writes `infer_output_drift_baseline_transport_locator.json`.
- `just drift-rollback-approved-baseline <current-pointer> <restored-approval> <rolled-back-at-unix-ms>` writes `infer_output_drift_baseline_rollback.json`.
- `just drift-supersede-baseline-approval <previous-approval> <next-approval> <superseded-at-unix-ms>` writes `infer_output_drift_baseline_supersession.json`.
- `just drift-refresh-baseline <summary> <receipt> <current-baseline>` writes `infer_output_drift_baseline_refresh.json`.

## Profiling

- `just profile-infer` writes `infer_hotspot_summary.json`, follows `artifacts/inference/current`, and follows the active `BACKEND` env contract.
- On macOS, profiling requires `xcrun xctrace version` under full Xcode.
- The profiling recipe forces `XCTRACE=/usr/bin/xctrace` while clearing `DEVELOPER_DIR` and `SDKROOT`.
- `just profile-environment-snapshot` writes `profiling_environment_snapshot.json`.
- `just profile-refresh-environment-snapshot <current-snapshot> <profiler-path> <captured-at-unix-ms>` writes `profiling_environment_snapshot_refresh.json`.
- `just profile-provenance-receipt <snapshot> <captured-at-unix-ms>` writes `profiling_provenance_receipt.json`.
- `just profile-provenance-bundle <snapshot> <summary> <captured-at-unix-ms>` writes `profiling_provenance_evidence_bundle.json`.

## Source and lineage

- `just pretraining-source-approval-receipt <source> <source-revision> <approval-status> <approved-by> <approval-ticket> <approved-at-unix-ms>` writes `pretraining_source_approval_receipt.json`.
- `just pretraining-source-provenance-receipt <approval-receipt> <registry-entry-path> <upstream-locator> <reviewed-metadata-sha256>` writes `pretraining_source_provenance_receipt.json`.
- That workflow is the source provenance receipt layer over the source approval receipt and reviewed source metadata.
- `just pretraining-source-provenance-evidence-bundle <provenance-receipt>` writes `pretraining_source_provenance_evidence_bundle.json`.
- `just distributed-shard-lineage-receipt <metadata> <shard-id> <source> <source-revision> <checkpoint-group> <checkpoint-root> <checked-at-unix-ms>` writes `distributed_shard_lineage_receipt.json`.
- `just distributed-shard-lineage-evidence-bundle <receipt>` writes `distributed_shard_lineage_evidence_bundle.json`.
- `just distributed-shard-lineage-handoff <bundle>` writes `distributed_shard_lineage_evidence_handoff.json`.
- `just distributed-shard-lineage-record-handoff-history <handoff> <event> <recorded-at-unix-ms>` writes `distributed_shard_lineage_evidence_handoff_history.json`.
- `just distributed-shard-lineage-point-transport-locator <handoff>` writes `distributed_shard_lineage_transport_locator.json`.
- `just distributed-shard-lineage-point-locator <locator>` writes `distributed_shard_lineage_locator_pointer.json`.
- `just distributed-shard-lineage-record-locator-history <pointer> <event> <recorded-at-unix-ms>` writes `distributed_shard_lineage_locator_history.json`.
