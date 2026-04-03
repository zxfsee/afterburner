# Afterburner

Rust-first ML systems repo for training a Burn model, exporting a deployable
inference artifact, and exercising artifact-first deployment and observability
workflows behind explicit contracts.

## Quick start

```sh
# enter reproducible dev environment
nix develop

# train a model and export inference artifact
just train

# run inference using the exported artifact
just infer

# run deterministic eval guard and emit JSON summary
just eval-gate
```

`just infer` and `just eval-gate` resolve the artifact through `artifacts/inference/current` by default.
Override with `--artifact <path>`, `--artifact=<path>`, or positional `[artifact_path]` for compatibility.
The canonical workflow surface is `just`; use `just --list` or `just workflows`
to discover repo-managed entrypoints. The `afterburner <...>` CLI remains the
underlying contract surface behind those recipes. Operator-facing CLI commands
should stay terse and intent-first; low-level artifact assembly stays behind
`just` recipes or `afterburner debug ...`.

## At a glance

- Train a Burn model, export a deployable inference artifact, run deterministic inference/eval, and exercise artifact-first deployment and verification workflows from one Rust repo.
- Current scope is local-first runtime, deployment, and observability contract work; it is not a benchmark repo or a full serving platform.
- Uses Burn for model/runtime work, Nix for reproducible environments, and `just` as the workflow surface.

## System shape

- Training/runtime: model execution, artifact export, checkpointing, and training-side observability.
- Control plane: rollout, upload, cleanup, and deployment-side verification artifacts.
- Platform: local MacBook workflows first, then NixOS nodes and later cluster adapters.

## Read next

- [Architecture](./ARCHITECTURE.md): boundaries, invariants, and the current system shape
- [Workflow reference](./docs/workflows.md): operational recipes and artifact/receipt paths
- [Reference index](./docs/reference.md): contract and capability pointers that do not belong on the frontpage
- [ADRs](./docs/adr/): concrete architectural and contract decisions
- [Changelog](./CHANGELOG.md): active TODO queue and recent landed work

## Current scope

Afterburner separates training/runtime, control plane, and platform substrate.
The runtime decides how one job uses GPUs; the scheduler decides who gets GPUs
and when; the platform provides the execution substrate. The later cluster path
stays `kube-rs-compatible` and reuses the same lease and lifecycle model as the
single-node scheduler instead of inventing a second scheduler vocabulary.
`lws` remains a later optional adapter reference, not a current scheduler
boundary. The Gateway API inference extension also remains a later optional
traffic-management reference, not a current cluster-serving boundary.

Training runs through `afterburner train`. The repo currently supports the
MNIST path plus a bounded text-pretraining path via `--task text`, and writes
versioned artifacts under `artifacts/train/` and `artifacts/inference/`.
Inference runs through `afterburner infer`, consumes only the inference
artifact, and returns deterministic JSON on success.
Distributed optimizer-state recovery remains an in-job runtime concern and
stays anchored to an explicit `checkpoint_group` instead of scheduler-local
placement state.

## HTTP wrapper (optional)

`afterburner-http` is a thin transport adapter over the same inference path as
the CLI. It exposes `GET /healthz` and `POST /infer`, with deterministic error
payloads, bounded request limits, and graceful shutdown semantics. The CLI
remains the canonical interface. The adapter remains synchronous `tiny_http`
based today; adapter-side tracing stays on explicit
`AFTERBURNER_TRACEPARENT` propagation, and Tokio remains deferred until measured
concurrency or deployment pressure justifies it.

Rationale is documented in [ADR-001: Training vs Inference Separation](./docs/adr/001-training-vs-inference.md).

## Artifact contract

Training exports versioned inference artifacts under
`artifacts/inference/<version>/`. The active version is tracked by
`artifacts/inference/current`, and the manifest carries checksum, signature,
and precision metadata. Structured artifacts and events stay on JSON; the
inference manifest stays on TOML. `capnproto-rust` remains a later optional
wire-format reference only if measured high-volume pressure appears.

The optimized model capability surface is explicit. The only supported runtime
precision contract today is `weights_dtype = "f32"`,
`activation_dtype = "f32"`, and `quantization = "none"`. Quantization,
compression, export, and packaging remain planning-only optimization steps
until dedicated optimized-model contracts land.

The optimized model package contract is explicit as well. The
`optimized_model_package_contract.schema.json` surface records the chosen
`export_format`, the reproducible `packaging_inputs`, and the emitted
`package_layout` for an optimized artifact handoff without changing the base
inference artifact contract.

Burn refresh and artifact-format migration planning are explicit too.
`burn_bpk_migration_surface_inventory.json` records the current `.mpk`
touchpoints and cutover-sensitive surfaces so a later `.bpk` migration can land
as one tracked contract change instead of a scattered dependency refresh.

The optimized model local profile is explicit too.
`afterburner profile optimized-model-local-profile` writes
`optimized_model_local_profile.json` from an optimized package contract plus
observed local quality, latency, memory, and package-size measurements so
optimized-model decisions stay evidence-driven.

Detailed operational workflow reference lives in [docs/workflows.md](./docs/workflows.md).
Deeper contract and capability detail now lives in [docs/reference.md](./docs/reference.md).

See [ADR-002: Artifact Contract](./docs/adr/002-artifact-contract.md) for rationale.

## Runtime environment

Training and inference are backend-agnostic.

Backend selection is explicit (`BACKEND=cpu|wgpu|metal`). On Apple hardware,
`BACKEND=metal` uses Burn's Metal-backed GPU path, and if `BACKEND` is unset
the runtime attempts `wgpu` first and falls back to `cpu`.

The stack is intentionally simple:
- deterministic builds
- clear system boundaries
- infrastructure that can be reasoned about end-to-end

License: MIT
