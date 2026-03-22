# ADR-035: Distributed Training Capability Subset Scope

## Context

The repo is adding distributed-training runtime work, scheduler work, and
platform work incrementally. That creates a scope risk: future TODOs can drift
from “implement the minimum capability required by current workloads” toward
“rebuild a general distributed training framework.”

The current project target is narrower than DeepSpeed-class parity. It needs a
Rust/Burn-first distributed runtime, explicit contracts, reproducible
infrastructure, and measured capability growth tied to actual workloads.

## Decision

Distributed-training runtime work is capability-subset work, not framework-parity work.

- Implement the minimum distributed-training capability surface required by repo
  workloads, in dependency order.
- Treat DeepSpeed-class systems as sources of patterns and design ideas, not as
  parity targets.
- Prioritize features only when they:
  - are needed by a real target workload,
  - fit the Burn-side runtime layer,
  - have a clear contract boundary,
  - can be profiled or validated,
  - and do not force generic support for unrelated combinations.

Current subset shape:

- Phase 1: DP, world/rank topology, process groups/collectives, checkpoint/state
  contract, profile schema/harness, and the single-node scheduler lifecycle path.
- Phase 2: ZeRO-1, and maybe ZeRO-2, plus an explicit capability surface driven
  by measured profiles.
- Phase 3: selective TP or PP only for model families and layouts the repo
  actually uses.

Current stance: this ADR defines scope and sequencing rules only. It does not
commit the repo to broad support for all DP/TP/PP/SP-CP/EP/ZeRO combinations or
to generic framework parity.

## Consequences

- Distributed runtime work stays tied to concrete workloads and measured need.
- The repo can borrow proven ideas from existing frameworks without taking on
  their full surface area.
- Future TODOs should justify new parallelism features by workload, contract,
  and verification, not by parity pressure.
