# Roadmap

This document outlines **intentional extensions** to the current design.
They are ordered to preserve the existing training ↔ inference boundary.
These are deliberately excluded from the current implementation to keep the core system small and inspectable.

## Completed
- Preprocessing parity between training and inference.
- Minimal HTTP wrapper that keeps the CLI canonical.
- Versioned inference artifacts with a current pointer for rollbacks.

## Observability hooks
- Emit structured logs for:
  - artifact loading
  - backend selection (CPU vs GPU)
  - inference latency
- Prepare for OpenTelemetry integration without backend coupling.
- Persist training metrics and logs as structured artifacts under `artifacts/train/`.

## Security & compliance
- Prepare for artifact signing in regulated environments.

## Scale considerations
- Explicit batch inference support with limits.
- Memory and execution safeguards for GPU inference.
- Deterministic startup behavior under load.
