# ADR-030: Profiling Provenance Receipt

## Context

The repo now defines profiling environment provenance and writes a
`profiling_environment_snapshot.json` artifact, but it still has no explicit
receipt for when that provenance snapshot was captured or refreshed. Without a
receipt, provenance reviews would have to infer capture timing from surrounding
profiling runs or local logs.

## Decision

Future profiling provenance updates should write one explicit artifact:
`profiling_provenance_receipt.json`

The minimum receipt contract should include:

- `profiling_environment_snapshot.json`
- `profile_kind`
- `profiler_version`
- `captured_at_unix_ms`

Current stance: this decision defines the profiling provenance receipt boundary
only. It does not add a new profiling command yet.

## Consequences

- Future profiling comparisons can distinguish the provenance payload from the
  record of when it was captured or refreshed.
- Profiling workflows gain one explicit audit artifact instead of relying on
  shell logs for provenance capture timing.
- The repo records the receipt vocabulary now while keeping the current snapshot
  contract stable.
