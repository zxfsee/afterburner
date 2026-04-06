# ADR-051: Artifact Contract And Serialization Strategy

## Problem

Afterburner already has explicit local artifact manifests, rollout ownership,
and provider-neutral upload planning. The remaining architectural question is
how artifact packaging, remote locator surfaces, and serialization choices
should evolve without fragmenting the contract layer.

The problem is not whether one tool or format is individually appealing. It is
how to keep:

- machine-readable artifacts and events stable,
- remote artifact access provider-neutral,
- and post-training optimization/package work explicit,

without widening into multiple contract formats or provider-specific vocabularies.

## Forces / Constraints

- Current structured artifact and event contracts already use JSON broadly.
- The inference manifest already uses TOML and is a separate contract surface.
- Core artifact logic must stay provider-neutral and deterministic.
- Remote save/load should not leak storage-SDK choices into core logic.
- Post-training optimization and packaging are a separate pipeline from runtime
  execution.
- Introducing a second general-purpose contract format now would create broad
  fixture/schema/operator churn without measured leverage.

## Decision

Use one explicit artifact-contract and serialization strategy centered on JSON
artifacts/events, TOML manifests, provider-neutral locators, and a separate
post-training packaging pipeline.

### Serialization defaults

- machine-readable artifact and event contracts stay on JSON.
- The inference manifest stays on TOML.
- Do not introduce a broad JSON-to-RON migration for current artifact or event
  surfaces.
- Do not adopt `capnproto-rust` now or add a parallel Cap'n Proto contract
  surface.

### Remote locator boundary

- Define one explicit remote artifact locator surface:
  - `provider`
  - `locator`
  - `mode`
- Keep remote save/load out of core model logic.
- Keep provider implementations as adapters over this locator contract rather
  than binding core contracts directly to one storage SDK.

### Optimization and packaging boundary

- Treat model optimization and packaging as a separate post-training artifact
  pipeline.
- Keep the planning/profile and package contract explicit through:
  - `model_optimization_profile.schema.json`
  - `optimized_model_package_contract.schema.json`
- Keep these package-contract fields explicit:
  - `export_format`
  - `packaging_inputs`
  - `package_layout`
  - `output_constraints`
- Keep `optimized_model_capability_surface.schema.json` explicit.
- quantization remains part of the explicit optimization planning surface.
- Quantization, compression, export, and packaging remain planning-only today.
- Keep reduced-precision and compressed optimized artifacts unsupported until
  dedicated runtime contracts land.

## Consequences

- Fixtures, schemas, and operator workflows stay on one stable structured
  contract model.
- The repo avoids broad contract churn from adding RON or Cap'n Proto without a
  proved need.
- Remote artifact access can evolve without collapsing into one provider's API
  vocabulary.
- Optimization, compression, export, and packaging work can evolve as a
  dedicated artifact pipeline rather than leaking into runtime semantics.

## Alternatives Considered

### Broad JSON-to-RON migration

Rejected because it would create wide contract churn without a demonstrated
cross-surface benefit.

### Early Cap'n Proto adoption

Rejected because there is no current measured runtime or data path that needs a
second schema-driven binary contract format.

### Provider-specific remote storage contracts

Rejected because they would couple core artifact logic to one storage SDK or
remote path vocabulary.

### Collapse packaging into runtime contracts

Rejected because post-training optimization/package work is a distinct leverage
axis and should stay explicit as its own artifact pipeline.

## Current Implementation Mapping

- structured artifacts/events: JSON
- inference manifest: TOML
- remote access surface: provider-neutral locator contract
- optimization/package planning: explicit profile plus package contract
- non-adopted formats: RON for broad contracts, Cap'n Proto for current runtime
  or data surfaces

## Revisit Triggers

- a concrete isolated surface shows JSON or TOML is materially insufficient
- a high-volume runtime or data path shows measured pressure for a binary schema
  format
- a provider-neutral locator contract is no longer sufficient for remote access
- optimized artifact packaging needs a dedicated runtime contract for reduced
  precision or compressed outputs
