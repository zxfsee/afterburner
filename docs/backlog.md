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

- Burn `.bpk` artifact migration contract [Frameworks, Runtime Infra]
  - Goal: Migrate the repo's inference artifact contract from `.mpk` to `.bpk` only after a pinned Burn refresh confirms the target APIs and the repo is ready to cut over docs, fixtures, CLI paths, and event payloads together.
  - Kind: `mixed`
  - Boundary: `core-contract`
  - Contracts: `artifact`, `cli`, `event`
  - Scope: `Cargo.toml`, `src/`, `fixtures/`, `tests/`, `README.md`, `ARCHITECTURE.md`, `docs/adr/`
  - Blocked-by: Newer stable Burn release after `0.20.1`; see ADR-033.

- Deterministic scheduler/runtime simulation harness [Distributed Training, Runtime Infra, Experimentation/Eval Infra]
  - Goal: Add one deterministic event-driven simulation harness plus explicit artifacts/receipts for scheduler lifecycle, lease transitions, topology/layout feasibility, checkpoint/restart state transitions, and artifact emission so policy and state-machine bugs can be validated without pretending to simulate real GPU or NCCL timing.
  - Kind: `mixed`
  - Boundary: `runtime-scheduler`
  - Contracts: `artifact`, `event`, `ops`
  - Scope: `src/`, `tests/`, `fixtures/`, `justfile`, `docs/reference.md`, `ARCHITECTURE.md`, `docs/adr/`
