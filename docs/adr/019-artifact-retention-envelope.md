# ADR-019: Artifact Retention Envelope

## Context

The repository now produces train, infer, eval, deploy, profiling, and drift
management artifacts under `artifacts/`. Without one stated retention envelope,
operators would have to guess which files are required for active runtime,
rollback, and auditability, and which files can be pruned after they are
superseded.

## Decision

Keep this minimum artifact retention envelope:

- retain `artifacts/inference/current`,
- retain the referenced `artifacts/inference/<version>/` artifact directory for
  the active runtime contract,
- retain active rollout and rollback evidence under `artifacts/deploy/`,
- retain active train-side contract artifacts under `artifacts/train/` that
  drive current operational decisions,
- and retain the current approved drift baseline state under `artifacts/eval/`
  for rollout comparison.

Prune candidates are artifacts that are explicitly superseded and not referenced
by the current runtime or active rollout state, especially under
`artifacts/profiling/` and older transient summaries or receipts in
`artifacts/eval/`.

Current stance: this decision defines the retention envelope only. It does not
add automatic prune commands yet.

## Consequences

- Operators get one explicit rule for what must stay available to preserve
  runtime, rollback, and auditability.
- Profiling and superseded eval artifacts can be pruned deliberately instead of
  accumulating by default.
- Future cleanup automation has a contract boundary to implement against rather
  than inferring importance from directory names.
