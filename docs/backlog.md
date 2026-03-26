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

- Cooperative checkpoint preemption fit gate [Runtime Infra, Serving/Deployment Infra]
  - Goal: Define the scheduler-side contract for cooperative checkpoint/resume preemption so urgent jobs can interrupt lower-priority work only at safe runtime checkpoint boundaries instead of assuming transparent GPU task suspension.
  - Kind: `gate`
  - Boundary: `adapter-deployment`
  - Contracts: `ops`, `event`
  - Scope: `docs/adr/`, `README.md`, `tests/`
  - Blocked-by:
    - GPU scheduler boundary and lifecycle gate.
    - Burn distributed optimizer and checkpoint state contract.

- kube-rs GPU scheduler placement fit gate [Runtime Infra, Serving/Deployment Infra]
  - Goal: Define a kube-rs-compatible control-plane contract for node inventory, topology-aware GPU placement, queue admission, and job lifecycle reconciliation so the later cluster path can reuse the same lease and state-boundary model as the single-node scheduler.
  - Kind: `gate`
  - Boundary: `adapter-deployment`
  - Contracts: `ops`
  - Scope: `docs/adr/`, `README.md`, `tests/`
  - Blocked-by:
    - GPU scheduler boundary and lifecycle gate.
    - Single-node GPU lease and systemd scheduler adapter.

- Burn `.bpk` artifact migration contract [Frameworks, Runtime Infra]
  - Goal: Migrate the repo's inference artifact contract from `.mpk` to `.bpk` only after a pinned Burn refresh confirms the target APIs and the repo is ready to cut over docs, fixtures, CLI paths, and event payloads together.
  - Kind: `mixed`
  - Boundary: `core-contract`
  - Contracts: `artifact`, `cli`, `event`
  - Scope: `Cargo.toml`, `src/`, `fixtures/`, `tests/`, `README.md`, `ARCHITECTURE.md`, `docs/adr/`
  - Blocked-by: Newer stable Burn release after `0.20.1`; see ADR-033.

- Quantized and compressed model capability surface gate [Runtime Infra, Frameworks]
  - Goal: Define the supported versus unsupported reduced-precision and compression modes for local-first optimized models so the repo extends the current explicit quantization contract deliberately instead of accepting ad hoc optimized artifacts.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `artifact`, `cli`, `event`
  - Scope: `docs/adr/`, `README.md`, `tests/`, `fixtures/`
  - Blocked-by: Model optimization and packaging fit gate.

- Optimized model packaging and export contract [Runtime Infra, Serving/Deployment Infra]
  - Goal: Define the artifact shape, metadata, export format, and packaging inputs for quantized or compressed models so optimized releases can be reproduced, audited, and handed off without overloading the base training checkpoint contract.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `artifact`, `cli`
  - Scope: `docs/adr/`, `README.md`, `tests/`, `fixtures/`
  - Blocked-by:
    - Model optimization and packaging fit gate.
    - Quantized and compressed model capability surface gate.

- Optimized model eval and local-profile contract [Experimentation/Eval Infra, Runtime Infra]
  - Goal: Materialize the quality, latency, memory, and size acceptance surface for optimized models on constrained hardware so quantization or compression choices are driven by measured tradeoffs instead of release pressure alone.
  - Kind: `mixed`
  - Boundary: `adapter-cli`
  - Contracts: `artifact`, `event`, `ops`
  - Scope: `src/`, `fixtures/`, `tests/`, `README.md`, `justfile`
  - Blocked-by:
    - Quantized and compressed model capability surface gate.
    - Optimized model packaging and export contract.

- Optimized model publish adapter [Serving/Deployment Infra, Runtime Infra]
  - Goal: Extend publishing workflows so approved optimized model artifacts and their packaging metadata can be released to downstream destinations such as Hugging Face without inventing a second artifact identity or approval path.
  - Kind: `mixed`
  - Boundary: `adapter-deployment`
  - Contracts: `artifact`, `cli`, `ops`, `event`
  - Scope: `src/`, `fixtures/`, `tests/`, `README.md`, `justfile`, `docs/adr/`
  - Blocked-by:
    - Optimized model packaging and export contract.
    - Hugging Face model publish adapter.

- Tokio adapter runtime fit gate [Serving/Deployment Infra, Runtime Infra]
  - Goal: Evaluate whether adapter-side concurrency, tracing, and deployment pressure justify adopting a Tokio-based runtime in adapters so the repo can grow into heavier async workflows without changing core/runtime boundaries prematurely.
  - Kind: `gate`
  - Boundary: `adapter-deployment`
  - Contracts: `none`
  - Scope: `docs/adr/`, `README.md`, `tests/`
  - Blocked-by:
    - Distributed tracing correlation contract.
    - HTTP infer load profile contract.

