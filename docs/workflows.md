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
- `just test-root-compile-smoke` is the compile-only smoke path for root-package integration tests.
- `just test-repo-workflow-compile-smoke` is the compile-only smoke path for the repo workflow crate.
- `just test-compile-smoke` is the compile-only integration-test smoke path for broad helper/layout changes.
- `just test-doc-gates` runs the shared docs/reference validators.
- `just test-repo-workflow-gates` runs the package-scoped queue/objective-lock workflow guards.
- `just test-queue-gates` runs the full queue guard set across package boundaries: repo-workflow gates plus the root `developer_workflows` coverage.
- `just test-deploy-family` runs the grouped deploy-family workflow guards.
- `just test-drift-family` runs the grouped drift-family workflow guards.
- `just objective-lock-pin-execute-top-item` verifies the queue snapshot, then pins the current top TODO scope into `.git/afterburner/objective-lock.json` before implementation work starts.
- `just objective-lock-pin-top-scope-fix` pins the narrow metadata-repair objective that allows only `Cargo.toml` and `CHANGELOG.md`.
- `just objective-lock-pin-queue`, `just objective-lock-pin-backlog`, `just objective-lock-pin-docs`, and `just objective-lock-pin-review` pin the non-execution objective classes before queue, backlog, docs, or review-only passes.
- `just objective-lock-check-worktree <action>` validates the current worktree paths against the pinned objective before commits or phase switches.
- `just objective-lock-clear` clears the transient `.git/afterburner/objective-lock.json` record before repinning a different objective.
- Queue state refresh and repair: `just queue-refresh`, `just queue-top-runnable-check`, `just queue-promote-next-runnable`, `just queue-snapshot-check`, `just queue-completion-boundary-check`.
- `just repo-lock-repair` is the typed stale-lock recovery path. It removes `.git/index.lock` only when the lock age is beyond the configured stale threshold.
- `just queue-refresh`, `just queue-fix-top-scope`, and `just queue-promote-next-runnable` run `just repo-lock-repair` before checking repo locks, so aged stale index locks are repaired on the canonical queue-maintenance path.
- `just queue-execute-preflight` still fails fast on a stale `.git/index.lock` before touching queue metadata or running `jj`-backed worktree checks.
- Queue metadata repair: `just queue-fix-top-scope`, `just changelog-top-scope-fix`.
- `just queue-fix-top-scope` preserves unchanged in-scope worktree paths from the current top TODO while regenerating `CHANGELOG.md` and restamping the queue snapshot.
- Queue execution entry: `just queue-execute-preflight`, `just queue-execute-preflight --repair-stale-snapshot`, `just queue-resume`.
- `just queue-execute-preflight` delegates queue freshness checks, optional stale-snapshot repair, execute pinning, and worktree validation to the typed queue preflight helper, carrying unchanged `Cargo.toml` and `CHANGELOG.md` forward when a top-scope repair already updated them.
- If `just queue-top-runnable-check` reports a blocked top active TODO, run `just queue-promote-next-runnable` before execution so the highest-priority runnable item becomes the active top slot.
- If the active queue is empty and no runnable backlog item exists, `just queue-top-runnable-check` succeeds and `just queue-refresh` simply restamps the empty queue horizon.
- If `just queue-execute-preflight` fails because `Cargo.toml` and `CHANGELOG.md` are the only rejected paths, treat that as stale top TODO scope metadata and run `just queue-fix-top-scope` before repinning execute.
- If queue snapshot lineage is behind because queue work resumed after backlog-only or maintenance commits, run `just queue-execute-preflight --repair-stale-snapshot` to repair and continue, `just queue-resume` to do the same repair explicitly, or `just queue-refresh` if you only want to refresh the queue snapshot.
- `just changelog` validates the queue-refresh objective against `CHANGELOG.md`, regenerates the changelog body, and stamps the active queue snapshot.
- `just queue-snapshot-check` validates that the stamped queue snapshot still matches the current active TODO block and either the current parent commit or the immediately previous parent commit, so a standalone queue-refresh commit stays valid under `jj`.
- `just workflow-surface-check-deployment-verification` validates the deployment verification family stays surface-complete across CLI, `just`, fixtures, workflow docs, and the reference index.
- `just workflow-surface-check-public-cli` keeps the admitted public CLI family set and grouped-path guard checked.
- `just workflow-surface-check-operator-grammar` keeps the grouped nested operator grammar checked.
- `just workflow-surface-check-justfile-thinness` keeps the current `justfile` branching and generic bucket debt bounded to the explicit allowlist.
- `just workflow-surface-check-reference-docs` validates the family-level workflow-to-reference linkage notes in `docs/reference.md`.
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
| Heartbeat | `afterburner deploy scheduler-heartbeat ...` | May default output location or current lease/job context only when the source is explicit and inspectable; must report assumptions. | Writes the primary heartbeat artifact while history/reconciliation/supersession/pointer helpers stay behind `afterburner debug deploy scheduler-heartbeat ...`. |
| Debug | `afterburner debug <domain> <action> ...` | No hidden orchestration by default; prefer explicit low-level inputs. | Low-level artifact writers, reconciliation builders, history appenders, pointer mutations, and fixture/debug helpers. |

Use this matrix as the admission check for new public commands: if a candidate command does not fit
one of these operator intents, it should default to `just` or `afterburner debug ...` until a real
operator use case justifies promotion.

- Deployment stays grouped under these operator-facing families:
- Contract check: `just deploy-check`.
- Deployment stack launch and kube-rs lease stay grouped under these operator flows:
- Launch planning: `just deploy-launch-plan`, `just deploy-launch-receipt`.
- Bundle flow: `just deploy-launch-bundle`, `just deploy-launch-bundle-reconcile`.
- Handoff flow: `just deploy-launch-handoff`, `just deploy-launch-handoff-reconcile`.
- Low-level launch writers stay behind `afterburner debug deploy launch ...`; the grouped `just deploy-launch-*` recipes remain the operator-facing launch entrypoint.
- Kube lease flow: `just kube-rs-lease-reconcile`.
- Low-level kube lease writers stay behind `afterburner debug deploy lease ...`; `just kube-rs-lease-reconcile` remains the operator-facing lease entrypoint.
- `just workflow-surface-check-deployment-stack` guards the grouped deployment-stack workflow map and reference split.
- Scheduler heartbeat stays grouped under these operator flows:
- Primary heartbeat flow: `just scheduler-heartbeat`.
- Detailed heartbeat history, reconciliation, supersession, pointer, and rollback support artifacts stay behind `afterburner debug deploy scheduler-heartbeat ...`.
- `just workflow-surface-check-scheduler-heartbeat` keeps the grouped heartbeat workflow map and contract surface checked.
- Deployment utility stays grouped under these operator flows:
- Scheduler/runtime simulation: `just scheduler-runtime-simulate`.
- Burn migration inventory: `just burn-bpk-migration-surface-report`, `cargo nextest run --locked --test burn_stable_release_availability`.
- Load and publish utilities: `just distributed-load-profile`, `afterburner deploy hf-publish --request <path>`.
- `just workflow-surface-check-deployment-matrix` keeps the grouped deployment matrix wording checked across the main deployment section.
- `just workflow-surface-check-deployment-utility` guards the grouped deployment-utility workflow map and reference split.

## Rollout verification

- Rollout verification stays grouped under these operator flows:
- Candidate check: `just rollout-check` delegates to `afterburner deploy rollout-check`.
- Promotion: `just rollout-promote` delegates to `afterburner deploy promote-current`.
- Post-promotion verification: `just rollout-verify` delegates to `afterburner verify rollout`.
- Rollback: `just rollout-rollback` delegates to `afterburner rollback current-pointer`.
- Deployment verification stays grouped under these operator flows:
- Receipt flow: `just deployment-verification-receipt`, `just deployment-verification-receipt-reconcile`.
- Receipt support flows: history/reconciliation, transport, locator, and rollback stay grouped under the receipt flow, but the detailed support artifacts stay behind `afterburner verify receipt ...` and `afterburner rollback verification-receipt ...`.
- Bundle flow: `just deployment-verification-bundle`, `just deployment-verification-bundle-reconcile`.
- Bundle support flows: history/reconciliation, transport, locator, and rollback stay grouped under the bundle flow, but the detailed support artifacts stay behind `afterburner verify bundle ...` and `afterburner rollback verification-bundle ...`.
- Handoff flow: `just deployment-verification-handoff`, `just deployment-verification-handoff-reconcile`.
- Handoff support flows: history/reconciliation and transport stay grouped under the handoff flow, but the detailed support artifacts stay behind `afterburner verify handoff ...`.
- `just workflow-surface-check-deployment-verification` guards the grouped deployment-verification workflow map and contract surface.

## Cleanup

- Cleanup stays grouped under these operator flows:
- Scope and policy: `just cleanup-inventory`, `just cleanup-policy`.
- Plan and execute: `just cleanup-dry-run`, `just cleanup-execute`.
- Evidence pack: `just cleanup-evidence-bundle`.
- `just workflow-surface-check-cleanup` guards the grouped cleanup workflow map and reference split.

## Drift

- Drift stays grouped under these operator flows:
- Signal capture: `just drift-receipt`.
- Baseline setup: `just drift-baseline`, `just drift-approve-baseline`.
- Detailed baseline state, checkpointing, export, transport, rollback, and supersession artifacts stay behind `afterburner debug drift baseline ...`; the grouped `just drift-*` recipes remain the operator-facing drift entrypoint.
- Baseline refresh: `just drift-refresh-baseline`.
- `just workflow-surface-check-drift` guards the grouped drift workflow map and reference split.

## Profiling

- Profiling stays grouped under these operator flows:
- Hotspot capture and infer path: `just profile-infer` delegates to `afterburner profile infer`.
- The native profiling command still owns current pointer resolution and any explicit `BACKEND`
  override contract behind the stable `just profile-infer` entrypoint.
- On macOS, profiling requires `xcrun xctrace version` under full Xcode.
- The native profiling command forces `XCTRACE=/usr/bin/xctrace` while clearing `DEVELOPER_DIR` and `SDKROOT`.
- Environment capture: `just profile-environment-snapshot`, `just profile-refresh-environment-snapshot`.
- Provenance and evidence: `just profile-provenance-receipt`, `just profile-provenance-bundle`.
- `just workflow-surface-check-profiling` guards the grouped profiling workflow map and reference split.

## Source and lineage

- Pretraining source stays grouped under these operator flows:
- Approval flow: `just pretraining-source-approval`.
- `just pretraining-source-approval` writes `pretraining_source_approval_receipt.json`.
- Provenance flow: `just pretraining-source-provenance-receipt`, `just pretraining-source-provenance-evidence-bundle`.
- The `source provenance receipt` captures reviewed source metadata over one approval receipt; `bundle` packages the approval and provenance evidence together.
- `just workflow-surface-check-pretraining-source` guards the grouped source workflow map and recipe surface.
- Distributed shard lineage stays grouped under these operator flows:
- Low-level lineage artifact writers stay behind `afterburner debug lineage ...`; the grouped `just distributed-shard-lineage-*` recipes remain the operator-facing entrypoint.
- Receipt flow: `just distributed-shard-lineage-receipt`.
- `just distributed-shard-lineage-receipt` writes `distributed_shard_lineage_receipt.json`.
- Bundle flow: `just distributed-shard-lineage-bundle`, `just distributed-shard-lineage-bundle-reconcile`, `just distributed-shard-lineage-bundle-history`.
- Handoff flow: `just distributed-shard-lineage-handoff`, `just distributed-shard-lineage-handoff-reconcile`, `just distributed-shard-lineage-handoff-history`, `just distributed-shard-lineage-handoff-reconciliation-history`.
- Locator flow: `just distributed-shard-lineage-locator-pointer`, `just distributed-shard-lineage-locator-history`, `just distributed-shard-lineage-transport-locator`, `just distributed-shard-lineage-transport-locator-reconcile`, `just distributed-shard-lineage-transport-locator-history`.
- `just workflow-surface-check-distributed-shard-lineage` guards the grouped lineage workflow map and recipe surface.
