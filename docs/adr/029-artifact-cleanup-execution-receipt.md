# ADR-029: Artifact Cleanup Execution Receipt

## Context

The cleanup path now has an inventory artifact and a dry-run receipt boundary,
but it still has no explicit audit artifact for a real cleanup execution. Without
that receipt, future removal steps would have to rely on shell logs to explain
what was actually deleted or skipped.

## Decision

Cleanup runs now write one explicit artifact:
`artifact_cleanup_execution_receipt.json`

The minimum receipt contract should include:

- `artifact_cleanup_dry_run_receipt.json`: the dry-run input it executed from
- `policy_profile`: the cleanup policy profile applied to the run
- `evidence_sources`: references back to the dry-run receipt plus its inventory
  and policy inputs
- `removed_paths`: the paths actually removed
- `skipped_paths`: the paths intentionally not removed during execution
- `executed_at_unix_ms`: when the cleanup execution was recorded

Current stance: the repo now materializes this receipt through
`afterburner cleanup execute`. Execution stays strictly downstream of an
approved dry-run receipt.

## Consequences

- Future cleanup runs can be audited from one explicit receipt instead of raw
  terminal output.
- Removal and skip reviews can trace execution back to the approved dry-run
  plan and its inventory/policy inputs without consulting raw logs.
- Execution remains layered on the existing inventory and dry-run contracts
  instead of inventing a parallel cleanup vocabulary.
- The repo records the execution audit surface now before any destructive
  cleanup behavior is introduced.
