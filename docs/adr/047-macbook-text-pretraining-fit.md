# ADR-047: MacBook Text Pretraining Fit

## Context

The repo wants a real text-pretraining path that is more representative than
MNIST, but still feasible on one MacBook. Without an explicit workload envelope,
future text-pretraining work could either stay too toy-like to be useful or
balloon into a frontier-scale target that does not fit the local-first project
shape.

The tokenizer contract, source selection, and text artifact surface are tracked
as separate follow-on items, so this decision should only define the minimum
useful workload envelope.

## Decision

The smallest useful MacBook text-pretraining envelope is:

- model family: decoder-only language model
- execution shape: single node, single GPU/device
- parameter scale: roughly `50M` to `300M` parameters
- sequence length: `1024` tokens
- token budget: `50M` to `200M` tokens per training run
- optimizer family: `AdamW`
- checkpoint cadence: at least every `5` minutes or every epoch, whichever comes
  first

Acceptance for this envelope should be explicit and lightweight:

- a deterministic tiny-overfit or short-loss-descent gate for text training
- successful checkpoint/resume inside the bounded run
- runtime and artifact contracts that still fit a single MacBook workflow

Current stance: this ADR defines the workload envelope only. It does not choose
the tokenizer, dataset slice, or final text artifact contract yet.

## Consequences

- Future text-pretraining work now has a bounded target instead of drifting
  between toy and frontier-scale assumptions.
- FineWeb-Edu source selection, tokenizer/packing, and text artifact work can
  refine a shared workload envelope instead of inventing their own.
- MNIST can remain the cheapest smoke path while text training grows behind a
  separate, more realistic local-first envelope.
