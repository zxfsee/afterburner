# Roadmap

This document outlines **intentional extensions** to the current design.
They are ordered to preserve the existing training ↔ inference boundary.
These are deliberately excluded from the current implementation to keep the core system small and inspectable.

## Artifact contract hardening
- Generate `artifacts/inference/manifest.toml` during training.
- Include:
  - input shape, dtype, normalization constants
  - output semantics (logits vs probabilities)
  - model architecture identifier and version
- Validate the manifest at inference startup to fail fast on contract drift.

## Preprocessing parity
- Extract MNIST normalization into a shared preprocessing module.
- Reuse the same logic for:
  - training data loading
  - inference input preparation
- Eliminate implicit train/serve assumptions.

## Model versioning
- Add explicit semantic versions to inference artifacts.
- Allow multiple inference artifacts to coexist under `artifacts/inference/`.
- Enable safe rollout and rollback without retraining.

## Observability hooks
- Emit structured logs for:
  - artifact loading
  - backend selection (CPU vs GPU)
  - inference latency
- Prepare for OpenTelemetry integration without backend coupling.
- Persist training metrics and logs as structured artifacts under `artifacts/train/`.

## Deployment shape
- Wrap the inference binary in a minimal HTTP or gRPC service.
- Keep the CLI as the canonical execution path.
- Treat the service layer as a thin transport adapter.

## Security & compliance
- Enforce artifact immutability at runtime.
- Add checksum verification for inference artifacts.
- Prepare for artifact signing in regulated environments.

## Scale considerations
- Explicit batch inference support with limits.
- Memory and execution safeguards for GPU inference.
- Deterministic startup behavior under load.
