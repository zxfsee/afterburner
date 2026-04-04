# ADR-026: Pretraining Source Approval Receipt

## Context

The source registry decision now says each source entry should carry an
`approval_status`, but that still leaves approval changes implicit. Without one
explicit receipt, a source could move from review to approval state without a
durable audit artifact explaining who approved it and when.

## Decision

Pretraining source approval updates can now write one explicit artifact:
`pretraining_source_approval_receipt.json`

The minimum receipt contract should include:

- `source`
- `source_revision`
- `approval_status`
- `approved_by`
- `approval_ticket`
- `approved_at_unix_ms`

Current stance: the repo now materializes this receipt through
`afterburner source approval receipt`.

## Consequences

- Source approval state can be audited separately from dataset manifests and the
  future source registry definition.
- Approval changes get a stable review vocabulary before any registry behavior
  is added.
- Later registry or provenance work can consume one explicit approval receipt
  instead of inferring status changes from edited source entries alone.
