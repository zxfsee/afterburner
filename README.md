# Afterburner

## What this is
Minimal Rust-based ML system demonstrating model training, inference,
and reproducible infrastructure using Burn and Nix.

## Design goals
- Reproducibility over convenience
- Explicit boundaries between training and inference
- Inspectable artifacts
- Minimal but intentional infrastructure

## Training
How training works, where artifacts go.

## Inference
How inference is invoked and why CLI-first.

## Training
Training is executed as a standalone binary and produces versioned artifacts under `artifacts/`,
including checkpoints, metrics, and an inference-ready model artifact.

## Inference
Inference is executed via a separate CLI binary that consumes only the inference artifact,
without access to training internals.

Training and inference are exposed as separate binaries to make system boundaries explicit.

## Infrastructure choices
This project uses Rust, Burn, and Nix to favor explicitness, reproducibility, and inspectability over rapid iteration or convenience.
Trade-offs include slower experimentation in exchange for clearer system boundaries.

## Artifact Contract

Training produces a stable, inference-ready artifact under `artifacts/inference/`, consisting of:
- serialized model weights (Burn `CompactRecorder`)
- a lightweight manifest describing input and output assumptions

Inference binaries treat this artifact as immutable and consume it as their sole input.
Training checkpoints, metrics, and logs are intentionally excluded from the inference contract.

This mirrors production model-serving systems, where training pipelines and runtime
environments are cleanly separated.
