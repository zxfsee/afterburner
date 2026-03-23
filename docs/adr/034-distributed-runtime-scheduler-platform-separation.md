# ADR-034: Distributed Runtime, Scheduler, And Platform Separation

## Context

The repo is growing toward distributed training, GPU scheduling, and multiple
execution environments ranging from a single MacBook to remote NixOS nodes and
later Kubernetes clusters. Without an explicit boundary, scheduler policy can
bleed into model execution concerns, or runtime parallelism choices can become
implicitly tied to one deployment substrate.

The intended system shape has three distinct layers:

- distributed training runtime,
- GPU scheduler / allocator,
- platform / infrastructure substrate.

## Decision

Keep these layers separate.

- The distributed training runtime owns in-job execution semantics:
  DP today, and future TP/PP/SP-CP/EP/ZeRO-style expansion, collectives,
  gradient synchronization, rank/world topology, microbatch execution, and
  checkpoint/optimizer state.
- The GPU scheduler / allocator owns cross-job control-plane concerns:
  queueing, admission, topology-aware placement, node inventory, lease
  ownership, lifecycle, and preemption policy.
- The platform / infrastructure layer owns reproducible environments and launch
  substrate concerns:
  drivers/runtime libraries, node provisioning, and concrete execution surfaces
  such as systemd, NixOS hosts, and Kubernetes.

The only required runtime/scheduler coupling is an explicit lifecycle boundary:

- scheduler to runtime: `START`, `STOP`, `KILL`
- runtime to scheduler: `READY`, `CHECKPOINTED`, `FAILED`, `HEARTBEAT`

The scheduler may assign nodes, GPU ids, resources, and rank assignments, but
it does not interpret TP/PP/ZeRO or model-state internals. The runtime may
choose its internal parallelism strategy, but it does not own cluster
scheduling policy or infrastructure orchestration.

The scheduler side is also a workload-driven capability subset, not a Slurm
parity target. Queueing, admission, placement, leases, lifecycle, and related
fit decisions should grow only as real workloads require them.

Current stance: this ADR defines separation and coupling boundaries only. It
does not define a final job schema, Kubernetes object model, or preemption
protocol beyond the lifecycle split above.

## Consequences

- Single-node systemd execution, remote NixOS execution, and later Kubernetes
  execution can reuse the same runtime and scheduler boundaries instead of
  forcing environment-specific training semantics.
- Runtime evolution across DP/TP/PP/ZeRO-style work remains orthogonal to
  scheduler policy and placement evolution.
- Scheduler work can stay focused on resource allocation and lifecycle without
  embedding model execution details.
- Future contract work should treat lifecycle signals as the narrow integration
  surface between runtime and scheduler rather than inventing cross-layer state
  sharing.
