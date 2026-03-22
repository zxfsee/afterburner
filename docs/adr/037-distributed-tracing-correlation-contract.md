# ADR-037: Distributed Tracing Correlation Contract

## Context

The deployable Afterburner stack now spans rollout checks, deploy-stack
validation, load-profile workflows, and HTTP service traffic. Existing adapter
events already expose local identity such as `artifact_version`, `request_id`,
and profile paths, but they do not yet share one explicit correlation surface
across commands and service requests.

The repo also keeps an explicit no-heavy-runtime stance: distributed tracing
correlation should not force a Tokio or OpenTelemetry SDK adoption before a
concrete need justifies that larger runtime change.

## Decision

Add a thin adapter-layer distributed tracing correlation contract.

- Use W3C `traceparent` as the explicit propagation format.
- HTTP service requests may supply `traceparent`; if absent, the adapter
  generates one and echoes it in the response headers.
- Adapter events and relevant deployment/load-test artifacts carry:
  - `traceparent`
  - `trace_id`
- CLI/deployment workflows may propagate one trace across commands through
  `AFTERBURNER_TRACEPARENT`.
- Keep this correlation surface in adapters and artifacts only; do not add a
  Tokio runtime or heavy OpenTelemetry SDK as part of this change.

Current stance: this ADR defines correlation and propagation fields only. It
does not claim full tracing semantics, span export, or OpenTelemetry runtime
integration.

## Consequences

- Load tests, rollout checks, and HTTP traffic can share one explicit
  correlation identity across adapter events and artifacts.
- Deployment-side artifacts can be reviewed together by `trace_id` without
  inferring command grouping from timestamps or filenames alone.
- The repo preserves the current synchronous/runtime-light architecture while
  keeping a future Tokio/tracing move possible if scale and workflow pressure
  justify it later.
