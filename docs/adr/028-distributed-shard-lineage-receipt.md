# ADR-028: Distributed Shard Lineage Receipt

## Context

The repo now defines the minimum lineage fields that distributed shard metadata
will need over time, but it still has no explicit receipt for when that lineage
state was checked or refreshed. Without a receipt, provenance reviews would have
to infer lineage validation from surrounding run notes or future metadata edits.

## Decision

Lineage updates can now write one explicit artifact:
`distributed_shard_lineage_receipt.json`

The minimum receipt contract should include:

- `shard_id`
- `source`
- `source_revision`
- `checkpoint_group`
- `checked_at_unix_ms`

Current stance: the repo now materializes this receipt through the grouped
`just distributed-shard-lineage-receipt` flow, with the low-level writer kept
behind `afterburner debug lineage receipt`.

## Consequences

- Lineage reviews get one explicit audit artifact instead of relying on implicit
  metadata changes.
- Future distributed training and recovery work can distinguish lineage payloads
  from lineage check/refresh records.
- The repo records the receipt vocabulary now while keeping the active shard
  metadata contract unchanged.
