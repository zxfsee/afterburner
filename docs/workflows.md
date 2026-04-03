# Workflows

Operational workflow details live here so the README can stay focused on project shape and first-run orientation.

`justfile` is the executable source of truth for exact recipe parameters. This page is a grouped workflow map: which recipe family to use, what it does, and which artifact family it materializes.

Operator workflows should prefer a small intent-first command set. If a step exists only to write
one internal artifact variant, append one history record, or materialize one reconciliation delta,
prefer a `just` recipe or `afterburner debug ...` surface over a new public operator command.

## Operator command model

- Public top-level intents should stay small: `deploy`, `verify`, `rollback`, `inspect`, `debug`.
- Public commands should use a resource/action shape, for example
  `afterburner deploy scheduler-heartbeat ...` or `afterburner inspect scheduler-heartbeat ...`,
  rather than flattening artifact taxonomy into one long command name.
- Public orchestration commands may apply safe defaults from current pointers or profiles, but they
  must report the assumptions they made.
- Keep `--dry-run` and `--explain` on public orchestration commands when they automate multiple
  internal steps or apply assumptions.
- Low-level artifact writers, reconciliation builders, history appenders, and pointer mutations
  belong behind `just` or `afterburner debug ...`, not the main operator surface.

## Public CLI admission rule

- Expose only operator-intent commands on the public CLI.
- A command that only writes one artifact variant, history record, reconciliation delta, or pointer
  update does not qualify as a public operator command by default.
- New public commands must map to a human action such as deploy, verify, rollback, inspect, or a
  similarly clear operator task.
- If a command mainly exists for fixture generation, transition bookkeeping, reconciliation, or
  history capture, keep it behind `just` or `afterburner debug ...` unless operators need to run it
  directly.
- Explicit artifacts still matter underneath; tests, fixtures, reconciliation artifacts, and
  history artifacts prove the lower layers without expanding the public command surface.

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
- `just objective-lock-pin-top-scope-fix` pins the narrow metadata-repair objective that allows only `Cargo.toml` and `CHANGELOG.md`.
- `just objective-lock-pin-queue`, `just objective-lock-pin-backlog`, `just objective-lock-pin-docs`, and `just objective-lock-pin-review` pin the non-execution objective classes before queue, backlog, docs, or review-only passes.
- `just objective-lock-check-worktree <action>` validates the current worktree paths against the pinned objective before commits or phase switches.
- `just objective-lock-clear` clears the transient `.git/afterburner/objective-lock.json` record before repinning a different objective.
- `just queue-refresh` is the canonical queue-only refresh path: pin queue objective, regenerate `CHANGELOG.md`, verify the stamped queue snapshot, then clear the transient lock.
- `just queue-refresh`, `just queue-fix-top-scope`, and `just queue-execute-preflight` fail fast on a stale `.git/index.lock` before touching queue metadata or running `jj`-backed worktree checks. If no Git or `jj` process is still running, remove the lock and retry.
- `just queue-fix-top-scope` is the explicit stale-top-scope repair path: pin the narrow metadata objective, validate that only `Cargo.toml` is in-flight, regenerate `CHANGELOG.md`, verify the stamped queue snapshot, then clear the transient lock.
- `just queue-execute-preflight` is the canonical execute-start path: verify queue freshness, pin the top active TODO scope, then validate the current worktree against the execute objective before implementation continues.
- If `just queue-execute-preflight` fails because `Cargo.toml` and `CHANGELOG.md` are the only rejected paths, treat that as stale top TODO scope metadata and run `just queue-fix-top-scope` before repinning execute.
- If queue snapshot lineage is behind because queue work resumed after backlog-only or maintenance commits, run `just queue-refresh` first.
- `just changelog` validates the queue-refresh objective against `CHANGELOG.md`, regenerates the changelog body, and stamps the active queue snapshot.
- `just changelog-top-scope-fix` is the same changelog regeneration path under the dedicated `top-scope-fix` objective.
- `just queue-snapshot-check` validates that the stamped queue snapshot still matches the current active TODO block and either the current parent commit or the immediately previous parent commit, so a standalone queue-refresh commit stays valid under `jj`.
- `just workflow-surface-check-deployment-verification` validates the deployment verification family stays surface-complete across CLI, `just`, fixtures, workflow docs, and the reference index.
- `just huggingface-publish` shells the provider-neutral publish request through the Hugging Face adapter.

## Deployment

Operator-facing deployment entrypoints should stay narrow. Prefer `just` for proactive multi-step
flows and reserve `afterburner debug deploy ...` for low-level artifact mutation and contract
inspection steps that are useful for tests, fixture generation, or debugging.

### Target operator command matrix

This is the target shape for CLI cleanup work. It is a command-model target, not a claim that every
row is fully implemented yet.

| Intent | Public command shape | Safe defaults / assumptions | Underlying surfaces |
| --- | --- | --- | --- |
| Deploy | `afterburner deploy <resource> ...` | May resolve current pointers, target profiles, or stack profiles when one canonical choice exists; must report what it assumed. | Public command may drive multiple artifact writers underneath. |
| Verify | `afterburner verify <resource> ...` | May resolve the current deployed or promoted artifact/profile by default; must report the evidence inputs it selected. | Public command may emit receipts, bundles, reconciliation artifacts, and history underneath. |
| Rollback | `afterburner rollback <resource> ...` | May restore the current active pointer or previous rollout target by default; must report the source it restored from. | Public command may emit rollback records, pointer updates, supersession artifacts, and history underneath. |
| Inspect | `afterburner inspect <resource> ...` | Should prefer read-only inspection over mutation and default to the current active resource when unambiguous. | Read-only operator surface over existing artifacts and pointers. |
| Heartbeat | `afterburner deploy scheduler-heartbeat ...` | May default output location or current lease/job context only when the source is explicit and inspectable; must report assumptions. | Writes the primary heartbeat artifact while history/reconciliation/supersession helpers stay behind `just` or `debug`. |
| Debug | `afterburner debug <domain> <action> ...` | No hidden orchestration by default; prefer explicit low-level inputs. | Low-level artifact writers, reconciliation builders, history appenders, pointer mutations, and fixture/debug helpers. |

Use this matrix as the admission check for new public commands: if a candidate command does not fit
one of these operator intents, it should default to `just` or `afterburner debug ...` until a real
operator use case justifies promotion.

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
- Scheduler heartbeat stays grouped under three workflow entrypoints:
- `just scheduler-heartbeat` writes `gpu_scheduler_heartbeat.json`.
- `just scheduler-heartbeat-point` writes `gpu_scheduler_heartbeat_pointer.json`.
- `just scheduler-heartbeat-manage <record-history|reconcile|supersede|record-supersession-history|reconcile-supersession|record-supersession-reconciliation-history|point-record-history|point-reconcile|point-supersede|point-record-supersession-history|point-supersession-reconcile|point-record-supersession-reconciliation-history|point-rollback|point-record-rollback-history|point-rollback-reconcile|point-record-rollback-reconciliation-history|point-rollback-supersede|point-record-rollback-supersession-history|point-rollback-supersession-reconcile|point-record-rollback-supersession-reconciliation-history>` covers the remaining heartbeat history, reconciliation, supersession, and rollback artifacts.
- `just workflow-surface-check-scheduler-heartbeat` keeps the exhaustive CLI, fixture, and grouped recipe surface checked.
- `just scheduler-runtime-simulate` writes `scheduler_runtime_simulation_report.json`.
- `just burn-bpk-migration-surface-report` writes `burn_bpk_migration_surface_inventory.json`.
- `cargo nextest run --locked --test burn_stable_release_availability` enforces the checked stable Burn release availability state recorded in ADR-033 before the `.bpk` migration can advance.
- `just distributed-load-profile` writes `distributed_load_profile.json`.
- `afterburner deploy hf-publish --request <path>` writes `huggingface_publish_receipt.json`.

## Rollout verification

- `just rollout-check` runs eval, upload planning, and deploy-check together.
- `just rollout-promote` advances the current inference pointer.
- `just rollout-verify` exercises infer and eval on the promoted artifact.
- `just rollout-rollback` restores the previous current pointer.
- Deployment verification stays grouped under three family maps:
- Receipt family: `just deployment-verification-receipt` anchors `deployment_verification_receipt*.json`.
- Core receipt sidecars: `just deployment-verification-receipt-manage <record-history|reconcile|point-transport-locator|record-transport-locator-history|reconcile-transport-locator|record-transport-locator-reconciliation-history>` covers the receipt history, reconciliation, and transport-locator artifacts.
- Receipt locator and rollback sidecars stay grouped under three entrypoints:
- `just deployment-verification-point-receipt-locator` writes `deployment_verification_receipt_locator_pointer.json`.
- `just deployment-verification-receipt-locator-manage <record-history|reconcile|record-reconciliation-history>` covers the receipt locator history and reconciliation artifacts.
- `just deployment-verification-receipt-rollback-manage <rollback|record-locator-rollback-history|reconcile-rollback|record-rollback-reconciliation-history|supersede-rollback|reconcile-rollback-supersession|record-rollback-supersession-history|record-rollback-supersession-reconciliation-history>` covers the receipt locator rollback and rollback-supersession artifacts.
- Bundle family: `just deployment-verification-bundle` anchors `deployment_verification_evidence_bundle*.json`.
- Core bundle sidecars: `just deployment-verification-bundle-manage <record-history|reconcile|point-transport-locator|record-transport-locator-history|reconcile-transport-locator|record-transport-locator-reconciliation-history>` covers the bundle history, reconciliation, and transport-locator artifacts.
- Bundle locator and rollback sidecars stay grouped under three entrypoints:
- `just deployment-verification-point-bundle-locator` writes `deployment_verification_evidence_bundle_locator_pointer.json`.
- `just deployment-verification-bundle-locator-manage <record-history|reconcile|record-reconciliation-history>` covers the bundle locator history and reconciliation artifacts.
- `just deployment-verification-bundle-rollback-manage <rollback-locator|record-locator-rollback-history|rollback|record-rollback-history|reconcile-rollback|record-rollback-reconciliation-history|supersede-rollback|reconcile-rollback-supersession|record-rollback-supersession-history|record-rollback-supersession-reconciliation-history>` covers the bundle locator rollback, bundle rollback, and rollback-supersession artifacts.
- Handoff family: `just deployment-verification-handoff` anchors `deployment_verification_evidence_handoff*.json`.
- Handoff sidecars: `just deployment-verification-handoff-manage <record-history|point-transport-locator|record-transport-locator-history|reconcile-transport-locator|reconcile|record-reconciliation-history>` covers the handoff history, transport-locator, and reconciliation artifacts.
- `just workflow-surface-check-deployment-verification` keeps the exhaustive CLI, fixture, and recipe surface checked so this page can stay grouped instead of listing every artifact variant explicitly.

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
