# Architecture Overview

This system models a minimal ML production loop:

Training -> Artifact -> Inference

## High-level flow

[ Training Binary ]
        |
        v
[ artifacts/inference/model.mpk ]
        |
        v
[ artifacts/inference/manifest.toml ]
        |
        v
[ Inference Binary ]

Training produces artifacts.
Inference consumes artifacts.
No runtime coupling exists between training and inference.

### Trade-offs
This design prioritizes explicit boundaries and reproducibility over rapid iteration.
As a result, some conveniences common in ML prototypes (implicit preprocessing,
auto-versioning, embedded serving) are intentionally absent.

### Assumptions
The same artifact contract applies to non-image domains (e.g. sequence or graph tensors), where input semantics must be explicit and validated.

## Decisions
- [ADR-001: Training vs Inference Separation](./docs/adr/001-training-vs-inference.md)
- [ADR-002: Artifact Contract](./docs/adr/002-artifact-contract.md)
- [ADR-003: Artifact Manifest](./docs/adr/003-artifact-manifest.md)
