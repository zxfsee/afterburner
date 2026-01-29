# Afterburner

## Quick start

```sh
# enter reproducible dev environment
nix develop

# train a model and export inference artifact
just train

# run inference using the exported artifact
just infer
```

## What this is
Afterburner is a minimal Rust-based ML system demonstrating **training, inference, and reproducible infrastructure** using Burn and Nix.
It is intentionally small, but structured to reflect how production ML systems separate concerns between training, artifacts, and runtime.
While the example model uses MNIST, the system boundaries are designed for scientific ML workloads (e.g. molecular, protein, or graph-based models) where inference contracts, auditability, and deployment safety matter more than model accuracy.

## What this is not
This is not a production model, training pipeline, or serving system.
It is a boundary-focused reference implementation intended to demonstrate
system design judgment, not model performance.
It intentionally omits serving layers, rollout tooling, and multi-node scheduling concerns.

## Why this exists
This repository focuses on **system boundaries**, not model quality.

It demonstrates how to design ML systems where:
- training is experimental and mutable
- inference is stable, auditable, and deployable
- infrastructure choices are explicit and reproducible

The goal is to make training, artifacts, and runtime behavior easy to reason about and hard to misuse.

Future extensions are listed in [Roadmap](./ROADMAP.md).

## Design goals
- Reproducibility over convenience
- Explicit boundaries between training and inference
- Inspectable, versioned artifacts
- Minimal but intentional infrastructure

## Training
Training is executed as a standalone binary.

It produces artifacts under `artifacts/`, including:
- a trained model output under `artifacts/train/`
- a stable, inference-ready model artifact under `artifacts/inference/`

Training code owns experimentation and optimization, but does **not** define the inference contract or runtime behavior.

## Inference
Inference is executed via a separate CLI binary.

It consumes **only** the inference artifact and has no access to training internals.  
This enforces a strict boundary between model development and runtime execution.

Training and inference are intentionally exposed as separate binaries to mirror production ML serving systems.

## HTTP wrapper (optional)
Afterburner includes a minimal HTTP wrapper binary (`afterburner-http`) that exposes:
- `GET /healthz`
- `POST /infer` (accepts raw 784 bytes or JSON with base64, returns logits)

The HTTP wrapper is intentionally thin and still uses the same inference path as the CLI.
The CLI remains the canonical interface; the HTTP binary is only a transport adapter.

Rationale is documented in [ADR-001: Training vs Inference Separation](./docs/adr/001-training-vs-inference.md).

## Artifact contract
Training exports versioned inference artifacts under `artifacts/inference/<version>/`, consisting of:
- serialized model weights (Burn `CompactRecorder`)
- a lightweight `manifest.toml` describing input shape/dtype, normalization, model architecture identity, artifact version, and checksum

The active version is tracked by `artifacts/inference/current`, enabling safe rollout/rollback without retraining.
Set `ARTIFACT_VERSION` to control the exported version.

Rollback example:
```sh
printf "0.1.0\n" > artifacts/inference/current
```

Inference binaries treat this artifact as immutable and consume it as their sole input.  
Training checkpoints, metrics, and logs are explicitly excluded from the inference contract.
Inference emits minimal structured events as JSON lines on stderr for artifact loading and backend selection,
without introducing a logging framework.

This mirrors real-world model deployment, where training pipelines and serving environments are cleanly separated.

See [ADR-002: Artifact Contract](./docs/adr/002-artifact-contract.md) for rationale.

## CPU vs GPU execution
Training and inference are backend-agnostic.

The same model artifact can run on CPU or GPU without retraining:
- GPU is intended for training and throughput-oriented inference
- CPU is supported for development, debugging, and environments without accelerators

Backend selection is explicit (`BACKEND=cpu`), reflecting production systems where execution targets vary by cost, latency, and scale.

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
- [Roadmap](./ROADMAP.md)

---

License: MIT
