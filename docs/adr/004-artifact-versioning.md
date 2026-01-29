# ADR-004: Versioned Inference Artifacts

## Context

The inference artifact is immutable and is the only input to runtime binaries.
Without explicit versioning, multiple artifacts cannot safely coexist and there
is no reliable, low-risk rollback path.

## Decision

Inference artifacts are versioned using semantic versions and stored under:

`artifacts/inference/<version>/`

Training writes a `current` pointer file at `artifacts/inference/current` to
identify the active version. The manifest records the artifact version and
inference validates the manifest against the active artifact.

## Consequences

- Multiple artifacts can coexist without collisions.
- Rollouts and rollbacks are performed by updating `current`.
- Inference stays contract-driven without adding a serving layer.
