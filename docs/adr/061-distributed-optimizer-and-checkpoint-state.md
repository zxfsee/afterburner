# ADR-061: Distributed Optimizer And Checkpoint State

## Context

Afterburner already separates scheduler lifecycle policy from Burn-side runtime
state. Future multi-device training recovery still needs one explicit in-job
contract for optimizer-state and checkpoint-group semantics so shard/rank
restores remain coherent without leaking cluster scheduling into checkpoint
meaning.

## Decision

Define the minimum distributed optimizer-state and checkpoint-group contract as
an in-job runtime concern.

- Optimizer-state recovery belongs to the Burn-side distributed runtime, not
  the scheduler.
- Recovery state must remain anchored to an explicit `checkpoint_group`
  identity.
- Future multi-device recovery should keep shard/rank optimizer state grouped
  under that `checkpoint_group` instead of inferring restore boundaries from
  host-local directory layout.
- Scheduler lifecycle remains outside the checkpoint meaning; it may request
  stop/resume, but it does not define optimizer-state semantics.

## Consequences

- Distributed recovery planning now has one clear boundary between runtime
  state and scheduler lifecycle.
- Future checkpoint/index work can anchor optimizer-state restore semantics to
  one explicit group identifier.
- The repo avoids coupling cluster placement policy to model/optimizer restore
  rules.
