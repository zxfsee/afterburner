# ADR-018: Distributed Checkpoint Index

## Context

The repo already has a shard-level metadata contract through
`distributed_shard_metadata.schema.json`, but recovery and promotion flows still
lack one stated index surface for a whole distributed checkpoint set. Without a
checkpoint index, consumers would have to infer shard membership and checkpoint
roots from directory layout or host-local naming conventions.

## Decision

The minimum distributed checkpoint index is index-only metadata, not a new
runtime checkpoint subsystem. A future checkpoint index must at least make these
fields explicit:

- `artifact_version`
- `checkpoint_root`
- `shard_count`
- `shard_metadata_path`

`shard_metadata_path` must resolve to `distributed_shard_metadata.schema.json`
compatible entries, so the checkpoint index stays layered on the existing shard
contract instead of duplicating ownership and checksum fields.

## Consequences

- Future checkpoint recovery work has one explicit metadata entry point instead
  of inferring from directory layout.
- The index stays aligned with the existing shard metadata contract.
- The repo records the minimum contract now without adding a distributed
  checkpoint runtime prematurely.
