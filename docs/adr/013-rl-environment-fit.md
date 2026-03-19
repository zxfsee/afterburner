# ADR-013: RL Environment Fit

## Context

The repo already carries a minimal RL artifact contract at
`fixtures/rl_rollout_metadata.schema.json`, but it does not yet define how much
environment/runtime machinery belongs inside Afterburner. Introducing simulator
bindings, vectorized environment orchestration, or step-level rollout storage
now would add runtime and contract surface before a concrete RL training loop
exists.

## Decision

- The smallest viable near-term shape is a single-environment, externally
  managed rollout producer that emits `rl_rollout_metadata.schema.json`
  compatible artifacts.
- The `environment` field stays a stable environment identity string, not a
  transport or simulator configuration payload.
- Vectorized environment orchestration is deferred until there is a measured
  throughput or sample-quality need that cannot be met by the single-environment
  shape.
- Simulator bindings and rollout-step payload contracts stay out of the repo for
  now; Afterburner should first consume or validate minimal rollout metadata
  artifacts rather than own the runtime loop.

Current stance: keep the RL contract artifact-first and runtime-light. One
single-environment metadata record is enough to preserve option space without
locking the repo into a simulator stack prematurely.

## Consequences

- Future RL work starts from a minimal artifact boundary instead of a baked-in
  simulator/runtime choice.
- The existing rollout metadata schema remains useful without forcing rollout
  storage or vectorized collection semantics too early.
- If RL training later needs in-repo environment orchestration, that decision
  must come with measured justification and a separate contract update.
