# ADR-058: Distributed Runtime Benchmark Harness

## Context

The repo already has:

- an explicit distributed runtime profile schema
- an explicit layout-feasibility artifact

What is still missing is a reproducible runner that executes a candidate trial,
captures benchmark configuration, and persists the resulting artifact instead of
relying on ad hoc shell scripts.

## Decision

Add a grouped profile command:

- `afterburner profile distributed-runtime-benchmark`

The command:

- executes an explicit benchmark command,
- records benchmark configuration and runtime layout,
- records supplied performance/memory metrics,
- writes a `distributed_runtime_benchmark_run.json` artifact,
- and emits a matching event.

Current stance: this harness standardizes execution and persistence. It does not
claim that the repo can execute every distributed mode listed in the runtime
profile vocabulary.

## Consequences

- Runtime benchmark trials now have one reproducible entrypoint instead of ad
  hoc scripts.
- The benchmark harness stays separate from the later aggregated runtime profile
  contract while reusing the same core fields.
- Later profile synthesis can consume one explicit run artifact instead of
  scraping command output.
