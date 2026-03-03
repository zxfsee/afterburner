# Backlog

Parked items live here until promoted into the active TODO queue.

## Direction

- Target system shape is a contract-first end-to-end MLOps pipeline: `data -> train -> eval -> promote -> deploy -> observe -> rollback`.
- Promotion rule: by default, promote the next smallest unblocked item from the backlog into the active TODO horizon.

Rules:
- Keep this list priority-ranked within backlog.
- Every item must declare metadata: `Goal`, `Scope`, `Contracts`, `Boundary`, `Kind`, and `Blocked-by` when applicable.
- `Blocked-by` should reference exact active TODO/backlog item titles; include short inline context only when needed.
- Items must exist in exactly one place: active TODO queue or backlog (not both).
- Promotion into active TODO queue must preserve priority order and dependency constraints.

## Items

- Dataset contract hardening for scale readiness [Data Infra, Pre-training]
  - Goal: Define and gate dataset ingestion/lineage contracts (source identity, split semantics, checksums, schema versioning) so higher-throughput training can scale without data ambiguity.
  - Kind: `mixed`
  - Boundary: `core-contract`
  - Contracts: `artifact`, `event`
  - Scope: `src/data.rs`, `tests/pretraining_sample_contract.rs`, `tests/pretraining_sample_schema.rs`, `fixtures/pretraining_sample_metadata.schema.json`
  - Blocked-by: Training scalability-readiness contract (needs ingestion/throughput observability fields declared first).

- Artifact upload adapter contract [Serving/Deployment Infra, Runtime Infra]
  - Goal: Define a provider-agnostic artifact upload contract and gate, with optional Hugging Face adapter later, while keeping core runtime/storage-independent.
  - Kind: `mixed`
  - Boundary: `adapter-deployment`
  - Contracts: `artifact`, `ops`
  - Scope: `src/bin/`, `tests/`, `docs/adr/`, `ARCHITECTURE.md`
  - Blocked-by: Deploy-rs baseline deployment contract (upload authority/provenance model must be decided first).

- Backend performance profile gate [Kernels, Runtime Infra]
  - Goal: Add benchmark/acceptance gates that identify when `wgpu` is not the best runtime choice and define objective criteria for introducing a native backend.
  - Kind: `gate`
  - Boundary: `adapter-cli`
  - Contracts: `ops`
  - Scope: `justfile`, `tests/perf_backend_profile.rs`, `artifacts/eval/`, `README.md`
  - Blocked-by: Eval pipeline monitoring contract definition (requires stable duration/error metric fields).

- Custom kernel adoption threshold contract [Kernels, Numerics]
  - Goal: Define measurable thresholds and compatibility gates for custom kernel introduction so optimization work is evidence-driven and reversible.
  - Kind: `mixed`
  - Boundary: `core-contract`
  - Contracts: `artifact`
  - Scope: `src/train.rs`, `src/model.rs`, `tests/kernel_logits_shape.rs`, `fixtures/kernel_adoption_thresholds.json`, `docs/adr/`
  - Blocked-by: Backend performance profile gate (depends on established backend performance baselines).

- Burn TUI observability dashboard adapter [Experimentation/Eval Infra, Runtime Infra]
  - Goal: Add a lightweight dashboard adapter over existing structured events (without coupling core to UI runtime) for operator-facing eval/training monitoring.
  - Kind: `mixed`
  - Boundary: `adapter-cli`
  - Contracts: `event`
  - Scope: `src/bin/`, `tests/`, `README.md`
  - Blocked-by: Eval pipeline monitoring contract definition (needs stable RED + eval context event schema).

- OpenTelemetry semantic alignment gate [Experimentation/Eval Infra, Runtime Infra]
  - Goal: Align local eval/infer monitoring event names, units, and required fields with OpenTelemetry semantic conventions, without adding OTel SDK/exporter runtime dependencies yet.
  - Kind: `gate`
  - Boundary: `none`
  - Contracts: `event`
  - Scope: `fixtures/eval_pipeline_monitoring_event.json`, `tests/eval_cli.rs`, `tests/http_graceful_shutdown.rs`, `ARCHITECTURE.md`
  - Blocked-by: Eval pipeline monitoring contract definition (requires stable local event contract before semantic mapping).

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
  - Blocked-by: Eval pipeline monitoring contract definition (avoid structural churn while core monitoring contracts are still moving).

- Deploy-rs baseline deployment contract [Serving/Deployment Infra]
  - Goal: Introduce a minimal `serokell/deploy-rs` flake contract and validation gate so deployment wiring is explicit, testable, and adapter-scoped.
  - Kind: `mixed`
  - Boundary: `adapter-deployment`
  - Contracts: `artifact`, `ops`
  - Scope: `flake.nix`, `justfile`, `docs/adr/`, `ARCHITECTURE.md`
  - Blocked-by: HTTP infer success fixture gate; Eval pipeline monitoring artifact schema gate; deployment target profile model and artifact rollout ownership decision.
