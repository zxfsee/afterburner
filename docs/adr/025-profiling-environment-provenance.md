# ADR-025: Profiling Environment Provenance

## Context

`profiling_hotspot_summary.schema.json` already records local identity such as
`artifact_version`, `backend`, `profiler`, and `profile_command`. That is enough
to inspect one run in isolation, but not enough to compare hotspot artifacts
across hosts or profiler installations without external notes.

## Decision

Keep the existing profiling summary contract as the anchor surface, but require
future profiling artifacts or attached provenance layers to make at least these
fields explicit:

- `host_os`
- `host_arch`
- `profiler_version`
- `profiler_path`

Current stance: this decision defines the minimum profiling environment
provenance boundary only. It does not add a new runtime command or change the
current profiling summary schema yet.

## Consequences

- Future profiling runs can be compared with an explicit host and profiler
  context instead of relying on unstated local environment assumptions.
- The repo keeps the current hotspot summary contract stable while recording the
  missing provenance fields that later profiling workflows should expose.
- The later profiling environment snapshot work can implement one adapter-side
  output against this boundary instead of inventing fields ad hoc.
