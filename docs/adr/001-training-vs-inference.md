# ADR-001: Separate Training and Inference Binaries

## Context
Model training and inference have different lifecycles, dependencies, and operational requirements.
In early prototypes, they are often tightly coupled, which makes deployment and reasoning harder.

## Decision
Training and inference are implemented as separate binaries.
Inference consumes only a stable artifact and has no access to training internals.

## Consequences
- Clear boundary between experimentation and runtime
- Easier to reason about deployment and serving
- Slightly more boilerplate compared to a monolithic binary design.
