# ADR-016: OpenTelemetry Profiling Fit

## Context

The repo already emits a profiling summary artifact through
`profiling_hotspot_summary.schema.json`. That artifact includes local execution
identity such as `artifact_version`, `backend`, and `profile_command`, but it
does not carry OpenTelemetry distributed tracing or resource fields.

The open question is whether profiling artifacts should adopt explicit
OpenTelemetry distributed-tracing style correlation now, or whether that would pull in premature
telemetry vocabulary and SDK/runtime surface before a concrete consumer exists.

## Decision

- The current profiling summary contract is sufficient for now: `artifact_version`,
  `backend`, `weights_artifact`, and `profile_command` are the identity fields to
  correlate profiling runs with local infer artifacts.
- Do not add OpenTelemetry distributed tracing/resource fields to
  `profiling_hotspot_summary.schema.json` yet.
- Do not add a heavy OpenTelemetry SDK or runtime to produce profiling metadata.
- If later adapter-side workflows need explicit distributed tracing linkage, add that as a
  thin adapter-layer extension to the profiling summary contract rather than as a
  core telemetry subsystem.

## Consequences

- Profiling remains lightweight, inspectable, and offline-friendly.
- Existing profiling artifacts keep stable local identity without implying full
  OpenTelemetry semantic-convention support.
- Future distributed trace correlation remains possible, but only when a concrete workflow
  proves the need.
