# ADR-001: Training vs Inference Separation

## Context

Model training and inference have different lifecycles and operational requirements.
Early prototypes often couple these paths, which makes deployment and reasoning harder.

## Decision

Training and inference stay logically separated in core design, while exposed through
a unified CLI:

- `afterburner train`
- `afterburner infer`
- `afterburner eval`

Inference consumes only a stable artifact and has no access to training internals.
An optional HTTP wrapper exists as a separate binary and uses the same inference path.

## Consequences

- Clear boundary between experimentation and runtime without requiring separate task binaries.
- Simpler command surface (`afterburner <subcommand>`) for local use and automation.
- Boundary discipline must be enforced by core APIs and contract validation.
