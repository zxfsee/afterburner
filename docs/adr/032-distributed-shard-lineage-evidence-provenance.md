# ADR-032: Distributed Shard Lineage Evidence Provenance

## Context

`distributed_shard_lineage_receipt.json` can record the lineage fields being
checked, but it still does not say how those values were sourced. Without one
explicit evidence-reference vocabulary, later provenance reviews would have to
infer which shard metadata file or checkpoint root the receipt summarized.

## Decision

Keep `distributed_shard_lineage_receipt.json` as the anchor artifact, but
require future lineage receipts to make evidence provenance explicit through an
`evidence_sources` layer that can at least carry:

- `metadata_path`
- `checkpoint_root`
- `observed_at_unix_ms`

Current stance: this decision defines the minimum evidence provenance boundary
only. It does not add a new lineage verification command or event yet.

## Consequences

- Lineage receipts can point back to concrete shard metadata and checkpoint
  evidence without embedding raw logs.
- Future distributed provenance tooling can reuse one evidence-reference
  vocabulary instead of inventing ad hoc lineage source fields.
- The repo records the provenance boundary now while keeping the current
  lineage-receipt contract narrow.
