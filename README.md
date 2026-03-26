# Afterburner

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

`just infer` and `just eval` resolve the artifact through `artifacts/inference/current` by default.
Override with `--artifact <path>`, `--artifact=<path>`, or positional `[artifact_path]` for compatibility.
The canonical workflow surface is `just`; use `just --list` or `just workflows`
to discover repo-managed entrypoints. The `afterburner <...>` CLI remains the
underlying contract surface behind those recipes.

## At a glance

- Rust-first ML systems repo focused on boundaries, contracts, and reproducible workflows.
- Uses Burn for model/runtime work, Nix for reproducible environments, and `just` as the workflow surface.
- Optimized for clarity of training, artifact, deployment, and observability contracts over model quality benchmarks.

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

## What this is

Afterburner is a Rust-first ML systems repo for explicit ML runtime,
artifact, deployment, and observability boundaries. It is not a benchmark repo
or a full serving platform.

## Why Afterburner

Most ML repos are clear about models and unclear about contracts. Afterburner
optimizes for the opposite:
- training stays mutable
- inference stays stable and deployable
- infrastructure choices stay explicit and reproducible
- evolution happens behind inspectable contract boundaries

## Design goals

- Reproducibility over convenience
- Explicit boundaries between training and inference
- Inspectable, versioned artifacts
- Minimal but intentional infrastructure
- Expand capability without rewriting architecture

## Architecture note

Afterburner separates training/runtime, control plane, and platform substrate.
The runtime decides how one job uses GPUs; the scheduler decides who gets GPUs
and when; the platform provides the execution substrate. Detailed boundaries,
lifecycle semantics, and distributed/runtime stance live in
[Architecture](./ARCHITECTURE.md) and the linked ADRs.

## Training

Training runs through `afterburner train`. The repo currently supports the
MNIST path plus a bounded text-pretraining path via `--task text`.

Training writes versioned artifacts under `artifacts/train/` and
`artifacts/inference/`, including the training scalability contract and kernel
adoption threshold contract. The text path also emits
`text_pretraining_run.json`, learner checkpoints, `text_pretraining_eval_summary.json`,
and a separate text inference sidecar.

## Inference

Inference runs through `afterburner infer` and consumes only the inference
artifact. It has no access to training internals and returns deterministic JSON
on success.

## HTTP wrapper (optional)

`afterburner-http` is a thin transport adapter over the same inference path as
the CLI. It exposes `GET /healthz` and `POST /infer`, with deterministic error
payloads, bounded request limits, and graceful shutdown semantics.

Rationale is documented in [ADR-001: Training vs Inference Separation](./docs/adr/001-training-vs-inference.md).

## Artifact contract

Training exports versioned inference artifacts under `artifacts/inference/<version>/`, consisting of:
- serialized model weights (Burn `CompactRecorder`)
- a lightweight `manifest.toml` describing input shape/dtype, precision metadata, normalization, model architecture identity, artifact version, checksum, signature placeholders, and canonicalization metadata

The active version is tracked by `artifacts/inference/current`, enabling
safe rollout/rollback without retraining. The manifest carries checksum,
signature, and precision metadata, and unsupported reduced-precision artifacts
fail fast at startup.

Detailed operational workflow reference lives in [docs/workflows.md](./docs/workflows.md).
Deeper contract and capability detail now lives in [docs/reference.md](./docs/reference.md).

See [ADR-002: Artifact Contract](./docs/adr/002-artifact-contract.md) for rationale.

## CPU vs GPU execution

Training and inference are backend-agnostic.

The same model artifact can run on CPU or GPU without retraining:
- GPU is intended for training and throughput-oriented inference
- CPU is supported for development, debugging, and environments without accelerators

Backend selection is explicit (`BACKEND=cpu|wgpu|metal`), reflecting production systems where execution targets vary by cost, latency, and scale.
On Apple hardware, `BACKEND=metal` uses Burn's Metal-backed GPU path under the existing Burn GPU stack.
If `BACKEND` is unset, training attempts `wgpu` first and falls back to `cpu` automatically when no compatible GPU adapter is available.

## Infrastructure choices

This project uses **Rust, Burn, and Nix** to favor explicitness, reproducibility, and inspectability over rapid iteration.

The trade-off is slower experimentation in exchange for:
- deterministic builds
- clear system boundaries
- infrastructure that can be reasoned about end-to-end

This mirrors environments where deployment safety and auditability outweigh iteration speed.

## Documentation

- [Architecture overview](./ARCHITECTURE.md)
- [Design decisions (ADRs)](./docs/adr/)
- [Changelog](./CHANGELOG.md)

---

License: MIT
