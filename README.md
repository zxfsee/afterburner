# Afterburner

## What this is
Afterburner is a minimal Rust-based ML system demonstrating **training, inference, and reproducible infrastructure** using Burn and Nix.

It is intentionally small, but structured to reflect how production ML systems separate concerns between training, artifacts, and runtime execution.

## Why this exists
This repository focuses on **system boundaries**, not model quality.

It demonstrates how to design ML systems where:
- training is experimental and mutable
- inference is stable, auditable, and deployable
- infrastructure choices are explicit and reproducible

The goal is to make training, artifacts, and runtime behavior easy to reason about and hard to misuse.

## Design goals
- Reproducibility over convenience
- Explicit boundaries between training and inference
- Inspectable, versioned artifacts
- Minimal but intentional infrastructure

## Training
Training is executed as a standalone binary.

It produces artifacts under `artifacts/`, including:
- training checkpoints
- metrics and logs
- a stable, inference-ready model artifact

Training code owns experimentation and optimization, but does **not** define the inference contract.

## Inference
Inference is executed via a separate CLI binary.

It consumes **only** the inference artifact and has no access to training internals.  
This enforces a strict boundary between model development and runtime execution.

Training and inference are intentionally exposed as separate binaries to mirror production ML serving systems.

Rationale is documented in [ADR-001: Training vs Inference Separation](docs/adr/001-training-vs-inference.md).

## Artifact contract
Training exports a single, versioned inference artifact under `artifacts/infer/`, consisting of:
- serialized model weights (Burn `CompactRecorder`)
- a lightweight manifest describing input and output assumptions

Inference binaries treat this artifact as immutable and consume it as their sole input.  
Training checkpoints, metrics, and logs are explicitly excluded from the inference contract.

This mirrors real-world model deployment, where training pipelines and serving environments are cleanly separated.

See [ADR-002: Artifact Contract](docs/adr/002-artifact-contract.md) for rationale.

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

## Documentation
- [Architecture overview](docs/architecture.md)
- [Design decisions (ADRs)](docs/adr/)
