# ADR-034: Distributed Runtime Growth Model And Boundaries

## Problem

Afterburner is growing toward distributed training, scheduler control-plane
work, and multiple execution substrates. Without one explicit decision tying
those together, the repo risks three kinds of drift:

- scheduler policy bleeding into model-execution semantics,
- local throughput knobs being misread as distributed-runtime contracts,
- and future distributed work expanding toward framework parity instead of the
  minimum workload-driven capability subset.

The architecture needs one problem-centered decision that defines:

- the runtime / scheduler / platform boundaries,
- the growth model for distributed capability,
- and the current supported runtime capability surface.

## Forces / Constraints

- The current runtime boundary is Burn-aligned.
- `worker_parallelism` is a local throughput knob, not a distributed topology
  or strategy contract.
- Future distributed work must stay explicit about topology, device groups, and
  lifecycle boundaries.
- The repo is not targeting DeepSpeed-class framework parity.
- New capabilities should be promoted only when a real workload, contract, and
  verification path justify them.
- The runtime, scheduler, and infrastructure layers must remain separable across
  single-node and future cluster substrates.

## Decision

Use a workload-driven distributed runtime growth model with explicit runtime /
scheduler / platform boundaries.

### Boundary model

Keep these three layers separate:

- distributed training runtime
- GPU scheduler / allocator
- platform / infrastructure substrate

The distributed training runtime owns in-job execution semantics: DP today, and
future TP/PP/SP-CP/EP/ZeRO-style expansion, collectives, gradient
synchronization, topology, microbatch execution, and checkpoint / optimizer
state.

The GPU scheduler / allocator owns cross-job concerns: queueing, admission,
placement, inventory, lease ownership, lifecycle, and preemption policy.

The platform / infrastructure layer owns reproducible environments and launch
substrates: drivers, host provisioning, systemd/NixOS execution, and later
cluster substrates.

The only required scheduler/runtime coupling is the explicit lifecycle boundary:

- scheduler to runtime: `START`, `STOP`, `KILL`
- runtime to scheduler: `READY`, `CHECKPOINTED`, `FAILED`, `HEARTBEAT`

### Growth model

Distributed runtime work is capability-subset work, not framework-parity work.

- Implement the minimum capability surface required by real workloads.
- Treat DeepSpeed-class systems as sources of patterns, not parity targets.
- Promote features only when they:
  - are required by a real target workload,
  - fit the Burn-side runtime layer,
  - have an explicit contract boundary,
  - can be profiled or validated,
  - and do not force broad generic support for unrelated combinations.

Burn's strategy-oriented distributed surfaces are the right conceptual alignment
for future in-job distributed execution. Afterburner should not overload
`worker_parallelism` to mean distributed strategy or topology.

### Current capability surface

Supported now:

- single-device execution by default
- a guarded explicit single-node data parallel execution path for training when `world_size`,
  `device_group`, and concrete participant devices are declared explicitly
- local CPU, `wgpu`, and `metal` backend selection
- existing training and inference artifact contracts

Unsupported now:

- `ZeRO-1/2/3`
- tensor parallelism (`TP`)
- pipeline parallelism (`PP`)
- sequence/context parallelism (`SP/CP`)
- expert parallelism (`EP`)
- generic multi-axis composition across those modes

## Consequences

- Single-node and future cluster substrates can reuse one runtime/scheduler
  boundary instead of embedding environment-specific training semantics.
- The repo keeps `worker_parallelism` honest as a local throughput knob.
- Profiling and planning stay tied to one explicit supported/unsupported runtime
  surface instead of inferred capability claims.
- Future distributed work must promote one capability at a time instead of
  silently widening the surface.

## Alternatives Considered

### Collapse scheduler and runtime semantics into one subsystem

Rejected because it would blur execution semantics, placement policy, and
infrastructure concerns into one coupled control surface.

### Treat local throughput settings as the distributed contract

Rejected because `worker_parallelism` is not enough to describe topology,
strategy, or device-group semantics.

### Pursue broad framework parity up front

Rejected because it would widen the repo faster than the workload, contract,
and verification surfaces justify.

## Current Implementation Mapping

- current runtime capability: single-device by default plus a guarded explicit
  single-node DP train path
- current backend surface: CPU / `wgpu` / `metal`
- current explicit distributed execution artifact:
  `distributed_runtime_execution.json`
- current local-only throughput knob: `worker_parallelism`
- later explicit capability candidates: `ZeRO`, `TP`, `PP`, `SP/CP`, `EP`

## Revisit Triggers

- a real workload requires executed multi-device runtime support
- topology or device-group metadata becomes explicit in a new contract
- scheduler/runtime lifecycle needs additional signals beyond the current narrow
  boundary
- measured runtime profiles justify promoting a new distributed capability
