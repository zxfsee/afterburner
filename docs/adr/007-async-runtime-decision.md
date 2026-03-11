# ADR-007: Async Runtime Decision

## Context

The repository has an optional HTTP adapter, but it currently serves requests
through a synchronous `tiny_http` path and keeps core logic runtime-agnostic.
Introducing Tokio would add a new runtime dependency, change adapter execution
model assumptions, and expand the build and operational surface.

## Decision

No async runtime is adopted at this stage.

The current adapter baseline remains:

- synchronous HTTP serving via `tiny_http`
- runtime-agnostic core logic
- telemetry/exporter concerns isolated to adapters if they are introduced later

Tokio is deferred until profiling or deployment evidence shows that the current
adapter model is insufficient for required concurrency, latency, or integration
needs.

## Consequences

- Adapter code stays simpler and avoids a premature runtime commitment.
- Future Tokio adoption remains possible, but must be justified by measured
  evidence rather than convenience alone.
- Any eventual async runtime remains an adapter concern, not a core dependency.
