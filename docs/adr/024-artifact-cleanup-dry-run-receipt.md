# ADR-024: Artifact Cleanup Dry-Run Receipt

## Context

The repo now has `artifact_cleanup_inventory.json`, which distinguishes retained
paths from prune candidates under the current retention envelope. What is still
missing is one explicit artifact for the operator-facing dry-run decision: what
would be removed, how many retained items were intentionally skipped, and when
the preview was generated.

## Decision

Future cleanup dry-runs should write one explicit artifact:
`artifacts/deploy/artifact_cleanup_dry_run_receipt.json`

The minimum receipt contract should include:

- `inventory_path`: the `artifact_cleanup_inventory.json` input the dry-run was
  evaluated against
- `planned_removals`: the paths that the dry-run would remove
- `retained_count`: how many retained paths were intentionally kept
- `prune_candidate_count`: how many prune candidates were reviewed
- `generated_at_unix_ms`: when the dry-run receipt was produced

Current stance: this decision defines the dry-run receipt boundary only. It does
not add a cleanup command or deletion behavior yet.

## Consequences

- Cleanup review can converge on one explicit receipt instead of ad hoc terminal
  output.
- The dry-run layer stays downstream of the existing cleanup inventory contract
  instead of inventing a second artifact classification surface.
- Future cleanup automation can add behavior later without changing the minimum
  review vocabulary first.
