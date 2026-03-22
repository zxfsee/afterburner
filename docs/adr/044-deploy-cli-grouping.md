# ADR-044: Deploy CLI Grouping

## Context

The repo already groups several operator-heavy workflows under `drift`,
`cleanup`, and `profile`, but deployment-side verbs still appear as unrelated
top-level commands. That leaves the deployment surface inconsistent even though
upload planning, stack checks, verification bundles, and load profiles all
belong to the same operator-facing domain.

## Decision

Group deployment-side commands under one `deploy` family.

- `afterburner deploy upload`
- `afterburner deploy stack-check`
- `afterburner deploy verification-bundle`
- `afterburner deploy load-profile`

Keep `train`, `infer`, and `eval` top-level.

Current stance: this is a cutover to the grouped deploy CLI surface. The old
flat top-level deploy commands are not retained as the primary contract.

## Consequences

- Deployment-oriented workflows now share one explicit CLI namespace instead of
  a scattered set of top-level verbs.
- Just recipes, docs, and tests can refer to one deployment command family.
- The deploy surface matches the repo's grouped command style instead of
  remaining a special case.
