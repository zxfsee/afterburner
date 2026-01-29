# Roadmap

This document outlines **intentional extensions** to the current design.
They are ordered to preserve the existing training ↔ inference boundary.
These are deliberately excluded from the current implementation to keep the core system small and inspectable.

## Completed
- Preprocessing parity between training and inference.
- Minimal HTTP wrapper that keeps the CLI canonical.
- Versioned inference artifacts with a current pointer for rollbacks.
- Structured observability events for training and inference.

## Security & compliance
- Prepare for artifact signing in regulated environments.

## Scale considerations
- Explicit batch inference support with limits.
- Memory and execution safeguards for GPU inference.
- Deterministic startup behavior under load.
