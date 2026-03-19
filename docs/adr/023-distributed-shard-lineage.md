# ADR-023: Distributed Shard Lineage

## Context

`distributed_shard_metadata.schema.json` already identifies shard ownership,
index, and checksum, but it still does not explain where a shard came from in
dataset or checkpoint terms. Without explicit lineage, operators would have to
recover provenance from filenames, local directories, or external run notes.

## Decision

The distributed shard lineage contract should stay layered on
`distributed_shard_metadata.schema.json`, not replace it. A future lineage-aware
shard metadata surface must at least make these fields explicit:

- `shard_id`
- `source`
- `source_revision`
- `checkpoint_group`

Current stance: this decision defines the minimum lineage boundary only. It does
not add new runtime sharding logic or extend the current parser yet.

## Consequences

- Future distributed training and recovery work gets one explicit provenance
  vocabulary instead of inferring lineage from local naming.
- Shard metadata can remain the anchor contract while gaining the minimum fields
  needed to tie each shard back to dataset and checkpoint history.
- The repo records the lineage requirement now without prematurely changing the
  active shard metadata schema or loader.
