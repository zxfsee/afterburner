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

- Artifact upload adapter contract [Serving/Deployment Infra, Runtime Infra]
  - Goal: Define a provider-agnostic artifact upload contract and gate, with optional Hugging Face adapter later, while keeping core runtime/storage-independent.
  - Kind: `mixed`
  - Boundary: `adapter-deployment`
  - Contracts: `artifact`, `ops`
  - Scope: `src/bin/`, `tests/`, `docs/adr/`, `ARCHITECTURE.md`
  - Blocked-by: Deploy-rs baseline deployment contract.

- Promotion/rollback orchestration gate [Serving/Deployment Infra, Experimentation/Eval Infra]
  - Goal: Define and validate deterministic promotion/rollback orchestration (`train -> eval -> promote -> deploy -> verify -> rollback`) with explicit pass/fail evidence at each step.
  - Kind: `mixed`
  - Boundary: `adapter-deployment`
  - Contracts: `artifact`, `ops`, `event`
  - Scope: `justfile`, `tests/`, `docs/adr/`, `ARCHITECTURE.md`
  - Blocked-by: Deploy-rs baseline deployment contract; Artifact upload adapter contract.

- OpenTelemetry profiling correlation investigation gate [Runtime Infra, Frameworks]
  - Goal: Evaluate whether profiling artifacts should correlate with OpenTelemetry-style trace/resource metadata without introducing a heavy telemetry SDK or runtime prematurely.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `artifact`, `event`
  - Scope: `docs/adr/`, `ARCHITECTURE.md`, `README.md`, `tests/`
  - Blocked-by: Unified profiling dashboard adapter.

- Multibillion-scale system target envelope gate [Distributed Training, Inference, Serving/Deployment Infra]
  - Goal: Define the minimum training, inference, and deployment constraints for eventual multibillion-parameter support so sharding, checkpointing, precision, and rollout contracts evolve intentionally instead of piecemeal.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `artifact`, `ops`
  - Scope: `docs/adr/`, `ARCHITECTURE.md`, `README.md`, `tests/`
  - Blocked-by: Deploy-rs baseline deployment contract; Arrow/DataFusion/Ballista/Parquet fit investigation gate.
