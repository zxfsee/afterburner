# ADR-060: JSON Over RON Fit

## Context

Afterburner already uses JSON pervasively for machine-readable artifacts and
events, while the inference manifest stays in TOML. The repo has a large
fixture, schema, CLI, and event surface built around `serde_json`.

RON would be more Rust-centric, but switching current artifact or event
contracts to RON would create broad contract churn without a demonstrated
cross-surface benefit.

## Decision

Keep JSON as the default structured artifact and event format.

- Machine-readable artifact and event contracts stay on JSON.
- The inference manifest stays on TOML.
- In other words: machine-readable artifact and event contracts stay on JSON,
  and the inference manifest stays on TOML.
- Do not introduce a broad JSON-to-RON migration for current artifact or event
  surfaces.
- Reconsider RON only for a clearly isolated, human-edited, Rust-local surface
  if JSON or TOML prove materially insufficient there.

## Consequences

- Existing fixtures, schemas, and event tooling remain stable.
- Cross-language readability and generic tooling remain straightforward.
- The repo avoids broad contract churn for a format change without measured
  leverage.
