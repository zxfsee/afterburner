# ADR-015: Multibillion-Scale Target Envelope

## Context

The repo already has several explicit contracts that touch pieces of larger-scale
operation: `training_scalability_contract.json`, `distributed_shard_metadata.schema.json`,
manifest precision metadata, rollout ownership, upload planning, and deployment
target profiles. What is still missing is one stated envelope for how those
pieces fit together when the eventual target grows to multibillion-parameter
training and deployment.

Without that envelope, future sharding, checkpointing, precision, and rollout
changes could drift independently and force an accidental runtime architecture.

## Decision

The minimum multibillion-scale target envelope is contract-first and consists of:

- training-side throughput and worker assumptions remaining explicit through
  `training_scalability_contract.json`,
- shard ownership, indices, and checksums remaining explicit through
  `distributed_shard_metadata.schema.json`,
- inference artifact precision remaining explicit in the manifest `[precision]`
  contract instead of being inferred from backend behavior,
- rollout authority remaining explicit through
  `artifact_rollout_ownership.schema.json`,
- upload planning remaining explicit through
  `artifact_upload_request.schema.json`,
- and deployment targeting remaining explicit through
  `deployment_target_profile.schema.json`.

Current stance: the repo is not claiming multibillion readiness at runtime yet.
It is declaring the minimum contract envelope that future work must preserve so
larger-scale training, inference, and deployment can evolve intentionally.

## Consequences

- Future large-scale work has a concrete contract checklist before runtime or
  orchestration complexity expands.
- Precision, sharding, and rollout changes now have to stay aligned instead of
  evolving as separate local optimizations.
- The repo keeps option space open for future checkpoint and deployment work
  without committing to a distributed runtime implementation prematurely.
