# ADR-056: GPU Colocation Fit

## Context

The repo already records that GPU colocation is distinct from preemption, but it
still needs one explicit decision about when colocation is even a fit for small
clusters. Without that fit gate, scheduler work could treat sharing as a
default policy rather than a constrained optimization for known low-saturation
workloads.

## Decision

GPU colocation is a fit only as an explicitly constrained scheduling mode.

- Colocation is only reasonable for workloads that are known not to saturate a
  whole GPU.
- Colocation must be opt-in and workload-scoped, not the default scheduler
  policy.
- Colocation should be evaluated with explicit interference limits, especially:
  - latency regression
  - throughput regression
  - memory pressure
- Colocation is not a substitute for checkpoint-based preemption.

Current stance: this ADR defines fit and constraints only. It does not add a
colocation implementation yet.

## Consequences

- The scheduler remains queue-first by default instead of drifting into
  opportunistic sharing everywhere.
- Later colocation work can target one explicit constraint model instead of
  treating all jobs as shareable.
- Colocation stays separate from preemption and from the core distributed
  runtime surface.
