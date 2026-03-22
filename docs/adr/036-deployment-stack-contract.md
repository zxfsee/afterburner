# ADR-036: Deployment Stack Contract

## Context

The repo already has an explicit deployment target profile, deploy-rs baseline,
upload planning, and local rollout recipes. But the deployable unit is still
underspecified: deployment can resolve a host and artifact root without stating
the service entrypoint, service surface, rollout pointer, or observability hook
that make up the full Afterburner runtime stack.

Without that stack contract, deployment risks treating the model file as the
whole deployable system instead of the application plus its rollout and
observability surfaces.

## Decision

Add an explicit deployment stack contract.

- `fixtures/deployment_stack_profile.example.json` is the source-of-truth stack
  profile fixture.
- The contract includes:
  - `deployable_unit`
  - `service_entrypoint`
  - `service_surface`
  - `artifact_roots`
  - `rollout_entrypoint`
  - `observability`
- `afterburner deployment-stack-check` validates the deployment target profile
  plus the deployment stack profile and writes a checked
  `deployment_stack_check.json` artifact.
- `just deploy-check` must validate both the deploy-rs baseline and the stack
  contract so deployment covers service surface, artifact roots, rollout
  entrypoint, and observability hooks together.

Current stance: this ADR defines the deployment stack contract for the current
Afterburner HTTP service surface. It does not introduce a long-running deploy
service or provider-specific deployment logic.

## Consequences

- Deployment now treats the full runtime stack as the deployable unit instead of
  reducing deployment to model-file placement.
- Service entrypoint, rollout pointer, and observability wiring become explicit,
  inspectable contract data.
- Future deployment adapters can consume one checked stack artifact instead of
  rediscovering runtime layout from recipes or host-local conventions.
