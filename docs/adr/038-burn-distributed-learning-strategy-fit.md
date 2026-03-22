# ADR-038: Burn Distributed Learning Strategy Fit

## Context

The current training-side scalability contract exposes `worker_parallelism` as a
local throughput knob. That is sufficient for the repo's current single-process
trainer, but it is too weak to describe future Burn-side distributed execution.

Burn's training APIs already model distributed learning around strategy and
multi-device concepts rather than a single worker-count scalar. In particular,
Burn exposes strategy-oriented surfaces such as `TrainingStrategy`,
`SupervisedTraining::with_training_strategy`, `MultiDevicesTrainStep`, and
`optimize_multi`, which line up with explicit distributed execution semantics
better than a local `worker_parallelism` count.

## Decision

Afterburner should align conceptually with Burn's distributed learning-strategy
model for future in-job distributed execution, but it should not overload the
current `worker_parallelism` contract to mean that strategy today.

- Keep `worker_parallelism` as a local throughput knob for the current trainer.
- Do not treat `training_scalability_contract.json` as a distributed strategy or
  topology contract.
- Future Burn-side distributed runtime work should use a strategy-oriented
  surface with explicit topology and device-group metadata instead of extending
  `worker_parallelism` ad hoc.
- The follow-on topology gate should define world-size, rank, and device-group
  fields before any repo contract claims data-parallel or multi-device training
  semantics.

Current stance: this ADR records fit and direction only. It does not add a
distributed runtime implementation or change the current training loop.

## Consequences

- The repo keeps the current trainer and training scalability contract honest:
  they describe local throughput, not distributed semantics.
- Future distributed runtime work can stay aligned with Burn terminology and API
  shape instead of inventing a parallel repo-local vocabulary.
- The next topology gate becomes the required contract step before any
  distributed strategy claims are surfaced in artifacts or events.
