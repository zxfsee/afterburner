# Backlog

Parked items live here until promoted into the active TODO queue.

## Direction

- Target system shape is a contract-first end-to-end MLOps pipeline: `data -> train -> eval -> promote -> deploy -> observe -> rollback`.
- This is a narrative model, not a priority rule. Active work is still driven by the runnable TODO queue and dependency unlocks.
- Promotion rule (default): promote the highest-priority runnable backlog item into the active TODO horizon.

Rules:
- Keep this list priority-ranked within backlog.
- Every item must declare metadata: `Goal`, `Scope`, `Contracts`, `Boundary`, `Kind`, and `Blocked-by` when applicable.
- `Blocked-by` should reference exact active TODO/backlog item titles (or stable IDs/pointers if available); include short inline context only when needed.
- Items must exist in exactly one place: active TODO queue or backlog (not both).
- Promotion into active TODO queue must preserve priority order and dependency constraints.

## Items

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

- Collective synchronization boundary gate [Distributed Training, Frameworks]
  - Goal: Define where gradient synchronization and collective communication live relative to core training logic so future Burn collective adoption does not leak backend/runtime details across core boundaries.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `none`
  - Scope: `docs/adr/`, `ARCHITECTURE.md`, `tests/`

- Distributed optimizer and checkpoint state contract [Distributed Training, Runtime Infra]
  - Goal: Define the minimum optimizer-state and checkpoint-group contract needed for distributed recovery so future multi-device training can resume consistently across shards and ranks.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `artifact`
  - Scope: `docs/adr/`, `README.md`, `tests/`
