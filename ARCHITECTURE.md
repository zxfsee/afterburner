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

## Distributed Training Layers

The distributed training stack is split into three layers with explicit
responsibility boundaries:

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

### Trade-offs

This design prioritizes explicit boundaries and reproducibility over rapid iteration.
As a result, some conveniences common in ML prototypes (implicit preprocessing,
auto-versioning, embedded serving) are intentionally absent.

## Invariants

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
- Inference artifact precision is explicit in the manifest. The current runtime accepts only
  `f32` weights/activations with `quantization = none`; any reduced-precision artifact must
  declare itself explicitly and remain rejected until a dedicated runtime path is introduced.
- No async runtime is adopted. The current HTTP adapter remains synchronous and `tiny_http`-based
  until profiling or deployment requirements show that a Tokio-class runtime is necessary.
- Deployment adapters must resolve through an explicit deployment target profile contract before
  introducing deploy tooling. Target host, user, system, artifact root, and activation strategy
  are contract data, not ad-hoc shell configuration.
- The baseline deploy-rs adapter consumes the deployment target profile contract directly from the
  checked fixture/example and uses `activate.custom` against the packaged `afterburner` binary
  rather than inventing a NixOS system deployment surface.
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
- Promotion and rollback orchestration are currently local justfile contracts over the explicit
  eval, upload, deploy-check, and current-pointer surfaces; they are not a separate long-running
  deployment subsystem.
- Distributed runtime, scheduler, and platform concerns remain distinct layers.
  - The runtime owns in-job execution semantics and checkpoint/state handling.
  - The scheduler owns queueing, placement, leases, and lifecycle policy.
  - The platform layer owns launch substrate and host/cluster provisioning only.
  - The only required scheduler/runtime coupling is the explicit lifecycle boundary
    (`START`/`STOP`/`KILL`, `READY`/`CHECKPOINTED`/`FAILED`/`HEARTBEAT`).
- Standards integration (e.g. OpenTelemetry, OpenAPI) occurs in adapters only;
  core remains framework- and SDK-independent.
- Adapter monitoring events may keep local operation names (`infer_done`, `eval_done`), but shared
  infer/eval monitoring fields should align on explicit units and stable identity keys
  (`duration_ms`, `batch_size`, `artifact_version`) so adapter-local logs can be mapped
  deliberately onto OpenTelemetry conventions later without claiming full semconv naming or
  pulling an SDK into core.
- Profiling artifacts follow the same rule: the current hotspot summary keeps local identity fields
  (`artifact_version`, `backend`, `weights_artifact`, `profile_command`) and does not add
  OpenTelemetry distributed tracing/resource fields or SDK/runtime dependencies yet.
- Profiling artifacts also need a small profiling environment provenance layer over time:
  `profiling_hotspot_summary.schema.json` should eventually be accompanied by at least `host_os`,
  `host_arch`, `profiler_version`, and `profiler_path` so hotspot comparisons do not depend on
  unstated host assumptions.
- Profiling provenance should also gain an explicit profiling provenance receipt layer over time,
  so later profiling reviews can audit when `profiling_environment_snapshot.json` was captured or
  refreshed instead of inferring timing from surrounding workflow logs.
- Profiling summaries also use a lightweight hotspot taxonomy for interpretation:
  execution (`afterburner::...`), framework (`burn_tensor::...`), compiler/runtime (`cubecl...`),
  and incidental support work. The raw symbol remains the contract; the taxonomy is for triage.
- Optimization policy changes must be expressed as explicit artifacts or gates before
  introducing new execution paths. Custom kernels are justified only when the checked
  kernel-adoption contract is refreshed with current model coverage and supporting
  backend profile evidence.
- CubeCL/CubeK are the preferred first candidate if custom-kernel work is ever justified,
  because the current Burn GPU stack already carries them transitively. They remain out of
  the repo's direct dependencies and public contracts until the checked threshold is crossed.
- Burn dependency refreshes should stay on the latest stable line, not pre-release churn. The
  current explicit stance is Burn `0.20.1`; a later refresh should wait for a newer stable release
  and keep `.mpk` to `.bpk` migration as a separate contract decision.
- RL work remains artifact-first for now: `rl_rollout_metadata.schema.json` is the current
  contract surface, with a single-environment identity string and no in-repo simulator
  bindings or vectorized environment runtime until measured need justifies them.
- The multibillion-scale target envelope is also contract-first: larger-scale claims must stay
  grounded in explicit training scalability, distributed_shard_metadata, manifest precision,
  rollout ownership, upload request, and deployment target profile contracts before runtime
  orchestration expands.
- Distributed checkpoint recovery should also stay contract-first: a future checkpoint index must
  declare `artifact_version`, `checkpoint_root`, `shard_count`, and `shard_metadata_path` instead
  of inferring shard membership from directory layout alone.
- Distributed shard metadata also needs an explicit distributed shard lineage layer over time:
  shard ownership alone is not enough, and future lineage-aware metadata should at least tie each
  `shard_id` back to `source`, `source_revision`, and `checkpoint_group` instead of host-local
  naming.
- Distributed shard lineage should also gain an explicit distributed shard lineage receipt layer so
  later provenance checks can record when `shard_id`, `source`, `source_revision`, and
  `checkpoint_group` were reviewed without mutating the lineage payload itself.
- Distributed shard lineage evidence provenance should also be explicit: future distributed shard
  lineage receipts should carry distributed shard lineage evidence provenance through structured
  `evidence_sources` references instead of treating reviewed shard metadata and checkpoint roots as
  unstated context.
- Artifact cleanup should preserve a minimum retention envelope: keep
  `artifacts/inference/current`, the referenced `artifacts/inference/<version>/`,
  active rollout evidence under `artifacts/deploy/`, required decision artifacts under
  `artifacts/train/`, and the current approved drift baseline state; `artifacts/profiling/`
  and superseded eval artifacts are prune candidates once nothing active references them.
- Cleanup review should also converge on one explicit cleanup dry-run receipt layered on
  `artifact_cleanup_inventory.json`, so planned removals can be audited without mixing preview
  decisions into the inventory classification artifact itself.
- Cleanup execution should also converge on one explicit cleanup execution receipt layered on
  `artifact_cleanup_dry_run_receipt.json`, so future destructive runs can audit actual removals
  and skips separately from preview planning.
- Deployment verification should also converge on one explicit deployment verification receipt
  per promoted `artifact_version`, so post-deploy checks can be audited from a receipt artifact
  and matching event instead of raw logs alone.
- Deployment verification evidence provenance should also be explicit: future deployment
  verification receipts should carry deployment verification evidence provenance through
  structured `evidence_sources` references instead of treating the `evidence` field as an
  untyped log dump.
- Pretraining dataset manifests should treat `source` as a source registry key
  and `source_revision` as the approved snapshot selector within that source,
  so corpus identity does not degrade into free-form manifest labels as
  pretraining data expands.
- Pretraining source registry state should also have an explicit source approval receipt layer over
  time, so `approval_status` changes can be audited with `approved_by`, `approval_ticket`, and
  `approved_at_unix_ms` instead of being inferred from registry edits alone.
- Pretraining source approvals should also carry a source provenance receipt layer over time, so
  approval receipts can point back to `upstream_locator` and the reviewed registry entry instead of
  relying on surrounding process logs for review context.

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
