# Afterburner

## What this is
Minimal Rust-based ML system demonstrating training, inference,
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

Training and inference are exposed as separate binaries to make system boundaries explicit.

## Infrastructure choices
Why Rust, Burn, Nix.
What trade-offs were made.

## Artifact Contract
Training produces a stable, inference-ready artifact consisting of:
- serialized model weights (Burn CompactRecorder)
- a lightweight manifest describing input/output assumptions

Inference binaries treat this artifact as immutable and do not depend on training internals.
This mirrors how production model-serving systems separate training pipelines from deployment and runtime.
