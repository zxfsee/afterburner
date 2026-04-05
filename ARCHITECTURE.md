# Architecture Overview

This system models a minimal ML production loop:

Training -> Artifact -> Inference

## High-level flow

[ `afterburner train` ]
        |
        v
[ artifacts/inference/<version>/model.mpk ]
        |
        v
[ artifacts/inference/<version>/manifest.toml ]
        ^
        |
[ artifacts/inference/current ]
        |
        v
[ `afterburner infer` ]
        |
        v
[ Optional HTTP Wrapper ]

Training produces artifacts.
Inference consumes artifacts via the CLI and the optional HTTP wrapper.
No runtime coupling exists between training and inference.
The inference manifest includes checksum validation, explicit precision metadata,
signing-ready placeholders, and canonicalization metadata.

Training also emits JSONL observability events under `artifacts/train/` to keep
metrics and runtime metadata inspectable without introducing a logging stack.
Training-side contract artifacts also capture optimization decision guards
separately from runtime contracts. For example, custom kernel adoption is gated
by a checked contract that records the current convolution footprint plus the
objective evidence required before replacing backend-provided kernels.
The train module should stay split by responsibility: runtime orchestration,
artifact export, distributed metadata validation, observability emission, and
contract builders should live in separate train-side modules rather than being
collapsed into one file.

## Distributed Training Layers

The distributed training stack is split into three layers with explicit
responsibility boundaries:

```text
┌──────────────────────────────────────────────────────────────┐
│                   1) Distributed Training Runtime            │
│                        (Burn-side layer)                     │
│                                                              │
│  Responsibilities:                                           │
│  - DP (today)                                                │
│  - Future: ZeRO / TP / PP / SP/CP / EP                       │
│  - Rank/world topology                                       │
│  - Process groups / collectives                              │
│  - Gradient sync / communication                             │
│  - Checkpoint / optimizer state                              │
│  - Microbatch / pipeline execution                           │
│                                                              │
│  Does NOT do:                                                │
│  - cluster scheduling                                        │
│  - GPU placement across jobs                                 │
└──────────────────────────────────────────────────────────────┘
                           ▲
                           │ lifecycle boundary (explicit)
                           │
                           │ START / STOP / KILL
                           │ READY / CHECKPOINTED / FAILED / HEARTBEAT
                           │
┌──────────────────────────────────────────────────────────────┐
│              2) GPU Scheduler / Allocator (Control Plane)    │
│                                                              │
│  Responsibilities:                                           │
│  - Queueing / admission                                      │
│  - GPU placement (topology-aware)                            │
│  - Node inventory                                            │
│  - Lease ownership                                           │
│  - Job lifecycle                                             │
│  - Preemption policy                                         │
│                                                              │
│  Does NOT do:                                                │
│  - DP / TP / PP / ZeRO                                       │
│  - model execution                                           │
│  - gradient synchronization                                  │
└──────────────────────────────────────────────────────────────┘
                           ▲
                           │ deployment / execution substrate
                           │
┌──────────────────────────────────────────────────────────────┐
│                 3) Platform / Infra Layer                    │
│                      (NixOS + K8s/systemd)                   │
│                                                              │
│  Responsibilities:                                           │
│  - Reproducible environments                                 │
│  - CUDA / drivers / NCCL                                     │
│  - Node provisioning                                         │
│  - systemd (single-node)                                     │
│  - kube-rs / K8s (cluster path)                              │
│                                                              │
│  Does NOT do:                                                │
│  - scheduling policy logic                                   │
│  - training semantics                                        │
└──────────────────────────────────────────────────────────────┘
```

1. Distributed training runtime
   - Burn-side execution semantics inside one job.
   - Owns data/model parallel execution, collectives, rank/world topology,
     checkpoint and optimizer state, and future DP/TP/PP/ZeRO-style expansion.
2. GPU scheduler / allocator
   - Control-plane policy across jobs.
   - Owns queueing, admission, topology-aware GPU placement, node inventory,
     lease ownership, and job lifecycle.
3. Platform / infrastructure substrate
   - Execution substrate for the scheduler and runtime.
   - Owns reproducible environments, drivers/runtime libraries, node
     provisioning, and the concrete launch surfaces such as systemd, NixOS, and
     Kubernetes.

The runtime and scheduler meet only at an explicit lifecycle boundary:

- Scheduler to runtime: `START`, `STOP`, `KILL`
- Runtime to scheduler: `READY`, `CHECKPOINTED`, `FAILED`, `HEARTBEAT`

The scheduler may assign resources, nodes, GPU ids, and rank assignments, but
it must not own model-state semantics or internal parallelism choices. The
runtime may choose internal composition such as DP, TP, PP, SP/CP, EP, or ZeRO
variants, but it must not absorb cluster-level scheduling policy.

The scheduler/runtime lifecycle message contract should also stay explicit:

- `direction`
- `kind`
- `job_id`
- `lease_id`
- `resources` for `START`, including `node_ids`, `gpu_ids`, and `rank_assignments`

Distributed-training scope is workload-driven. The repo is not targeting
framework parity with DeepSpeed-class systems; it is implementing the minimum
capability subset required by real workloads, with explicit contracts,
profiling, and dependency order.

### Trade-offs

This design prioritizes explicit boundaries and reproducibility over rapid iteration.
As a result, some conveniences common in ML prototypes (implicit preprocessing,
auto-versioning, embedded serving) are intentionally absent.

## Invariants

### Core boundaries

- Adapters depend on core; core must not depend on adapters.
  - “Core” = library modules implementing artifact contract + preprocessing + inference logic.
  - “Adapters” = binaries/transport layers (CLI, HTTP wrapper) and any integration glue.
  - Any new adapter must call a stable core API; do not import adapter modules from core.
  - The current mechanical boundary is a Cargo workspace split:
    `crates/afterburner-core` holds shared library modules, while the root `afterburner`
    package is the adapter package that depends on and re-exports core modules.
- Public contracts (artifact format, CLI surface, HTTP surface, event schema) must be:
  - explicit and versioned when needed,
  - validated at boundaries,
  - protected by compatibility tests (e.g. golden fixtures).
- Structured artifact and event contracts stay on JSON by default, while the
  inference manifest remains TOML. No broad JSON-to-RON migration should happen
  without a new explicit decision.
- `capnproto-rust` is not part of the current artifact or event contract stack.
  Reconsider it only for a concrete later high-volume runtime or data surface if
  measured pressure shows JSON/TOML framing is insufficient.
- Inference artifact precision is explicit in the manifest. The current runtime accepts only
  `f32` weights/activations with `quantization = none`; any reduced-precision artifact must
  declare itself explicitly and remain rejected until a dedicated runtime path is introduced.
- No async runtime is adopted. The current HTTP adapter remains synchronous and `tiny_http`-based
  until profiling or deployment requirements show that a Tokio-class runtime is necessary.

### Deployment and rollout contracts

- Deployment adapters must resolve through an explicit deployment target profile contract before
  introducing deploy tooling. Target host, user, system, artifact root, and activation strategy
  are contract data, not ad-hoc shell configuration.
- Deployment adapters must also resolve through an explicit deployment stack contract.
  - The deployable unit is the Afterburner runtime stack, not just the model file.
  - Service entrypoint, service surface, artifact roots, rollout entrypoint, and observability
    hooks are contract data, not implied by recipes or host-local layout.
  - Launch planning should also resolve target and stack profiles into one explicit launch artifact
    instead of relying on shell-local startup assumptions.
- The baseline deploy-rs adapter consumes the deployment target profile contract directly from the
  checked fixture/example and uses `activate.custom` against the packaged `afterburner` binary
  rather than inventing a NixOS system deployment surface.
- The Gateway API inference extension is not part of the current deployment or
  cluster-serving boundary; reconsider it only as a later traffic-management
  reference or thin adapter target over the existing deployment stack.
- Arrow/DataFusion/Ballista are not part of the current runtime or artifact contract. The present
  fit decision is: prefer Parquet later for offline tabular dataset/eval/export artifacts if a
  measured need appears; consider DataFusion only for a real in-process analytical layer; keep
  Ballista out of scope until distributed query/ETL pressure is concrete.
- Deployment adapters must also consume an explicit artifact rollout ownership contract before
  upload, promotion, or deployment. Artifact provenance plus `rollout_owner`, `approved_by`,
  `approved_operations`, `approval_ticket`, and `approved_at_unix_ms` are contract data, not
  inferred CI/session state.
- The first upload adapter remains provider-neutral: it validates the manifest plus rollout
  ownership contracts and materializes an `artifact_upload_request.json` artifact instead of
  binding core or adapter code directly to a storage SDK.
- Remote model save/load should follow the same rule: use a provider-neutral
  locator contract in adapters rather than binding core artifact logic directly
  to one storage SDK or remote path shape.
- Provider-specific publish adapters should also stay thin on top of the
  provider-neutral upload request contract rather than inventing new core-facing
  approval or artifact identity surfaces.
- Promotion and rollback orchestration are currently local justfile contracts over the explicit
  eval, upload, deploy-check, and current-pointer surfaces; they are not a separate long-running
  deployment subsystem.
- Deployment-oriented CLI entrypoints are grouped under one `deploy` command family so stack checks,
  upload planning, verification bundles, and load profiles share one operator-facing namespace.
- Operator-facing CLI surfaces should stay intent-first and minimal. Low-level artifact mutation
  commands such as pointer writers, reconciliation builders, and history appenders belong behind
  repo-managed workflows or `afterburner debug ...` unless operators have a demonstrated need to
  invoke them directly.
- The target operator grammar is a small set of top-level intents such as `deploy`, `verify`,
  `rollback`, `inspect`, and `debug`, with resource/action subcommands underneath rather than
  artifact-taxonomy command names.

### Runtime, scheduler, and platform

- Distributed runtime, scheduler, and platform concerns remain distinct layers.
  - The runtime owns in-job execution semantics and checkpoint/state handling.
  - The scheduler owns queueing, placement, leases, and lifecycle policy.
  - The platform layer owns launch substrate and host/cluster provisioning only.
  - The only required scheduler/runtime coupling is the explicit lifecycle boundary
    (`START`/`STOP`/`KILL`, `READY`/`CHECKPOINTED`/`FAILED`/`HEARTBEAT`).
  - The Kubernetes path should remain `kube-rs`-compatible and reuse the same
    node inventory, topology-aware placement, lease ownership, and job lifecycle reconciliation
    model as the single-node scheduler.
  - `lws` is not part of the current scheduler contract; reconsider it only as a
    later workload-management reference or thin adapter target over the same
    kube-rs-compatible cluster path.
  - Scheduler/control-plane work is also a workload-driven capability subset,
    not a Slurm parity target.
  - Scheduler preemption is cooperative checkpoint/resume only; transparent GPU
    task suspension is not a supported assumption.
  - The deterministic scheduler/runtime simulation harness is a control-logic simulation harness only:
    lifecycle sequencing, lease transitions, layout
    feasibility, checkpoint/restart behavior, and artifact emission. It is not
    a GPU, NCCL, kernel, or timing simulator.
  - GPU colocation/sharing is a separate scheduling mode, not a synonym for
    preemption.
  - GPU colocation is only a fit for explicitly approved low-saturation
    workloads with measured interference limits; it is not the default
    scheduler policy.
  - For MacBook-local multi-process orchestration, `process-compose-flake` is a
    valid platform candidate, but only as a local substrate and never as a
    replacement for `systemd`, Kubernetes, or scheduler policy.
- Distributed-training runtime work is capability-subset work, not framework-parity work.
  - Prioritize the smallest workload-driven subset in dependency order.
  - Treat DeepSpeed-class systems as sources of patterns, not parity targets.
  - Treat external references such as Megatron-LM and DeepSpeed as pattern
    sources, not as architecture templates for this repo.
  - Combined implementations in other codebases do not change the layer
    boundary here: scheduler/control-plane concerns stay separate from in-job
    runtime semantics.
  - Within the runtime layer, topology, layout feasibility, and
    execution/state semantics should stay explicit contract surfaces rather
    than collapsing into one framework-shaped abstraction.
  - Do not promise broad generic support across all DP/TP/PP/SP-CP/EP/ZeRO combinations
    unless a concrete workload and contract require it.

### Adapter and observability constraints

- Standards integration (e.g. OpenTelemetry, OpenAPI) occurs in adapters only;
  core remains framework- and SDK-independent.
- Distributed tracing correlation also stays adapter-local and lightweight.
  - Use explicit `traceparent` propagation plus derived `trace_id` fields in adapter events
    and deployment/load-test artifacts when cross-command or cross-node correlation is needed.
  - `AFTERBURNER_TRACEPARENT` is the adapter-side rollout/load-test propagation hook.
  - Do not introduce a Tokio runtime or heavy OpenTelemetry SDK solely to add correlation fields.
- Adapter monitoring events may keep local operation names (`infer_done`, `eval_done`), but shared
  infer/eval monitoring fields should align on explicit units and stable identity keys
  (`duration_ms`, `batch_size`, `artifact_version`) so adapter-local logs can be mapped
  deliberately onto OpenTelemetry conventions later without claiming full semconv naming or
  pulling an SDK into core.

### Reference-owned stance catalog

- `docs/reference.md` owns the capability and fit catalog that grows faster than the core
  architecture boundaries. This file should stay focused on system shape, stable boundaries,
  architectural invariants, and the ADR index.
- Keep only short architecture anchors here. Detailed capability matrices, fit catalogs,
  workload envelopes, and contract-family inventories belong in `docs/reference.md` and the
  corresponding ADRs.
- Profiling and observability fits:
  - `OpenTelemetry` remains parked, profiling stays adapter-only, and the current profiling
    anchors remain `profiling_hotspot_summary.schema.json` and `profiling_environment_snapshot.json`.
  - The profiling environment provenance contract remains explicit and anchored to
    `profiling_hotspot_summary.schema.json`.
  - The profiling provenance receipt contract remains explicit and anchored to
    `profiling_environment_snapshot.json`.
  - The hotspot taxonomy stays explicit: `execution`, `framework`, `compiler`, and `incidental`.
  - See `docs/reference.md` plus ADR-016, ADR-020, ADR-025, ADR-030, and ADR-037 for the full fit
    and contract detail.
- Backend and optimization fits:
  - `Parquet`, `DataFusion`, and `Ballista` remain fit decisions, not architecture boundaries.
  - `CubeCL`, `CubeK`, `cutile-rs`, `BACKEND=metal`, Burn `0.20.1`, and the `.mpk` to `.bpk`
    cutover remain explicit fit decisions rather than core system shape.
  - Burn dependency refresh remains explicit through ADR-033 and the current Burn dependency refresh
    pin at `0.20.1`.
  - Model optimization and packaging remain a distinct post-training pipeline.
  - See `docs/reference.md` plus ADR-005, ADR-011, ADR-012, ADR-033, ADR-040, ADR-053, ADR-057,
    ADR-060, and ADR-063.
- Distributed runtime and lineage fits:
  - The multibillion target envelope stays anchored to `distributed_shard_metadata`.
  - The current distributed runtime capability surface remains `single-device execution only`,
    `First candidate expansion: DP`, while `ZeRO-1/2/3` and `TP, PP, SP/CP, EP` remain
    unsupported.
  - `distributed runtime profile trials`, the feasibility artifact, the `checkpoint index`,
    `checkpoint_group`, the distributed shard lineage receipt, and distributed shard lineage
    evidence provenance remain explicit contract anchors.
  - Distributed optimizer-state recovery remains an explicit in-job contract anchored to
    `checkpoint_group`; see ADR-061.
  - See `docs/reference.md` plus ADR-015, ADR-018, ADR-023, ADR-028, ADR-032, ADR-035, ADR-038,
    ADR-039, ADR-041, ADR-043, ADR-054, and ADR-058.
- Data and artifact lifecycle fits:
  - The artifact retention envelope remains explicit, and `artifacts/profiling/` stays a
    prune-candidate domain.
  - Deployment verification receipt remains anchored to `artifact_version`, and deployment
    verification evidence provenance remains anchored to the deployment verification receipt.
  - The deployment verification evidence provenance contract remains explicit; see ADR-027.
  - Pretraining source registry, source approval receipt, and source provenance receipt remain
    explicit, with `approval_status` and `upstream_locator` still visible contract anchors.
  - The local-first text path remains explicit: `decoder-only language model`,
    `fineweb-edu/slice`, `single-node, single-device execution`, `50M` to `300M` parameters,
    `1024` token context length, `50M` to `200M` token budgets, `AdamW`, tokenizer and packing
    surface, text inference profile sidecar, and `text_pretraining_eval_summary.json`.
  - RL remains artifact-first through `rl_rollout_metadata.schema.json` and the vectorized
    environment stance.
  - See `docs/reference.md` plus ADR-013, ADR-019, ADR-021, ADR-022, ADR-024, ADR-026, ADR-027,
    ADR-029, ADR-031, ADR-047, ADR-049, ADR-050, ADR-052, and ADR-059.

### Assumptions

The same artifact contract applies to non-image domains (e.g. sequence or graph tensors), where input semantics must be explicit and validated.

## Decisions

- [ADR-001: Training vs Inference Separation](./docs/adr/001-training-vs-inference.md)
- [ADR-002: Artifact Contract](./docs/adr/002-artifact-contract.md)
- [ADR-003: Artifact Manifest](./docs/adr/003-artifact-manifest.md)
- [ADR-004: Versioned Inference Artifacts](./docs/adr/004-artifact-versioning.md)
- [ADR-005: Custom Kernel Adoption Threshold](./docs/adr/005-custom-kernel-adoption-threshold.md)
- [ADR-006: Quantized Inference Artifact Contract](./docs/adr/006-quantized-inference-artifact-contract.md)
- [ADR-007: Async Runtime Decision](./docs/adr/007-async-runtime-decision.md)
- [ADR-008: Deployment Target Profile](./docs/adr/008-deployment-target-profile.md)
- [ADR-009: Artifact Rollout Ownership](./docs/adr/009-artifact-rollout-ownership.md)
- [ADR-010: Deploy-rs Baseline](./docs/adr/010-deploy-rs-baseline.md)
- [ADR-011: Arrow/DataFusion/Ballista/Parquet Fit](./docs/adr/011-data-infra-fit.md)
- [ADR-012: CubeCL/CubeK Fit](./docs/adr/012-cubecl-fit.md)
- [ADR-013: RL Environment Fit](./docs/adr/013-rl-environment-fit.md)
- [ADR-014: Artifact Upload Adapter Contract](./docs/adr/014-artifact-upload-adapter.md)
- [ADR-015: Multibillion-Scale Target Envelope](./docs/adr/015-multibillion-target-envelope.md)
- [ADR-016: OpenTelemetry Profiling Fit](./docs/adr/016-otel-profiling-fit.md)
- [ADR-017: Promotion And Rollback Orchestration](./docs/adr/017-promotion-orchestration.md)
- [ADR-018: Distributed Checkpoint Index](./docs/adr/018-distributed-checkpoint-index.md)
- [ADR-019: Artifact Retention Envelope](./docs/adr/019-artifact-retention-envelope.md)
- [ADR-020: Profiling Hotspot Taxonomy](./docs/adr/020-profiling-hotspot-taxonomy.md)
- [ADR-021: Pretraining Source Registry](./docs/adr/021-pretraining-source-registry.md)
- [ADR-022: Deployment Verification Receipt](./docs/adr/022-deployment-verification-receipt.md)
- [ADR-023: Distributed Shard Lineage](./docs/adr/023-distributed-shard-lineage.md)
- [ADR-024: Artifact Cleanup Dry-Run Receipt](./docs/adr/024-artifact-cleanup-dry-run-receipt.md)
- [ADR-025: Profiling Environment Provenance](./docs/adr/025-profiling-environment-provenance.md)
- [ADR-026: Pretraining Source Approval Receipt](./docs/adr/026-pretraining-source-approval-receipt.md)
- [ADR-027: Deployment Verification Evidence Provenance](./docs/adr/027-deployment-verification-evidence-provenance.md)
- [ADR-028: Distributed Shard Lineage Receipt](./docs/adr/028-distributed-shard-lineage-receipt.md)
- [ADR-029: Artifact Cleanup Execution Receipt](./docs/adr/029-artifact-cleanup-execution-receipt.md)
- [ADR-030: Profiling Provenance Receipt](./docs/adr/030-profiling-provenance-receipt.md)
- [ADR-031: Pretraining Source Provenance Receipt](./docs/adr/031-pretraining-source-provenance-receipt.md)
- [ADR-032: Distributed Shard Lineage Evidence Provenance](./docs/adr/032-distributed-shard-lineage-evidence-provenance.md)
- [ADR-033: Burn Dependency Refresh](./docs/adr/033-burn-dependency-refresh.md)
- [ADR-034: Distributed Runtime, Scheduler, And Platform Separation](./docs/adr/034-distributed-runtime-scheduler-platform-separation.md)
- [ADR-035: Distributed Training Capability Subset Scope](./docs/adr/035-distributed-training-capability-subset-scope.md)
- [ADR-036: Deployment Stack Contract](./docs/adr/036-deployment-stack-contract.md)
- [ADR-037: Distributed Tracing Correlation Contract](./docs/adr/037-distributed-tracing-correlation-contract.md)
- [ADR-038: Burn Distributed Learning Strategy Fit](./docs/adr/038-burn-distributed-learning-strategy-fit.md)
- [ADR-039: Distributed World And Rank Topology](./docs/adr/039-distributed-world-rank-topology.md)
- [ADR-040: Native Metal Backend Fit](./docs/adr/040-native-metal-backend-fit.md)
- [ADR-041: Distributed Runtime Capability Surface](./docs/adr/041-distributed-runtime-capability-surface.md)
- [ADR-043: Distributed Runtime Profile Schema](./docs/adr/043-distributed-runtime-profile-schema.md)
- [ADR-042: Scheduler Preemption And Colocation](./docs/adr/042-scheduler-preemption-and-colocation.md)
- [ADR-045: GPU Scheduler Boundary And Lifecycle](./docs/adr/045-gpu-scheduler-boundary-and-lifecycle.md)
- [ADR-046: Single-Node GPU Lease And Systemd Adapter](./docs/adr/046-single-node-gpu-lease-systemd-adapter.md)
- [ADR-047: MacBook Text Pretraining Fit](./docs/adr/047-macbook-text-pretraining-fit.md)
- [ADR-049: FineWeb-Edu Source Adoption](./docs/adr/049-fineweb-edu-source-adoption.md)
- [ADR-050: Text Tokenizer And Packing Contract](./docs/adr/050-text-tokenizer-and-packing-contract.md)
- [ADR-052: Text Model Artifact And Inference Contract](./docs/adr/052-text-model-artifact-and-inference-contract.md)
- [ADR-059: MacBook Text Pretraining Adapter](./docs/adr/059-macbook-text-pretraining-adapter.md)
- [ADR-060: JSON Over RON Fit](./docs/adr/060-json-over-ron-fit.md)
- [ADR-063: Capnproto-Rust Wire-Format Fit](./docs/adr/063-capnproto-wire-format-fit.md)
- [ADR-061: Distributed Optimizer And Checkpoint State](./docs/adr/061-distributed-optimizer-and-checkpoint-state.md)
- [ADR-062: kube-rs GPU Scheduler Placement Fit](./docs/adr/062-kube-rs-gpu-scheduler-placement-fit.md)
- [ADR-064: LWS Cluster Workload Fit](./docs/adr/064-lws-cluster-workload-fit.md)
- [ADR-065: Gateway API Inference Extension Fit](./docs/adr/065-gateway-api-inference-extension-fit.md)
- [ADR-051: Remote Model Save/Load Fit](./docs/adr/051-remote-model-save-load-fit.md)
- [ADR-056: GPU Colocation Fit](./docs/adr/056-gpu-colocation-fit.md)
- [ADR-055: Hugging Face Publish Adapter](./docs/adr/055-hugging-face-publish-adapter.md)
- [ADR-054: Distributed Runtime Layout Feasibility](./docs/adr/054-distributed-runtime-layout-feasibility.md)
- [ADR-058: Distributed Runtime Benchmark Harness](./docs/adr/058-distributed-runtime-benchmark-harness.md)
- [ADR-053: Model Optimization And Packaging Fit](./docs/adr/053-model-optimization-and-packaging-fit.md)
- [ADR-057: `cutile-rs` Backend Extension Fit](./docs/adr/057-cutile-backend-extension-fit.md)
- [ADR-048: process-compose-flake Local Substrate Fit](./docs/adr/048-process-compose-local-substrate-fit.md)
- [ADR-044: Deploy CLI Grouping](./docs/adr/044-deploy-cli-grouping.md)
