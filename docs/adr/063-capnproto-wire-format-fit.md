# ADR-063: Capnproto-Rust Wire-Format Fit

## Context

Afterburner's current structured contract split is explicit:

- machine-readable artifacts and events stay on JSON
- the inference manifest stays on TOML

That choice already keeps fixtures, schemas, CLI/event tooling, and cross-language
inspection simple. The open question is whether `capnproto-rust` should become a
current wire-format dependency for high-volume runtime or data surfaces, or whether
it should remain only a later option.

Cap'n Proto is attractive for compact binary framing and schema-driven transport, but
adopting it now would add a second structured contract format before the repo has a
measured runtime/data path that actually needs it.

## Decision

Do not adopt `capnproto-rust` now.

- Keep current machine-readable artifacts and events on JSON.
- Keep the inference manifest on TOML.
- Do not add a `capnproto-rust` dependency or a parallel Cap'n Proto contract
  surface at the current stage.
- Reconsider Cap'n Proto only for a concrete later high-volume runtime or data
  surface where measured pressure shows JSON/TOML framing is materially insufficient.
- If revisited later, keep Cap'n Proto adapter-side or surface-specific rather than
  broadening it into a repo-wide default contract format.

## Consequences

- Current fixtures, schemas, and operator workflows stay stable.
- The repo avoids introducing a second structured contract format without measured
  leverage.
- Option space stays open for a later schema-driven binary surface if runtime or
  data throughput pressure becomes concrete.
- Any future Cap'n Proto adoption remains a separate, explicit contract decision.
