# ADR-050: Text Tokenizer And Packing Contract

## Context

The repo now has a bounded MacBook text-pretraining target and a FineWeb-Edu
slice source fit, but it still lacks one explicit tokenizer and packing
contract. Without that profile, later text batches could depend on ad hoc
tokenizer choices, implicit special tokens, or inconsistent sequence packing
rules.

## Decision

Define one explicit tokenizer and packing profile contract:

- `text_tokenizer_packing_profile.schema.json`

The minimum profile records:

- `tokenizer_id`
- `tokenizer_revision`
- `vocab_size`
- `bos_token`
- `eos_token`
- `pad_token`
- `context_length`
- `truncation_policy`
- `packing_policy`

Current stance: this ADR defines the tokenizer/packing profile only. It does
not add tokenization code or a text-trained artifact contract yet.

## Consequences

- Future text batch preparation can point at one explicit tokenizer/packing
  profile instead of local preprocessing conventions.
- Sequence length and packing rules stay aligned with the bounded MacBook text
  target.
- The later text artifact and training-adapter work can reuse one stable profile
  contract instead of inventing parallel tokenizer metadata.
