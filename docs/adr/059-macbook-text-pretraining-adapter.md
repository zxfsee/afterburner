# ADR-059: MacBook Text Pretraining Adapter

## Context

The repo already has:

- a bounded MacBook-local text-pretraining target
- a pinned FineWeb-Edu source fit
- an explicit tokenizer/packing profile
- an explicit text inference sidecar contract

What is still missing is one real training adapter that exercises those
contracts on a single device without replacing the existing MNIST smoke path.

## Decision

Add one explicit text-training entrypoint:

- `afterburner train --task text`

The adapter:

- consumes `pretraining_dataset_manifest` plus `text_tokenizer_packing_profile`
- consumes a deterministic local token cache through
  `text_token_cache.schema.json`
- trains a small causal language-model path on one device
- uses Burn learner checkpointing with epoch-based checkpoint/resume
- writes `text_pretraining_run.json`
- writes a separate text inference sidecar under `artifacts/text_inference/<version>/`
- emits `text_train_done`

Current stance: this is the first bounded adapter cut. It does not claim full
tokenizer implementation, large-model parity, or a prompt-sampling runtime yet.

## Consequences

- The text-pretraining track now has a real executable path instead of only fit
  and contract ADRs.
- MNIST remains the cheapest smoke workflow because text training stays behind
  an explicit `--task text` path.
- Deterministic local cache, checkpoint/resume, and text artifact placement are
  now exercised together instead of remaining separate paper contracts.
