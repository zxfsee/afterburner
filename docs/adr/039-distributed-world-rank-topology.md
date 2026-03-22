# ADR-039: Distributed World And Rank Topology

## Context

The current repo has `training_scalability_contract.json` and
`distributed_shard_metadata.schema.json`, but neither contract defines a full
distributed execution topology. `worker_parallelism` is explicitly local-only,
and shard ownership alone is not enough to describe world-size, rank
assignment, or device-group structure for future Burn-side distributed
training.

Without explicit topology fields, later distributed execution would have to
infer semantics from local worker ids, shard filenames, or host-local process
layout.

## Decision

Future Burn-side distributed training should define one explicit topology
contract before claiming multi-device execution semantics.

The minimum topology surface should make these fields explicit:

- `world_size`
- `global_rank`
- `local_rank`
- `node_rank`
- `device_group`

Current stance:

- `training_scalability_contract.json` remains a local throughput contract.
- `distributed_shard_metadata.schema.json` remains a shard-ownership contract.
- Neither current contract should be treated as the topology source of truth.
- A later topology-aware artifact or ops contract should carry these fields
  explicitly instead of deriving them from `owner_worker_id`,
  `owner_worker_rank`, or shard layout.

This ADR defines the minimum topology vocabulary only. It does not add a new
schema fixture, parser, or distributed runtime implementation yet.

## Consequences

- Future distributed runtime work gets one explicit topology vocabulary instead
  of inferring execution structure from local process names or shard metadata.
- Shard ownership and execution topology remain separate concerns.
- The later distributed runtime capability gate can rely on an explicit
  topology contract instead of overloaded local worker fields.
