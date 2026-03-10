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
  - Blocked-by: Deploy-rs baseline deployment contract (upload authority/provenance model must be decided first).

- Custom kernel adoption threshold contract [Kernels, Numerics]
  - Goal: Define measurable thresholds and compatibility gates for custom kernel introduction so optimization work is evidence-driven and reversible.
  - Kind: `mixed`
  - Boundary: `core-contract`
  - Contracts: `artifact`
  - Scope: `src/train.rs`, `src/model.rs`, `tests/kernel_logits_shape.rs`, `fixtures/kernel_adoption_thresholds.json`, `docs/adr/`
  - Blocked-by: Backend performance profile gate (depends on established backend performance baselines).

- Async/runtime decision record (Tokio ecosystem) [Runtime Infra, Frameworks]
  - Goal: Decide if async runtime adoption is required for adapters and document constraints/trade-offs before introducing Tokio-dependent code.
  - Kind: `behavior`
  - Boundary: `adapter-http`
  - Contracts: `none`
  - Scope: `docs/adr/`, `ARCHITECTURE.md`
  - Blocked-by: Backend performance profile gate (runtime decision should be evidence-based, not assumption-based).

- Cargo workspace boundary split + dependency gate [Frameworks, Runtime Infra]
  - Goal: Split into Cargo workspace boundaries (`core` and adapters) and add an enforceable dependency gate so codebase growth remains maintainable and CI can target crates.
  - Kind: `mixed`
  - Boundary: `core-contract`
  - Contracts: `none`
  - Scope: `Cargo.toml`, `crates/`, `tests/`, `justfile`, `ARCHITECTURE.md`

- Deploy-rs baseline deployment contract [Serving/Deployment Infra]
  - Goal: Introduce a minimal `serokell/deploy-rs` flake contract and validation gate so deployment wiring is explicit, testable, and adapter-scoped.
  - Kind: `mixed`
  - Boundary: `adapter-deployment`
  - Contracts: `artifact`, `ops`
  - Scope: `flake.nix`, `justfile`, `docs/adr/`, `ARCHITECTURE.md`
  - Blocked-by: HTTP infer success fixture gate; deployment target profile model and artifact rollout ownership decision.

- Promotion/rollback orchestration gate [Serving/Deployment Infra, Experimentation/Eval Infra]
  - Goal: Define and validate deterministic promotion/rollback orchestration (`train -> eval -> promote -> deploy -> verify -> rollback`) with explicit pass/fail evidence at each step.
  - Kind: `mixed`
  - Boundary: `adapter-deployment`
  - Contracts: `artifact`, `ops`, `event`
  - Scope: `justfile`, `tests/`, `docs/adr/`, `ARCHITECTURE.md`
  - Blocked-by: Deploy-rs baseline deployment contract; Artifact upload adapter contract.
