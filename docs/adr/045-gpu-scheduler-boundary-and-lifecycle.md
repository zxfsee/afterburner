# ADR-045: GPU Scheduler Boundary And Lifecycle

## Context

The repo already separates runtime, scheduler, and platform layers, and it now
records a conservative scheduler stance around checkpoint-based preemption and
colocation. What is still missing is one explicit contract for the lifecycle
messages that cross the scheduler/runtime boundary.

Without that contract, queueing, admission, lease ownership, topology-aware
placement, and runtime lifecycle could drift into ad hoc command and event
shapes.

## Decision

Define one explicit scheduler/runtime lifecycle message contract:

- `gpu_scheduler_lifecycle_message.schema.json`

The contract carries:

- `direction`
- `kind`
- `job_id`
- `lease_id`

For `START`, it also carries explicit `resources` with:

- `node_ids`
- `gpu_ids`
- `rank_assignments`

The lifecycle boundary remains:

- scheduler to runtime: `START`, `STOP`, `KILL`
- runtime to scheduler: `READY`, `CHECKPOINTED`, `FAILED`, `HEARTBEAT`

Current stance: this ADR defines the boundary contract only. It does not add a
running scheduler or runtime integration yet.

## Consequences

- Queueing/admission/lease ownership/topology-aware placement stay owned by the
  scheduler side of the boundary.
- Runtime lifecycle reporting stays explicit without leaking model-state
  internals into scheduler policy.
- Future single-node and Kubernetes scheduler work can reuse one lifecycle
  contract instead of inventing environment-specific message shapes.
