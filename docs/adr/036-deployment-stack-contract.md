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
- `afterburner deploy stack-check` validates the deployment target profile
  plus the deployment stack profile and writes a checked
  `deployment_stack_check.json` artifact.
- `afterburner deploy stack-launch-plan` resolves the deployment target profile
  plus the deployment stack profile into one checked
  `deployment_stack_launch_plan.json` artifact.
- `afterburner deploy stack-launch-receipt` materializes the launch-time
  parameters over one checked launch plan as
  `deployment_stack_launch_receipt.json`.
- `afterburner deploy stack-launch-bundle` packages the launch plan plus launch
  receipt into one `deployment_stack_launch_evidence_bundle.json` artifact so
  downstream rollout tooling can consume a single stack launch entrypoint.
- `afterburner deploy reconcile-launch-bundle` exports one stable
  `deployment_stack_launch_evidence_bundle_reconciliation.json` artifact so
  downstream deployment tooling can compare desired plan-and-receipt-resolved
  bundle state against the current bundle without ad hoc shell checks.
- `afterburner deploy stack-launch-handoff` exports one compact
  `deployment_stack_launch_evidence_handoff.json` handoff artifact over the
  current launch evidence bundle for downstream transport or deployment tools.
- `afterburner deploy reconcile-launch-handoff` exports one stable
  `deployment_stack_launch_evidence_handoff_reconciliation.json` artifact so
  downstream deployment tooling can compare desired bundle-resolved handoff
  state against the current handoff without ad hoc shell checks.
- `afterburner deploy record-launch-handoff-history` appends one compact
  `deployment_stack_launch_evidence_handoff_history.json` artifact over handoff
  changes for downstream deployment audit tooling.
- `afterburner deploy record-launch-handoff-reconciliation-history` appends one
  compact `deployment_stack_launch_evidence_handoff_reconciliation_history.json`
  artifact over desired-versus-current handoff updates for downstream
  deployment audit tooling.
- `afterburner deploy point-launch-transport-locator` exports one stable
  `deployment_stack_launch_transport_locator.json` locator over the current
  launch handoff for transport-facing consumers.
- `afterburner deploy record-launch-transport-locator-history` appends one
  compact `deployment_stack_launch_transport_locator_history.json` artifact
  over transport-locator changes for downstream deployment audit tooling.
- `afterburner deploy reconcile-launch-transport-locator` exports one stable
  `deployment_stack_launch_transport_locator_reconciliation.json` artifact so
  downstream deployment tooling can compare desired handoff-resolved locator
  state against the current transport locator without ad hoc shell checks.
- `afterburner deploy record-launch-transport-locator-reconciliation-history`
  appends one compact
  `deployment_stack_launch_transport_locator_reconciliation_history.json`
  artifact over desired-versus-current transport locator updates for downstream
  deployment audit tooling.
- `afterburner deploy point-launch-locator` exports one stable
  `deployment_stack_launch_locator_pointer.json` current-pointer artifact over
  the current launch transport locator.
- `afterburner deploy reconcile-launch-locator` exports one stable
  `deployment_stack_launch_locator_reconciliation.json` artifact so downstream
  deployment tooling can compare desired launch-locator state against the
  current pointer without ad hoc shell checks.
- `afterburner deploy record-launch-locator-reconciliation-history` appends one
  compact `deployment_stack_launch_locator_reconciliation_history.json`
  artifact over desired-versus-current locator updates for downstream
  deployment audit tooling.
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
- Deployment-side launch planning can consume one resolved launch artifact
  instead of reconstructing environment and path expectations from recipes.
- Deployment-side rollout tooling can consume one packaged launch bundle rather
  than chasing separate launch plan and launch receipt artifacts.
- Downstream launch consumers can also resolve one smaller handoff artifact
  without depending on the full bundle layout.
- Transport-facing launch consumers can resolve one stable locator artifact
  without depending on the full handoff path convention.
- Pointer-oriented launch consumers can also resolve one stable current-pointer
  artifact without depending on the locator path convention itself.
- Future deployment adapters can consume one checked stack artifact instead of
  rediscovering runtime layout from recipes or host-local conventions.
