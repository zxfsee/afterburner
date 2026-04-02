# ADR-044: Deploy CLI Grouping

## Context

The repo already groups several operator-heavy workflows under `drift`,
`cleanup`, and `profile`, but deployment-side verbs still appear as unrelated
top-level commands. That leaves the deployment surface inconsistent even though
upload planning, stack checks, launch actions, heartbeat emission, and other
deploy-start operations belong to the same operator-facing domain. At the same
time, verification and rollback actions read more clearly as operator intents
than as deploy subcommands.

## Decision

Keep deploy-start commands under one `deploy` family, and move verification and
rollback actions to their own intent families.

- `afterburner deploy upload`
- `afterburner deploy stack-check`
- `afterburner deploy scheduler-heartbeat`
- `afterburner deploy load-profile`
- `afterburner verify receipt`
- `afterburner verify bundle`
- `afterburner rollback verification-bundle`

Keep `train`, `infer`, and `eval` top-level.

Current stance: deploy remains the public family for deploy-start actions. The
old deploy verification and rollback names are not retained as the primary
operator contract; low-level artifact mutation stays behind `afterburner debug
deploy ...`.

## Consequences

- Deploy-start workflows now share one explicit CLI namespace instead of a
  scattered set of top-level verbs.
- Verification and rollback read as operator intents instead of artifact-taxonomy
  deploy subcommands.
- Just recipes, docs, and tests can distinguish public operator actions from
  low-level debug surfaces.
