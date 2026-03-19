# ADR-020: Profiling Hotspot Taxonomy

## Context

The profiling summary contract already records raw hotspot symbols, sample counts,
and percentages. What is still implicit is how operators should interpret those
symbols when triaging performance regressions. Without a small shared taxonomy,
the same hotspot list can be read inconsistently across CLI, dashboard, and
future deployment-side diagnostics.

## Decision

Use this minimum hotspot taxonomy for interpreting profiling summaries:

- `execution`: direct Afterburner execution paths such as
  `afterburner::cmd_infer::run_infer`
- `framework`: Burn or tensor-runtime operations such as
  `burn_tensor::...`
- `compiler`: backend compiler or runtime preparation work such as `cubecl...`
- `incidental`: formatting, allocation, or support work that is observable but
  not a primary optimization target

The raw `symbol` field in the profiling summary remains the source of truth.
This taxonomy is an interpretation layer for triage, not a replacement schema.

## Consequences

- Performance discussions can distinguish between execution, framework,
  compiler/runtime, and incidental hotspots without inventing per-session labels.
- Future dashboards or reports can group hotspots consistently while preserving
  the existing raw profiling artifact contract.
- The repo gains a stable triage vocabulary without coupling itself to one
  profiler implementation.
