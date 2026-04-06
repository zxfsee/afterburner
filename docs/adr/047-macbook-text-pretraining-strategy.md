# ADR-047: MacBook Text Pretraining Strategy

## Problem

Afterburner needs a real local text-pretraining path that is more
representative than MNIST but still feasible on one MacBook. Without one
explicit decision, text-pretraining work risks splitting into two unrelated
threads:

- a workload envelope that is too vague to guide implementation, and
- an adapter path that is too concrete to stay aligned with that workload
  envelope.

The architecture needs one problem-centered decision that defines both:

- the bounded MacBook-local text-pretraining target, and
- the first adapter path that exercises that target without replacing the
  existing MNIST smoke path.

## Forces / Constraints

- The path must stay feasible on one local MacBook.
- The repo already has separate tokenizer, source, and text-inference contract
  work; this decision should align with them rather than replace them.
- The current training surface should keep MNIST as the cheapest smoke path.
- Text-pretraining must stay local-first and bounded, not drift toward
  frontier-scale assumptions.
- The first adapter cut should exercise checkpoint/resume, token cache, and
  text artifact placement together.

## Decision

Use one bounded MacBook text-pretraining strategy with an explicit local
adapter path.

### Workload envelope

The bounded MacBook-local text-pretraining target is:

- model family: decoder-only language model
- execution shape: single-node, single-device execution
- parameter scale: roughly `50M` to `300M` parameters
- sequence length: `1024` token context length
- token budget: `50M` to `200M` token budgets per training run
- optimizer family: `AdamW`
- checkpoint cadence: at least every `5` minutes or every epoch, whichever comes
  first

Acceptance stays explicit and lightweight:

- a deterministic tiny-overfit or short-loss-descent gate for text training
- successful checkpoint/resume inside the bounded run
- runtime and artifact contracts that still fit a single MacBook workflow

### Adapter path

Add one explicit text-training entrypoint:

- `afterburner train --task text`

The first adapter cut:

- consumes `pretraining_dataset_manifest` plus `text_tokenizer_packing_profile`
- consumes a deterministic local token cache through
  `text_token_cache.schema.json`
- trains a small causal language-model path on one device
- uses Burn learner checkpointing with epoch-based checkpoint/resume
- writes `text_pretraining_run.json`
- writes `text_pretraining_eval_summary.json`
- writes a separate text inference sidecar under `artifacts/text_inference/<version>/`
- emits `text_train_done`

## Consequences

- Future text-pretraining work has one bounded target instead of drifting
  between toy and frontier-scale assumptions.
- The text-pretraining track has a real executable path instead of only fit and
  contract ADRs.
- FineWeb-Edu source selection, tokenizer/packing, token cache, checkpointing,
  and text artifact placement can refine one shared local-first target instead
  of inventing separate workload assumptions.
- MNIST remains the cheapest smoke workflow because text training stays behind
  an explicit `--task text` path.

## Alternatives Considered

### Keep text-pretraining as envelope-only planning

Rejected because the repo would continue to have a bounded target without any
real adapter that exercises the contracts together.

### Jump directly to larger or distributed text training

Rejected because it would outrun the repo’s current local-first, bounded
execution model.

### Replace MNIST with text training as the default smoke path

Rejected because MNIST remains the cheapest smoke workflow and text training is
more expensive by design.

## Current Implementation Mapping

- workload envelope: bounded MacBook-local text-pretraining
- current adapter entrypoint: `afterburner train --task text`
- source/data side: `fineweb-edu/slice` plus local token cache
- runtime outputs: `text_pretraining_run.json`, `text_pretraining_eval_summary.json`
- text inference sidecar: `artifacts/text_inference/<version>/`

## Revisit Triggers

- local text training no longer fits the bounded MacBook envelope
- tokenizer/source/cache contracts change enough to invalidate the current
  adapter shape
- a larger or distributed text-training path becomes justified by concrete
  workloads
