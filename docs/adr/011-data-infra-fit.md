# ADR-011: Arrow/DataFusion/Ballista/Parquet Fit

## Context

The active roadmap includes possible future work around larger dataset artifacts,
evaluation outputs, deployment metadata, and distributed execution. Apache Arrow,
Parquet, DataFusion, and Ballista are plausible candidates for those surfaces,
but adopting them prematurely would expand the dependency and runtime surface
before a concrete need exists.

## Decision

- `Parquet` is the best current candidate for future offline, tabular dataset and
  evaluation artifacts where columnar storage and predicate-friendly reads would
  materially reduce I/O or schema churn.
- `Arrow IPC` is an acceptable in-memory or interchange format if a later adapter
  needs zero-copy tabular exchange, but it is not the default artifact contract.
- `DataFusion` is a future in-process query/runtime option only if Afterburner
  grows a real analytical or ETL layer over those columnar artifacts.
- `Ballista` is out of scope until there is a real distributed query or ETL
  problem that cannot be handled by simpler artifact workflows.

Current stance: no dependency is added yet. Existing JSON/TOML contracts remain
the default until a specific dataset/eval/deployment artifact demonstrates that
columnar storage or query execution would remove a measured bottleneck or a real
schema-management burden.

## Consequences

- Columnar technologies are acknowledged as likely fits for some future artifact
  classes without forcing them into the current runtime or core contract.
- The repo keeps option space open for Parquet and Arrow-based artifacts later.
- DataFusion and Ballista remain explicit future decisions, not silent framework
  drift.
