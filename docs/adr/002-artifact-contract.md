# ADR-002: Immutable Artifact Contract for Inference

## Context

Training produces many outputs: checkpoints, metrics, logs, and intermediate state.
Inference, however, requires a stable and minimal interface that can be deployed,
versioned, and reasoned about independently of the training pipeline.

Tightly coupling inference to training outputs increases operational complexity
and makes deployment and reproducibility harder.

## Decision

Training produces a single, inference-ready artifact set under `artifacts/inference/`
consisting of model weights and a manifest.

Inference binaries consume **only** this artifact set and have no access to training
internals, checkpoints, or logs.

The artifact is treated as immutable once produced.

## Consequences

- Clear and enforceable boundary between experimentation and runtime
- Simplifies deployment and serving workflows
- Enables reproducible inference across environments
- Requires discipline in defining and evolving the artifact contract
- Slightly more structure compared to ad-hoc checkpoint loading
