# ADR-021: Pretraining Source Registry

## Context

`pretraining_dataset_manifest.schema.json` already carries both `source` and
`source_revision`, but today that still leaves room for `source` to drift into
an ad hoc label rather than a stable corpus identity. If that happens, later
dataset refreshes could reuse the same `source_revision` shape while silently
changing what corpus family the manifest claims to describe.

## Decision

Treat `pretraining_dataset_manifest.schema.json` field `source` as a registry
key, not a free-form display label. `source_revision` continues to identify the
specific approved snapshot within that corpus identity.

The minimum future source registry contract should therefore pin, per entry:

- `source`: the stable registry key referenced by dataset manifests
- `upstream_locator`: the canonical upstream dataset location or resolver
- `license`: the governing license or usage terms identifier
- `approval_status`: whether the source is approved for repo use

Current stance: this decision defines the registry contract boundary only. It
does not add a new registry artifact or parser yet.

## Consequences

- Dataset manifests keep their current shape while gaining a stricter meaning:
  `source` names a registry entry and `source_revision` names the approved
  snapshot within it.
- Future pretraining dataset refreshes have one stable place to pin corpus
  identity and approval metadata instead of relying on local naming habits.
- Registry work can arrive later as a narrow follow-on contract without
  rewriting the current dataset manifest semantics first.
