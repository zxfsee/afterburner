# Backlog

Parked items live here until promoted into the active TODO queue.

## Direction

- Target system shape is a contract-first end-to-end MLOps pipeline: `data -> train -> eval -> promote -> deploy -> observe -> rollback`.
- This is a narrative model, not a priority rule. Active work is still driven by the runnable TODO queue and dependency unlocks.
- Promotion rule (default): promote the highest-priority runnable backlog item into the active TODO horizon.
- Burn-side distributed runtime work covers in-job execution semantics, topology, checkpoint/resume, and training state; scheduler/allocator work covers cross-job GPU placement, queueing, leases, and lifecycle, and the two tracks meet only at explicit lifecycle boundaries.
- Distributed-training runtime backlog items should follow workload-driven capability growth, not framework-parity work; use DeepSpeed-class systems as pattern references, not as parity targets.
- Model optimization and packaging are a separate leverage axis from distributed scale; treat quantization/compression/export/package work as post-training artifact work, not as distributed-runtime or scheduler work.

Rules:
- Keep this list priority-ranked within backlog.
- Every item must declare metadata: `Goal`, `Scope`, `Contracts`, `Boundary`, `Kind`, and `Blocked-by` when applicable.
- `Blocked-by` should reference exact active TODO/backlog item titles (or stable IDs/pointers if available); include short inline context only when needed.
- Items must exist in exactly one place: active TODO queue or backlog (not both).
- Promotion into active TODO queue must preserve priority order and dependency constraints.

## Items

- Protocol family surface consolidation [Runtime Infra, Serving/Deployment Infra]
  - Goal: Collapse repeated scheduler-heartbeat and deployment-verification variant families into smaller shared surfaces so new artifact/history/reconciliation variants stop multiplying modules, fixtures, and tests by noun swap.
  - Kind: `mixed`
  - Boundary: `core-contract`
  - Contracts: `artifact`, `cli`, `event`
  - Scope: `src/`, `fixtures/`, `tests/`, `justfile`, `docs/reference.md`, `docs/workflows.md`

- CLI and recipe dispatch decomposition [Runtime Infra, Serving/Deployment Infra]
  - Goal: Reduce semantic concentration in `src/main.rs` and `justfile` by extracting grouped dispatch/orchestration helpers so they stop acting as the permanent control-plane choke points.
  - Kind: `mixed`
  - Boundary: `none`
  - Contracts: `cli`
  - Scope: `src/main.rs`, `src/bin/`, `justfile`, `tests/`

- Reference and workflow surface consolidation [Runtime Infra]
  - Goal: Shrink `docs/reference.md` and `docs/workflows.md` from long artifact registries into grouped operator/index surfaces that stay useful without carrying every variant explicitly.
  - Kind: `mixed`
  - Boundary: `repo-workflow`
  - Contracts: `docs`
  - Scope: `docs/reference.md`, `docs/workflows.md`, `tests/`

- GitHub repo settings polish [Runtime Infra, Serving/Deployment Infra]
  - Goal: Apply the GitHub repository description/topics/website settings directly or from one small maintenance note without growing README, reference-index, or docs-only string-presence test surface.
  - Kind: `mixed`
  - Boundary: `repo-workflow`
  - Contracts: `none`
  - Scope: `unknown`

- Doc-test admission snapshot burn-down [Runtime Infra]
  - Goal: Shrink the transitional doc-test admission snapshot files in bounded cleanup batches so the allowlist and legacy exception sets do not ossify into permanent policy.
  - Kind: `mixed`
  - Boundary: `repo-workflow`
  - Contracts: `docs`
  - Scope: `tests/`

- Doc-test admission rule note [Runtime Infra]
  - Goal: Add one short repo policy note for the doc-test admission guard only if future review or agent behavior shows the guard alone is too opaque to apply consistently.
  - Kind: `mixed`
  - Boundary: `repo-workflow`
  - Contracts: `docs`
  - Scope: `docs/`
  - Blocked-by: Evidence that the existing doc-test admission guard is insufficient on its own
