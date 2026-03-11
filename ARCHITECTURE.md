# Architecture Overview

This system models a minimal ML production loop:

Training -> Artifact -> Inference

## High-level flow

[ `afterburner train` ]
        |
        v
[ artifacts/inference/<version>/model.mpk ]
        |
        v
[ artifacts/inference/<version>/manifest.toml ]
        ^
        |
[ artifacts/inference/current ]
        |
        v
[ `afterburner infer` ]
        |
        v
[ Optional HTTP Wrapper ]

Training produces artifacts.
Inference consumes artifacts via the CLI and the optional HTTP wrapper.
No runtime coupling exists between training and inference.
The inference manifest includes checksum validation plus signing-ready placeholders and canonicalization metadata.

Training also emits JSONL observability events under `artifacts/train/` to keep
metrics and runtime metadata inspectable without introducing a logging stack.

### Trade-offs

This design prioritizes explicit boundaries and reproducibility over rapid iteration.
As a result, some conveniences common in ML prototypes (implicit preprocessing,
auto-versioning, embedded serving) are intentionally absent.

## Invariants

- Adapters depend on core; core must not depend on adapters.
  - “Core” = library modules implementing artifact contract + preprocessing + inference logic.
  - “Adapters” = binaries/transport layers (CLI, HTTP wrapper) and any integration glue.
  - Any new adapter must call a stable core API; do not import adapter modules from core.
- Public contracts (artifact format, CLI surface, HTTP surface, event schema) must be:
  - explicit and versioned when needed,
  - validated at boundaries,
  - protected by compatibility tests (e.g. golden fixtures).
- Standards integration (e.g. OpenTelemetry, OpenAPI) occurs in adapters only;
  core remains framework- and SDK-independent.
- Adapter monitoring events may keep local operation names (`infer_done`, `eval_done`), but shared
  infer/eval monitoring fields should align on explicit units and stable identity keys
  (`duration_ms`, `batch_size`, `artifact_version`) so adapter-local logs can map cleanly onto
  OpenTelemetry-style semantics without pulling an SDK into core.

### Assumptions

The same artifact contract applies to non-image domains (e.g. sequence or graph tensors), where input semantics must be explicit and validated.

## Decisions

- [ADR-001: Training vs Inference Separation](./docs/adr/001-training-vs-inference.md)
- [ADR-002: Artifact Contract](./docs/adr/002-artifact-contract.md)
- [ADR-003: Artifact Manifest](./docs/adr/003-artifact-manifest.md)
- [ADR-004: Versioned Inference Artifacts](./docs/adr/004-artifact-versioning.md)
