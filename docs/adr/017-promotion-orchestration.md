# ADR-017: Promotion And Rollback Orchestration

## Context

The repo now has explicit eval, upload, rollout-ownership, deploy-target, and
artifact-versioning contracts, but there is still no stated local orchestration
flow connecting them. Without one explicit sequence, promotion and rollback
would drift into ad-hoc shell steps and lose deterministic pass/fail evidence.

## Decision

Add local orchestration recipes in `justfile`:

- `rollout-check` to gate a candidate artifact through eval, upload planning,
  and deploy definition validation,
- `rollout-promote` to save the previous `artifacts/inference/current` pointer
  and advance it to the candidate version,
- `rollout-verify` to exercise the promoted current pointer and rerun the eval
  gate against the candidate artifact,
- `rollout-rollback` to restore the saved previous current pointer.

This is intentionally a local contract orchestration layer, not a new deployment
service. It consumes the existing explicit contracts and keeps each step
inspectable and reversible.

## Consequences

- Promotion and rollback now have one documented local sequence instead of
  improvised shell state.
- Eval, upload, deploy-check, and current-pointer updates stay explicit and
  reviewable.
- Future deployment automation can reuse this sequence instead of inventing a
  second orchestration vocabulary.
