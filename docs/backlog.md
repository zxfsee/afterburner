# Backlog

Parked items live here until promoted into the active TODO queue.

## Direction

- Target system shape is a contract-first end-to-end MLOps pipeline: `data -> train -> eval -> promote -> deploy -> observe -> rollback`.
- This is a narrative model, not a priority rule. Active work is still driven by the runnable TODO queue and dependency unlocks.
- Promotion rule (default): promote the highest-priority runnable backlog item into the active TODO horizon.
- Burn-side distributed runtime work covers in-job execution semantics, topology, checkpoint/resume, and training state; scheduler/allocator work covers cross-job GPU placement, queueing, leases, and lifecycle, and the two tracks meet only at explicit lifecycle boundaries.

Rules:
- Keep this list priority-ranked within backlog.
- Every item must declare metadata: `Goal`, `Scope`, `Contracts`, `Boundary`, `Kind`, and `Blocked-by` when applicable.
- `Blocked-by` should reference exact active TODO/backlog item titles (or stable IDs/pointers if available); include short inline context only when needed.
- Items must exist in exactly one place: active TODO queue or backlog (not both).
- Promotion into active TODO queue must preserve priority order and dependency constraints.

## Items

- Burn distributed runtime layout feasibility gate [Distributed Training, Runtime Infra]
  - Goal: Define feasibility validation for candidate Burn-side DP/TP/PP/GAS/MBS/ZeRO layouts on 8-GPU nodes so invalid divisibility, topology, and memory combinations are rejected before any benchmark or training launch.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `ops`
  - Scope: `docs/adr/`, `README.md`, `tests/`
  - Blocked-by: Burn distributed runtime capability surface gate.

- Burn distributed runtime benchmark harness contract [Distributed Training, Experimentation/Eval Infra]
  - Goal: Materialize a reproducible benchmark runner for Burn distributed runtime profile trials so candidate configurations can be executed with seeded config, fixed measurement windows, stable metrics capture, and artifact persistence instead of ad hoc scripts.
  - Kind: `mixed`
  - Boundary: `adapter-cli`
  - Contracts: `artifact`, `ops`
  - Scope: `src/`, `fixtures/`, `tests/`, `README.md`, `justfile`
  - Blocked-by:
    - Burn distributed runtime capability surface gate.
    - Burn distributed runtime profile schema gate.
    - Burn distributed runtime layout feasibility gate.

- Burn distributed runtime profile contract [Distributed Training, Experimentation/Eval Infra]
  - Goal: Materialize a reproducible scaling-profile artifact over model size and node count from executed Burn distributed runtime profile trials so parallelization choices come from measured profiles instead of ad hoc tuning.
  - Kind: `mixed`
  - Boundary: `adapter-cli`
  - Contracts: `artifact`, `ops`
  - Scope: `src/`, `fixtures/`, `tests/`, `README.md`, `justfile`
  - Blocked-by: Burn distributed runtime benchmark harness contract.

- GPU scheduler boundary and lifecycle gate [Runtime Infra, Serving/Deployment Infra]
  - Goal: Define the scheduler/allocator boundary for queueing, admission, lease ownership, topology-aware GPU placement, and job lifecycle control so cluster scheduling stays separate from Burn runtime state and touches training only through explicit `START`/`STOP`/`KILL` and `READY`/`CHECKPOINTED`/`FAILED`/`HEARTBEAT` boundaries.
  - Kind: `gate`
  - Boundary: `adapter-deployment`
  - Contracts: `ops`
  - Scope: `docs/adr/`, `README.md`, `tests/`

- Single-node GPU lease and systemd scheduler adapter [Runtime Infra, Serving/Deployment Infra]
  - Goal: Materialize a systemd-first single-node scheduler path that tracks GPU inventory, queue admission, lease ownership, and graceful job stop/resume against the scheduler lifecycle contract before any cluster controller exists.
  - Kind: `mixed`
  - Boundary: `adapter-deployment`
  - Contracts: `ops`, `event`
  - Scope: `src/`, `fixtures/`, `tests/`, `README.md`, `justfile`
  - Blocked-by: GPU scheduler boundary and lifecycle gate.

- kube-rs GPU scheduler placement fit gate [Runtime Infra, Serving/Deployment Infra]
  - Goal: Define a kube-rs-compatible control-plane contract for node inventory, topology-aware GPU placement, queue admission, and job lifecycle reconciliation so the later cluster path can reuse the same lease and state-boundary model as the single-node scheduler.
  - Kind: `gate`
  - Boundary: `adapter-deployment`
  - Contracts: `ops`
  - Scope: `docs/adr/`, `README.md`, `tests/`
  - Blocked-by:
    - GPU scheduler boundary and lifecycle gate.
    - Single-node GPU lease and systemd scheduler adapter.

- CLI subcommand hierarchy migration contract [Runtime Infra, Serving/Deployment Infra]
  - Goal: Collapse the flat `afterburner` command namespace into grouped subcommands where it improves typical operator use, and cut over docs, tests, and workflow recipes in one explicit CLI contract change instead of accreting more top-level verbs.
  - Kind: `mixed`
  - Boundary: `adapter-cli`
  - Contracts: `cli`
  - Scope: `src/`, `tests/`, `README.md`, `justfile`, `docs/adr/`

- Remote model save/load fit gate [Serving/Deployment Infra, Runtime Infra]
  - Goal: Define whether Afterburner should support remote model artifact save/load beyond local filesystem paths, and if so, keep the contract adapter-first so artifact resolution and transfer do not couple core logic to one storage backend or SDK.
  - Kind: `gate`
  - Boundary: `adapter-deployment`
  - Contracts: `artifact`, `cli`, `ops`
  - Scope: `docs/adr/`, `README.md`, `tests/`, `src/`

- Burn `.bpk` artifact migration contract [Frameworks, Runtime Infra]
  - Goal: Migrate the repo's inference artifact contract from `.mpk` to `.bpk` only after a pinned Burn refresh confirms the target APIs and the repo is ready to cut over docs, fixtures, CLI paths, and event payloads together.
  - Kind: `mixed`
  - Boundary: `core-contract`
  - Contracts: `artifact`, `cli`, `event`
  - Scope: `Cargo.toml`, `src/`, `fixtures/`, `tests/`, `README.md`, `ARCHITECTURE.md`, `docs/adr/`

- HTTP infer load profile contract [Serving/Deployment Infra, Experimentation/Eval Infra]
  - Goal: Materialize a reproducible HTTP inference load profile and results artifact so deployment decisions can be checked against concurrency, latency, and error budgets under sustained load instead of only single-request fixtures.
  - Kind: `mixed`
  - Boundary: `adapter-http`
  - Contracts: `artifact`, `ops`
  - Scope: `src/bin/afterburner_http.rs`, `tests/`, `fixtures/`, `README.md`, `justfile`

- Artifact cleanup execution evidence provenance gate [Serving/Deployment Infra, Runtime Infra]
  - Goal: Define the minimum evidence-reference fields for cleanup execution receipts so later cleanup reviews can trace each removal or skip back to the dry-run inputs and observed artifacts without raw logs.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `artifact`, `ops`
  - Scope: `docs/adr/`, `ARCHITECTURE.md`, `README.md`, `tests/`

- Profiling provenance evidence bundle gate [Runtime Infra, Experimentation/Eval Infra]
  - Goal: Define the minimum evidence-bundle fields that profiling provenance receipts should eventually package so later profiling-side tooling can move provenance evidence without rewalking raw environment outputs and workflow logs ad hoc.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `artifact`
  - Scope: `docs/adr/`, `ARCHITECTURE.md`, `README.md`, `tests/`

- Deployment verification evidence bundle handoff gate [Serving/Deployment Infra, Experimentation/Eval Infra]
  - Goal: Define the minimum handoff contract for deployment verification evidence bundles so downstream deployment-side consumers can resolve one stable bundle entrypoint without depending on surrounding rollout workspace layout.
  - Kind: `gate`
  - Boundary: `adapter-deployment`
  - Contracts: `artifact`, `event`
  - Scope: `docs/adr/`, `ARCHITECTURE.md`, `README.md`, `tests/`

- Pretraining source provenance evidence bundle gate [Data Infra, Pre-training]
  - Goal: Define the minimum evidence-bundle fields that pretraining source provenance receipts should eventually package so later dataset and registry tooling can move approval provenance evidence without rewalking source metadata and review logs ad hoc.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `artifact`
  - Scope: `docs/adr/`, `ARCHITECTURE.md`, `README.md`, `tests/`

- Distributed shard lineage evidence bundle handoff gate [Distributed Training, Data Infra]
  - Goal: Define the minimum handoff contract for distributed shard lineage evidence bundles so downstream distributed tooling can resolve one stable bundle entrypoint without depending on surrounding checkpoint workspace layout.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `artifact`
  - Scope: `docs/adr/`, `ARCHITECTURE.md`, `README.md`, `tests/`


- JSON to RON format fit investigation gate [Data Infra, Runtime Infra]
  - Goal: Evaluate whether any current JSON-based artifact or event surfaces should migrate to RON, and whether the readability or ergonomics gains would justify the contract churn and tooling impact.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `artifact`, `event`
  - Scope: `docs/adr/`, `README.md`, `tests/`

- Burn distributed optimizer and checkpoint state contract [Distributed Training, Runtime Infra]
  - Goal: Define the minimum in-job optimizer-state and checkpoint-group contract needed for distributed recovery so future Burn-side multi-device training can resume consistently across shards and ranks without coupling cluster scheduling into checkpoint semantics.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `artifact`
  - Scope: `docs/adr/`, `README.md`, `tests/`
