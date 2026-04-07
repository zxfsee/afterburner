# ADR-032: Distributed Shard Lineage Evidence Provenance

## Context

`distributed_shard_lineage_receipt.json` can record the lineage fields being
checked, but it still does not say how those values were sourced. Without one
explicit evidence-reference vocabulary, later provenance reviews would have to
infer which shard metadata file or checkpoint root the receipt summarized.

## Decision

Keep `distributed_shard_lineage_receipt.json` as the anchor artifact, and
require lineage receipts to make evidence provenance explicit through an
`evidence_sources` layer that can at least carry:

- `metadata_path`
- `checkpoint_root`
- `observed_at_unix_ms`

Current stance: the repo now materializes this provenance layer through the
grouped `just distributed-shard-lineage-receipt` flow, with the low-level
writer kept behind `afterburner debug lineage receipt`.

## Consequences

- Lineage receipts can point back to concrete shard metadata and checkpoint
  evidence without embedding raw logs.
- Compact append-only history over lineage pointer changes can stay downstream
  and audit-focused as long as it remains derived from the explicit pointer
  artifact instead of controller or shell logs.
- Compact append-only history over lineage handoff changes can also stay
  downstream and audit-focused as long as it remains derived from the explicit
  handoff artifact instead of controller or shell logs.
- A reconciliation artifact over lineage handoff state is also acceptable as a
  downstream comparison surface as long as it remains derived from the explicit
  bundle and handoff artifacts instead of controller or shell checks.
- Compact append-only history over lineage handoff reconciliation changes can
  also stay downstream and audit-focused as long as it remains derived from the
  explicit reconciliation artifact instead of controller or shell logs.
- A reconciliation artifact over lineage transport locator state is also
  acceptable as a downstream comparison surface as long as it remains derived
  from the explicit handoff and locator artifacts instead of controller or
  shell checks.
- Compact append-only history over lineage transport locator reconciliation
  changes can also stay downstream and audit-focused as long as it remains
  derived from the explicit reconciliation artifact instead of controller or
  shell logs.
- Future distributed provenance tooling can reuse one evidence-reference
  vocabulary instead of inventing ad hoc lineage source fields.
- A reconciliation artifact over lineage evidence bundle state is also
  acceptable as a downstream comparison surface as long as it remains derived
  from the explicit receipt and bundle artifacts instead of controller or shell
  checks.
- Compact append-only history over lineage evidence bundle reconciliation
  changes can also stay downstream and audit-focused as long as it remains
  derived from the explicit reconciliation artifact instead of controller or
  shell logs.
- The repo records the provenance boundary now while keeping the current
  lineage-receipt contract narrow.
