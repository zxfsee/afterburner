# ADR-003: Artifact Manifest for Inference

## Context

The inference artifact is the sole input to the runtime binary. Without an
explicit contract, mismatches in model architecture or preprocessing can go
undetected and silently change inference behavior.

This ADR refines the boundary defined in ADR-002 by specifying the manifest
schema and validation requirements.

## Decision

Training must export a `manifest.toml` alongside the inference weights under
`artifacts/inference/`. The manifest includes:

- model architecture identifier and version
- expected input shape and dtype
- normalization constants and notes

Inference must load the manifest at startup and fail fast on any mismatch with
the compiled expectations.

## Consequences

- Inference becomes deterministic and contract-driven
- Mismatches are surfaced immediately with structured error output
- The artifact contract is explicit and auditable
