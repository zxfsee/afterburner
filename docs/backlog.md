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

- Single-node GPU lease and systemd scheduler adapter [Runtime Infra, Serving/Deployment Infra]
  - Goal: Materialize a systemd-first single-node scheduler path that tracks GPU inventory, queue admission, lease ownership, and graceful job stop/resume against the scheduler lifecycle contract before any cluster controller exists.
  - Kind: `mixed`
  - Boundary: `adapter-deployment`
  - Contracts: `ops`, `event`
  - Scope: `src/`, `fixtures/`, `tests/`, `README.md`, `justfile`
  - Blocked-by: GPU scheduler boundary and lifecycle gate.

- Cooperative checkpoint preemption fit gate [Runtime Infra, Serving/Deployment Infra]
  - Goal: Define the scheduler-side contract for cooperative checkpoint/resume preemption so urgent jobs can interrupt lower-priority work only at safe runtime checkpoint boundaries instead of assuming transparent GPU task suspension.
  - Kind: `gate`
  - Boundary: `adapter-deployment`
  - Contracts: `ops`, `event`
  - Scope: `docs/adr/`, `README.md`, `tests/`
  - Blocked-by:
    - GPU scheduler boundary and lifecycle gate.
    - Burn distributed optimizer and checkpoint state contract.

- GPU colocation fit gate [Runtime Infra, Serving/Deployment Infra]
  - Goal: Define whether selected low-saturation jobs may share one GPU under explicit compatibility and interference limits so utilization improvements stay separate from the scheduler's preemption model.
  - Kind: `gate`
  - Boundary: `adapter-deployment`
  - Contracts: `ops`
  - Scope: `docs/adr/`, `README.md`, `tests/`
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

- FineWeb-Edu slice source adoption gate [Pre-training, Data Infra]
  - Goal: Define the approved FineWeb-Edu slice source key, `source_revision`, license/usage envelope, shard/checksum expectations, and local size budget so MacBook-scale text corpus selection is explicit and reproducible instead of a one-off download choice.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `artifact`
  - Scope: `docs/adr/`, `README.md`, `tests/`, `fixtures/`
  - Blocked-by: MacBook text pretraining fit gate.

- Text tokenizer and sequence-packing contract [Pre-training, Data Infra, Runtime Infra]
  - Goal: Define tokenizer identity, vocabulary/versioning, special tokens, sequence length, truncation/packing policy, and text-sample manifest fields so MacBook-scale text batches do not depend on ad hoc preprocessing scripts or unstated tokenizer drift.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `artifact`, `ops`
  - Scope: `docs/adr/`, `README.md`, `tests/`, `fixtures/`
  - Blocked-by:
    - MacBook text pretraining fit gate.
    - FineWeb-Edu slice source adoption gate.

- Text model artifact and inference contract gate [Pre-training, Runtime Infra, Serving/Deployment Infra]
  - Goal: Define how a text-trained artifact is versioned, loaded, and sampled so decoder-only language-model outputs do not get forced through the current MNIST-oriented infer/eval surface when the repo starts training on text.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `artifact`, `cli`, `event`
  - Scope: `docs/adr/`, `README.md`, `tests/`, `fixtures/`
  - Blocked-by:
    - MacBook text pretraining fit gate.
    - Text tokenizer and sequence-packing contract.

- MacBook text pretraining adapter [Pre-training, Frameworks, Runtime Infra]
  - Goal: Materialize a small decoder-only language-model training path over the approved FineWeb-Edu slice with deterministic local caching, checkpoint/resume, and a bounded training recipe that fits a single MacBook while MNIST remains available as a separate smoke workflow.
  - Kind: `mixed`
  - Boundary: `adapter-cli`
  - Contracts: `artifact`, `cli`, `event`, `ops`
  - Scope: `src/`, `fixtures/`, `tests/`, `README.md`, `justfile`, `docs/adr/`
  - Blocked-by:
    - FineWeb-Edu slice source adoption gate.
    - Text tokenizer and sequence-packing contract.
    - Text model artifact and inference contract gate.

- Text pretraining eval and smoke gate [Pre-training, Experimentation/Eval Infra]
  - Goal: Add a deterministic text-side acceptance surface over loss/perplexity or a tiny overfit fixture so the new language-model path has a real regression gate while MNIST remains the cheapest end-to-end runtime sanity check.
  - Kind: `mixed`
  - Boundary: `adapter-cli`
  - Contracts: `artifact`, `event`, `ops`
  - Scope: `src/`, `fixtures/`, `tests/`, `README.md`, `justfile`
  - Blocked-by: MacBook text pretraining adapter.

- Model optimization and packaging fit gate [Runtime Infra, Serving/Deployment Infra, Experimentation/Eval Infra]
  - Goal: Define the post-training optimization surface for quantization, compression, export, and packaging so efficiency work for constrained hardware stays a separate artifact pipeline from distributed runtime scale-out work.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `artifact`, `ops`
  - Scope: `docs/adr/`, `README.md`, `tests/`

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

- Remote model save/load fit gate [Serving/Deployment Infra, Runtime Infra]
  - Goal: Define whether Afterburner should support remote model artifact save/load beyond local filesystem paths, and if so, keep the contract adapter-first so artifact resolution and transfer do not couple core logic to one storage backend or SDK.
  - Kind: `gate`
  - Boundary: `adapter-deployment`
  - Contracts: `artifact`, `cli`, `ops`
  - Scope: `docs/adr/`, `README.md`, `tests/`, `src/`

- Hugging Face model publish adapter [Serving/Deployment Infra, Runtime Infra]
  - Goal: Materialize a Hugging Face publishing adapter that consumes the existing provider-neutral `artifact_upload_request.json` contract and pushes approved model artifacts plus required metadata to Hugging Face without collapsing upload approval, artifact identity, or rollout vocabulary into vendor-specific CLI semantics.
  - Kind: `mixed`
  - Boundary: `adapter-deployment`
  - Contracts: `artifact`, `cli`, `ops`, `event`
  - Scope: `src/`, `fixtures/`, `tests/`, `README.md`, `justfile`, `docs/adr/`
  - Blocked-by: Remote model save/load fit gate.

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
