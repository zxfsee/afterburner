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
- `just queue-resume` is the explicit stale-lineage repair-and-continue path: refresh the stamped queue snapshot, rerun the horizon and completion-boundary checks, repin the execute objective with the repaired `CHANGELOG.md` baseline carried forward unchanged, then continue execute preflight.
- `just queue-refresh`, `just queue-fix-top-scope`, and `just queue-execute-preflight` fail fast on a stale `.git/index.lock` before touching queue metadata or running `jj`-backed worktree checks. If no Git or `jj` process is still running, remove the lock and retry.
- `just queue-completion-boundary-check` fails when the most recent landed commit touched the current top TODO scope without advancing `Cargo.toml` / `CHANGELOG.md`, so completed TODOs cannot silently remain active.
- `just queue-fix-top-scope` is the explicit stale-top-scope repair path: pin the narrow metadata objective, validate that only `Cargo.toml` is in-flight, regenerate `CHANGELOG.md`, verify the stamped queue snapshot, then clear the transient lock.
- `just queue-execute-preflight` is the canonical execute-start path: verify queue freshness, pin the top active TODO scope, then validate the current worktree against the execute objective before implementation continues.
- `just queue-execute-preflight --repair-stale-snapshot` keeps the same surface but performs the explicit stale-lineage repair first: refresh the queue snapshot, rerun the horizon and completion-boundary checks, then continue the normal preflight while carrying the repaired `CHANGELOG.md` baseline into execute pinning unchanged.
- If `just queue-execute-preflight` fails because `Cargo.toml` and `CHANGELOG.md` are the only rejected paths, treat that as stale top TODO scope metadata and run `just queue-fix-top-scope` before repinning execute.
- If queue snapshot lineage is behind because queue work resumed after backlog-only or maintenance commits, run `just queue-execute-preflight --repair-stale-snapshot` to repair and continue, `just queue-resume` to do the same repair explicitly, or `just queue-refresh` if you only want to refresh the queue snapshot.
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

- Deployment stays grouped under these operator-facing families:
- Contract check: `just deploy-check` validates `deployment_target_profile.example.json`, `deployment_stack_profile.example.json`, and writes `deployment_stack_check.json`.
- Deployment stack launch and kube-rs lease workflows stay grouped under these families:
- Launch planning: `just deploy-launch-plan`, `just deploy-launch-receipt`.
- Bundle flow: `just deploy-launch-bundle`, `just deploy-reconcile-launch-bundle`, `just deploy-record-launch-bundle-reconciliation-history`.
- Handoff flow: `just deploy-launch-handoff`, `just deploy-reconcile-launch-handoff`, `just deploy-record-launch-handoff-history`, `just deploy-record-launch-handoff-reconciliation-history`.
- Locator flow: `just deploy-point-launch-transport-locator`, `just deploy-record-launch-transport-locator-history`, `just deploy-reconcile-launch-transport-locator`, `just deploy-record-launch-transport-locator-reconciliation-history`, `just deploy-point-launch-locator`, `just deploy-reconcile-launch-locator`, `just deploy-record-launch-locator-history`, `just deploy-record-launch-locator-reconciliation-history`.
- Kube lease flow: `just kube-rs-lease-reconcile`, `just kube-rs-lease-point`, `just kube-rs-lease-record-history`, `just kube-rs-lease-record-reconciliation-history`.
- `just workflow-surface-check-deployment-stack` keeps the grouped deployment-stack recipe and reference split checked.
- Scheduler heartbeat stays grouped under these workflow entrypoints:
- `just scheduler-heartbeat` writes `gpu_scheduler_heartbeat.json`.
- `just scheduler-heartbeat-point` writes `gpu_scheduler_heartbeat_pointer.json`.
- `just scheduler-heartbeat-history` writes `gpu_scheduler_heartbeat_history.json`.
- `just scheduler-heartbeat-reconcile` writes `gpu_scheduler_heartbeat_reconciliation.json`.
- `just scheduler-heartbeat-reconciliation-history` writes `gpu_scheduler_heartbeat_reconciliation_history.json`.
- `just scheduler-heartbeat-supersede` writes `gpu_scheduler_heartbeat_supersession.json`.
- `just scheduler-heartbeat-supersession <history|reconcile|reconciliation-history>` covers the remaining heartbeat supersession artifacts.
- `just scheduler-heartbeat-pointer <history|reconcile|supersede|rollback>` covers pointer history, reconciliation, supersession, and rollback artifacts.
- `just scheduler-heartbeat-pointer-supersession <history|reconcile|reconciliation-history>` covers pointer supersession artifacts.
- `just scheduler-heartbeat-pointer-rollback <history|reconcile|reconciliation-history|supersede>` covers pointer rollback artifacts.
- `just scheduler-heartbeat-pointer-rollback-supersession <history|reconcile|reconciliation-history>` covers pointer rollback supersession artifacts.
- `just workflow-surface-check-scheduler-heartbeat` keeps the exhaustive CLI, fixture, and grouped recipe surface checked.
- Deployment utility workflows stay grouped under these entrypoints:
- Scheduler/runtime simulation: `just scheduler-runtime-simulate`.
- Burn migration inventory: `just burn-bpk-migration-surface-report`, `cargo nextest run --locked --test burn_stable_release_availability`.
- Load and publish utilities: `just distributed-load-profile`, `afterburner deploy hf-publish --request <path>`.
- `just workflow-surface-check-deployment-matrix` keeps the grouped deployment matrix wording checked across the main deployment section.
- `just workflow-surface-check-deployment-utility` keeps the grouped deployment-utility recipe and reference split checked.

## Rollout verification

- `just rollout-check` is a thin wrapper over `afterburner deploy rollout-check`; it runs eval, upload planning, and deploy-check together and writes `rollout_check.json`.
- `just rollout-promote` is a thin wrapper over `afterburner deploy promote-current`; it advances the current inference pointer, saves the previous version, and writes `inference_current_pointer_promotion.json`.
- `just rollout-verify` is a thin wrapper over `afterburner verify rollout`; it exercises infer and eval on the promoted artifact and writes `rollout_verify.json`.
- `just rollout-rollback` is a thin wrapper over `afterburner rollback current-pointer`; it restores the saved previous current pointer and writes `inference_current_pointer_rollback.json`.
- Deployment verification stays grouped under three family maps:
- Receipt family: `just deployment-verification-receipt` anchors `deployment_verification_receipt*.json`.
- Receipt sidecars: `just deployment-verification-receipt-history <record|reconciliation>`, `just deployment-verification-receipt-reconcile`, `just deployment-verification-receipt-transport <point|history|reconcile|reconciliation-history>`, `just deployment-verification-receipt-locator <point|history|reconcile|reconciliation-history>`, and `just deployment-verification-receipt-rollback <locator|locator-history|reconcile|reconciliation-history|supersede|supersession-history|supersession-reconcile|supersession-reconciliation-history>` cover the receipt history, reconciliation, transport, locator, and rollback artifacts.
- Bundle family: `just deployment-verification-bundle` anchors `deployment_verification_evidence_bundle*.json`.
- Bundle sidecars: `just deployment-verification-bundle-history <record|reconciliation>`, `just deployment-verification-bundle-reconcile`, `just deployment-verification-bundle-transport <point|history|reconcile|reconciliation-history>`, `just deployment-verification-bundle-locator <point|history|reconcile|reconciliation-history>`, and `just deployment-verification-bundle-rollback <locator|locator-history|apply|history|reconcile|reconciliation-history|supersede|supersession-history|supersession-reconcile|supersession-reconciliation-history>` cover the bundle history, reconciliation, transport, locator, and rollback artifacts.
- Handoff family: `just deployment-verification-handoff` anchors `deployment_verification_evidence_handoff*.json`.
- Handoff sidecars: `just deployment-verification-handoff-history <record|reconciliation>`, `just deployment-verification-handoff-reconcile`, and `just deployment-verification-handoff-transport <point|history|reconcile>` cover the handoff history, transport, and reconciliation artifacts.
- `just workflow-surface-check-deployment-verification` keeps the exhaustive CLI, fixture, and recipe surface checked so this page can stay grouped instead of listing every artifact variant explicitly.

## Cleanup

- Cleanup stays grouped under these workflow families:
- Inventory and policy: `just cleanup-inventory`, `just cleanup-policy`.
- Execution flow: `just cleanup-dry-run`, `just cleanup-execute`.
- Evidence packaging: `just cleanup-evidence-bundle`.
- `just workflow-surface-check-cleanup` keeps the grouped cleanup recipe and reference split checked.

## Drift

- `just drift-receipt` writes `infer_output_drift_receipt.json`.
- Drift stays grouped under these workflow families:
- Baseline setup: `just drift-baseline`, `just drift-approve-baseline`.
- Approved baseline state: `just drift-point-approved-baseline`, `just drift-record-approved-baseline-history`, `just drift-checkpoint-baseline`.
- Baseline exports: `just drift-export-baseline-bundle`, `just drift-export-baseline-handoff`, `just drift-point-baseline-transport-locator`.
- Baseline changes: `just drift-rollback-approved-baseline`, `just drift-supersede-baseline-approval`, `just drift-refresh-baseline`.
- `just workflow-surface-check-drift` keeps the grouped drift recipe and reference split checked.

## Profiling

- Profiling stays grouped under these workflow families:
- Hotspot capture: `just profile-infer`.
- `just profile-infer` is a thin wrapper over `afterburner profile infer`; it resolves the artifact through `artifacts/inference/current` and follows the active `BACKEND` env contract.
- On macOS, profiling requires `xcrun xctrace version` under full Xcode.
- The native profiling command forces `XCTRACE=/usr/bin/xctrace` while clearing `DEVELOPER_DIR` and `SDKROOT`.
- Environment capture: `just profile-environment-snapshot`, `just profile-refresh-environment-snapshot`.
- Profiling provenance: `just profile-provenance-receipt`, `just profile-provenance-bundle`.
- `just workflow-surface-check-profiling` keeps the grouped profiling recipe and reference split checked.

## Source and lineage

- Pretraining source stays grouped under two workflow entrypoints:
- `just pretraining-source-approval` writes `pretraining_source_approval_receipt.json`.
- `just pretraining-source-provenance <receipt|bundle>` covers `pretraining_source_provenance_receipt.json` and `pretraining_source_provenance_evidence_bundle.json`.
- The `source provenance receipt` captures reviewed source metadata over one approval receipt; `bundle` packages the approval and provenance evidence together.
- `just workflow-surface-check-pretraining-source` keeps the grouped CLI and recipe surface checked.
- Distributed shard lineage stays grouped under eight workflow entrypoints:
- `just distributed-shard-lineage-receipt` writes `distributed_shard_lineage_receipt.json`.
- `just distributed-shard-lineage-bundle` writes `distributed_shard_lineage_evidence_bundle.json`.
- `just distributed-shard-lineage-bundle-reconcile` writes `distributed_shard_lineage_evidence_bundle_reconciliation.json`.
- `just distributed-shard-lineage-bundle-history` writes `distributed_shard_lineage_evidence_bundle_reconciliation_history.json`.
- `just distributed-shard-lineage-handoff` writes `distributed_shard_lineage_evidence_handoff.json`.
- `just distributed-shard-lineage-handoff-reconcile` writes `distributed_shard_lineage_evidence_handoff_reconciliation.json`.
- `just distributed-shard-lineage-handoff-history <record|reconciliation>` covers `distributed_shard_lineage_evidence_handoff_history.json` and `distributed_shard_lineage_evidence_handoff_reconciliation_history.json`.
- `just distributed-shard-lineage-locator <point|history>` covers `distributed_shard_lineage_locator_pointer.json` and `distributed_shard_lineage_locator_history.json`.
- `just distributed-shard-lineage-locator-transport <point|reconcile|history>` covers `distributed_shard_lineage_transport_locator.json`, `distributed_shard_lineage_transport_locator_reconciliation.json`, and `distributed_shard_lineage_transport_locator_reconciliation_history.json`.
- `just workflow-surface-check-distributed-shard-lineage` keeps the grouped CLI and recipe surface checked.
