# ADR-051: Remote Model Save/Load Fit

## Context

The repo already has explicit local artifact manifests, rollout ownership, and a
provider-neutral upload request contract. What it still lacks is one explicit
decision about remote model save/load beyond local filesystem paths.

The important boundary is architectural: remote storage concerns should stay in
adapters and contracts, not leak storage-SDK choices into core artifact logic.

## Decision

Remote model save/load is a fit, but only as an adapter-first locator contract.

- Define one explicit remote artifact locator surface:
  - `provider`
  - `locator`
  - `mode`
- Keep remote save/load out of core model logic.
- Do not bind core or artifact contracts directly to one storage SDK.
- Future provider implementations should translate between this locator contract
  and concrete remote APIs.

Current stance: this ADR defines fit and the minimum locator contract only. It
does not add a remote loader/writer implementation yet.

## Consequences

- The repo can support remote model save/load later without collapsing into one
  provider's API vocabulary.
- Core artifact logic remains local/path-based and deterministic.
- Future remote adapters have one explicit contract to consume instead of
  inventing transport semantics ad hoc.
