# ADR-041: Distributed Runtime Capability Surface

## Context

The repo now has explicit decisions for distributed learning-strategy fit and
world/rank topology vocabulary, but it still lacks one statement of what the
runtime can and cannot actually execute today. Without that capability surface,
profiling work and future backlog items could imply support for configurations
the current Burn/Afterburner stack does not provide.

## Decision

The current distributed runtime capability surface is intentionally narrow.

Supported now:

- single-device execution only
- local CPU, `wgpu`, and `metal` backend selection
- existing training/inference artifact contracts

First candidate expansion:

- data parallel execution, after explicit topology, collectives, and checkpoint
  contracts are in place

Unsupported now:

- ZeRO-1/2/3
- tensor parallelism (TP)
- pipeline parallelism (PP)
- sequence/context parallelism (SP/CP)
- expert parallelism (EP)
- generic multi-axis composition across those modes

Current stance: this ADR defines the runtime capability boundary only. It does
not add a distributed execution path or claim that Burn's broader training
strategy vocabulary is already implemented in-repo.

## Consequences

- Profiling and planning must not assume distributed execution modes the repo
  cannot currently run.
- The later layout/profile gates can key off one explicit supported/unsupported
  list instead of inferring support from backlog titles or external framework
  features.
- Future distributed runtime work must intentionally promote one capability at a
  time instead of silently widening the surface.
