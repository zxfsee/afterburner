# ADR-027: Deployment Verification Evidence Provenance

## Context

`deployment_verification_receipt.json` already defines an `evidence` field, but
that alone still leaves the provenance model implicit. Without one explicit
evidence-reference vocabulary, future verification receipts could point at logs,
artifacts, or events inconsistently and make review harder.

## Decision

Keep `deployment_verification_receipt.json` as the anchor artifact, and require
verification receipts to make evidence provenance explicit through an
`evidence_sources` layer that can at least carry:

- `artifact_path`
- `event_name`
- `observed_at_unix_ms`

Current stance: the repo now materializes this provenance layer through
`afterburner deploy verification-receipt`.

## Consequences

- Deployment verification receipts can point back to concrete observed evidence
  without embedding raw logs directly.
- Later rollout automation can reuse one evidence-reference vocabulary instead
  of inventing per-check provenance fields.
- The repo records the provenance boundary now while keeping the current receipt
  contract narrow.
