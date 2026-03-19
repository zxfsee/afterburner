# ADR-022: Deployment Verification Receipt

## Context

The repo already defines rollout ownership, upload planning, deploy-target
profiles, and local promotion/rollback orchestration. What is still missing is
one explicit artifact proving that post-promote verification actually ran and
what it concluded. Without that receipt, rollout verification evidence remains
split across raw logs and ad hoc command output.

## Decision

Future deployment verification should write one explicit artifact:
`artifacts/deploy/<artifact_version>/deployment_verification_receipt.json`

and emit one matching event:
`deployment_verification_receipt_written`

The minimum receipt contract should include:

- `artifact_version`: the promoted artifact being verified
- `profile_name`: the deployment target profile or logical target being checked
- `verification_status`: pass or fail
- `verified_at_unix_ms`: when the verification decision was recorded
- `evidence`: references to the concrete checks or artifacts that justified the
  verification outcome

Current stance: this decision defines the verification receipt boundary only. It
does not add a new runtime command or deployment adapter yet.

## Consequences

- `rollout-verify` has a clear future output contract instead of relying on raw
  terminal logs as its long-term audit surface.
- Deployment-side verification can be reviewed from one artifact/event pair
  without reconstructing the outcome from unrelated rollout evidence.
- Future deployment automation can extend the existing promotion flow without
  inventing a second verification vocabulary.
