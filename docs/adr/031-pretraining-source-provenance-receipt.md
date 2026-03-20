# ADR-031: Pretraining Source Provenance Receipt

## Context

The source approval receipt now records approval state and operator identity,
but it still does not say which reviewed source metadata entry the approval was
based on. Without explicit provenance references, later dataset or registry
workflows would have to reconstruct approval context from surrounding process
logs.

## Decision

Keep `pretraining_source_approval_receipt.json` as the anchor artifact, but
require future source approval receipts to make provenance references explicit
through at least:

- `registry_entry_path`
- `upstream_locator`
- `reviewed_metadata_sha256`

Current stance: this decision defines the source provenance receipt boundary
only. It does not add a source registry artifact or a receipt-writing command
yet.

## Consequences

- Source approval receipts can point back to the reviewed source metadata
  without embedding the full source record directly.
- Later dataset and registry workflows can trace approvals from one explicit
  provenance vocabulary instead of inferring review context from logs.
- The repo records the provenance boundary now while keeping the current source
  approval receipt contract narrow.
