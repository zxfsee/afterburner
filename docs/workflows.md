# Workflows

Operational workflow details live here so the README can stay focused on project shape and first-run orientation.

`justfile` is the executable source of truth for exact recipe parameters. This page is a grouped workflow map: which recipe family to use, what it does, and which artifact family it materializes.

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
- `afterburner train --task text ...` is the text training adapter entrypoint.
- `just train-text` runs the bounded text adapter path.
- `just train-text-smoke` exercises the bounded text smoke path and writes `text_pretraining_eval_summary.json`.

## Validation and hygiene

- `just workspace-gate` validates the workspace/core dependency boundaries.
- `just objective-lock-pin-execute-top-item` verifies the queue snapshot, then pins the current top TODO scope into `.git/afterburner/objective-lock.json` before implementation work starts.
- `just objective-lock-pin-queue`, `just objective-lock-pin-backlog`, `just objective-lock-pin-docs`, and `just objective-lock-pin-review` pin the non-execution objective classes before queue, backlog, docs, or review-only passes.
- `just objective-lock-check-worktree <action>` validates the current worktree paths against the pinned objective before commits or phase switches.
- `just objective-lock-clear` clears the transient `.git/afterburner/objective-lock.json` record before repinning a different objective.
- `just queue-refresh` is the canonical queue-only refresh path: pin queue objective, regenerate `CHANGELOG.md`, verify the stamped queue snapshot, then clear the transient lock.
- `just queue-execute-preflight` is the canonical execute-start path: verify queue freshness, pin the top active TODO scope, then validate the current worktree against the execute objective before implementation continues.
- `just changelog` validates the queue-refresh objective against `CHANGELOG.md`, regenerates the changelog body, and stamps the active queue snapshot.
- `just queue-snapshot-check` validates that the stamped queue snapshot still matches the current active TODO block and either the current parent commit or the immediately previous parent commit, so a standalone queue-refresh commit stays valid under `jj`.
- `just workflow-surface-check-deployment-verification` validates the deployment verification family stays surface-complete across CLI, `just`, fixtures, workflow docs, and the reference index.
- `just huggingface-publish` shells the provider-neutral publish request through the Hugging Face adapter.

## Deployment

- `just deploy-check` validates `deployment_target_profile.example.json`, `deployment_stack_profile.example.json`, and writes `deployment_stack_check.json`.
- `just deploy-launch-plan` writes `deployment_stack_launch_plan.json`.
- `just deploy-launch-receipt` writes `deployment_stack_launch_receipt.json`.
- `just deploy-launch-bundle` writes `deployment_stack_launch_evidence_bundle.json`.
- `just deploy-reconcile-launch-bundle` writes `deployment_stack_launch_evidence_bundle_reconciliation.json`.
- `just deploy-record-launch-bundle-reconciliation-history` writes `deployment_stack_launch_evidence_bundle_reconciliation_history.json`.
- `just deploy-launch-handoff` writes `deployment_stack_launch_evidence_handoff.json`.
- `just deploy-reconcile-launch-handoff` writes `deployment_stack_launch_evidence_handoff_reconciliation.json`.
- `just deploy-record-launch-handoff-history` writes `deployment_stack_launch_evidence_handoff_history.json`.
- `just deploy-record-launch-handoff-reconciliation-history` writes `deployment_stack_launch_evidence_handoff_reconciliation_history.json`.
- `just deploy-point-launch-transport-locator` writes `deployment_stack_launch_transport_locator.json`.
- `just deploy-record-launch-transport-locator-history` writes `deployment_stack_launch_transport_locator_history.json`.
- `just deploy-reconcile-launch-transport-locator` writes `deployment_stack_launch_transport_locator_reconciliation.json`.
- `just deploy-record-launch-transport-locator-reconciliation-history` writes `deployment_stack_launch_transport_locator_reconciliation_history.json`.
- `just deploy-point-launch-locator` writes `deployment_stack_launch_locator_pointer.json`.
- `just deploy-reconcile-launch-locator` writes `deployment_stack_launch_locator_reconciliation.json`.
- `just deploy-record-launch-locator-history` writes `deployment_stack_launch_locator_history.json`.
- `just deploy-record-launch-locator-reconciliation-history` writes `deployment_stack_launch_locator_reconciliation_history.json`.
- `just kube-rs-lease-reconcile` writes `kube_rs_gpu_lease_reconciliation.json`.
- `just kube-rs-lease-point` writes `kube_rs_gpu_lease_pointer.json`.
- `just kube-rs-lease-record-history` writes `kube_rs_gpu_lease_history.json`.
- `just kube-rs-lease-record-reconciliation-history` writes `kube_rs_gpu_lease_reconciliation_history.json`.
- `just distributed-load-profile` writes `distributed_load_profile.json`.
- `afterburner deploy hf-publish --request <path>` writes `huggingface_publish_receipt.json`.

## Rollout verification

- `just rollout-check` runs eval, upload planning, and deploy-check together.
- `just rollout-promote` advances the current inference pointer.
- `just rollout-verify` exercises infer and eval on the promoted artifact.
- `just rollout-rollback` restores the previous current pointer.
- `just deployment-verification-receipt` writes `deployment_verification_receipt.json`.
- `just deployment-verification-record-receipt-history` writes `deployment_verification_receipt_history.json`.
- `just deployment-verification-reconcile-receipt` writes `deployment_verification_receipt_reconciliation.json`.
- `just deployment-verification-record-receipt-reconciliation-history` writes `deployment_verification_receipt_reconciliation_history.json`.
- `just deployment-verification-bundle` writes `deployment_verification_evidence_bundle.json`.
- `just deployment-verification-reconcile-bundle` writes `deployment_verification_evidence_bundle_reconciliation.json`.
- `just deployment-verification-record-bundle-reconciliation-history` writes `deployment_verification_evidence_bundle_reconciliation_history.json`.
- `just deployment-verification-handoff` writes `deployment_verification_evidence_handoff.json`.
- `just deployment-verification-record-handoff-history` writes `deployment_verification_evidence_handoff_history.json`.
- `just deployment-verification-reconcile-handoff` writes `deployment_verification_evidence_handoff_reconciliation.json`.
- `just deployment-verification-record-handoff-reconciliation-history` writes `deployment_verification_evidence_handoff_reconciliation_history.json`.

## Cleanup

- `just cleanup-inventory` writes `artifact_cleanup_inventory.json`.
- `just cleanup-policy` writes `artifact_cleanup_policy.json`.
- `just cleanup-dry-run` writes `artifact_cleanup_dry_run_receipt.json`.
- `just cleanup-execute` writes `artifact_cleanup_execution_receipt.json`.
- `just cleanup-evidence-bundle` writes `artifact_cleanup_evidence_bundle.json`.

## Drift

- `just drift-receipt` writes `infer_output_drift_receipt.json`.
- `just drift-baseline` writes `infer_output_drift_baseline.json`.
- `just drift-approve-baseline` writes `infer_output_drift_baseline_approval.json`.
- `just drift-point-approved-baseline` writes `infer_output_drift_baseline_pointer.json`.
- `just drift-record-approved-baseline-history` writes `infer_output_drift_baseline_history.json`.
- `just drift-checkpoint-baseline` writes `infer_output_drift_baseline_checkpoint.json`.
- `just drift-export-baseline-bundle` writes `infer_output_drift_baseline_bundle.json`.
- `just drift-export-baseline-handoff` writes `infer_output_drift_baseline_handoff.json`.
- `just drift-point-baseline-transport-locator` writes `infer_output_drift_baseline_transport_locator.json`.
- `just drift-rollback-approved-baseline` writes `infer_output_drift_baseline_rollback.json`.
- `just drift-supersede-baseline-approval` writes `infer_output_drift_baseline_supersession.json`.
- `just drift-refresh-baseline` writes `infer_output_drift_baseline_refresh.json`.

## Profiling

- `just profile-infer` writes `infer_hotspot_summary.json`, follows `artifacts/inference/current`, and follows the active `BACKEND` env contract.
- On macOS, profiling requires `xcrun xctrace version` under full Xcode.
- The profiling recipe forces `XCTRACE=/usr/bin/xctrace` while clearing `DEVELOPER_DIR` and `SDKROOT`.
- `just profile-environment-snapshot` writes `profiling_environment_snapshot.json`.
- `just profile-refresh-environment-snapshot` writes `profiling_environment_snapshot_refresh.json`.
- `just profile-provenance-receipt` writes `profiling_provenance_receipt.json`.
- `just profile-provenance-bundle` writes `profiling_provenance_evidence_bundle.json`.

## Source and lineage

- `just pretraining-source-approval-receipt` writes `pretraining_source_approval_receipt.json`.
- `just pretraining-source-provenance-receipt` writes `pretraining_source_provenance_receipt.json`.
- That workflow is the source provenance receipt layer over the source approval receipt and reviewed source metadata.
- `just pretraining-source-provenance-evidence-bundle` writes `pretraining_source_provenance_evidence_bundle.json`.
- `just distributed-shard-lineage-receipt` writes `distributed_shard_lineage_receipt.json`.
- `just distributed-shard-lineage-evidence-bundle` writes `distributed_shard_lineage_evidence_bundle.json`.
- `just distributed-shard-lineage-reconcile-evidence-bundle` writes `distributed_shard_lineage_evidence_bundle_reconciliation.json`.
- `just distributed-shard-lineage-record-evidence-bundle-reconciliation-history` writes `distributed_shard_lineage_evidence_bundle_reconciliation_history.json`.
- `just distributed-shard-lineage-handoff` writes `distributed_shard_lineage_evidence_handoff.json`.
- `just distributed-shard-lineage-reconcile-handoff` writes `distributed_shard_lineage_evidence_handoff_reconciliation.json`.
- `just distributed-shard-lineage-record-handoff-history` writes `distributed_shard_lineage_evidence_handoff_history.json`.
- `just distributed-shard-lineage-record-handoff-reconciliation-history` writes `distributed_shard_lineage_evidence_handoff_reconciliation_history.json`.
- `just distributed-shard-lineage-point-transport-locator` writes `distributed_shard_lineage_transport_locator.json`.
- `just distributed-shard-lineage-reconcile-transport-locator` writes `distributed_shard_lineage_transport_locator_reconciliation.json`.
- `just distributed-shard-lineage-record-transport-locator-reconciliation-history` writes `distributed_shard_lineage_transport_locator_reconciliation_history.json`.
- `just distributed-shard-lineage-point-locator` writes `distributed_shard_lineage_locator_pointer.json`.
- `just distributed-shard-lineage-record-locator-history` writes `distributed_shard_lineage_locator_history.json`.
