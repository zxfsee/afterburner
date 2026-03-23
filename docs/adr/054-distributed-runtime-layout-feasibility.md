# ADR-054: Distributed Runtime Layout Feasibility

## Context

The repo now has explicit topology vocabulary, runtime capability bounds, and a
future runtime profile schema. What is still missing is one explicit ops-level
artifact for deciding whether a candidate distributed layout is even valid
before any benchmark or training launch.

Without that gate, later profile and benchmark work could waste time on
divisibility, topology, or inventory combinations that are invalid on their
face.

## Decision

Define one explicit layout-feasibility artifact:

- `distributed_runtime_layout_feasibility.schema.json`

The minimum artifact records:

- inventory shape:
  - `node_count`
  - `gpus_per_node`
  - `world_size`
- candidate runtime layout:
  - `dp`
  - `tp`
  - `pp`
  - `sp_cp`
  - `ep`
  - `gas`
  - `microbatch_size`
  - `zero_stage`
- feasibility result:
  - `feasible`
  - `checks`

Current stance: this ADR defines the feasibility artifact only. It does not add
the validator command or benchmark harness yet.

## Consequences

- Later benchmark/profile work can reject obviously invalid layouts before
  spending runtime effort.
- Layout validation gets one explicit artifact shape instead of ad hoc command
  logic.
- The feasibility gate stays separate from both the runtime capability list and
  the executed runtime profile artifact.
