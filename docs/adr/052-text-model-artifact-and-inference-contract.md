# ADR-052: Text Model Artifact And Inference Contract

## Context

The repo now has a bounded text-pretraining target, a FineWeb-Edu source fit,
and an explicit tokenizer/packing profile. What is still missing is one
artifact/inference contract for text-trained models so they do not get forced
through the current MNIST-oriented infer/eval surface.

The current inference contract assumes image-shaped inputs and logits output for
classification. A text-trained decoder-only model needs a different task
surface: prompt-based sampling with explicit tokenizer and sampling defaults.

## Decision

Future text-trained artifacts should carry a separate inference sidecar:

- `text_inference_profile.schema.json`

The minimum text inference profile records:

- `task = \`causal-lm\``
- `tokenizer_profile`
- `max_context_tokens`
- `sampling_defaults`

The future CLI surface should also be explicit and prompt-oriented, for example:

`afterburner sample --artifact PATH --prompt TEXT --max-new-tokens N`

and the future event surface should be explicit as well:

- `text_sample_done`

with at least:

- `artifact_version`
- `task`
- `prompt_token_count`
- `generated_token_count`
- `sampling`

Current stance: this ADR defines the future contract boundary only. It does not
add a text model runtime implementation or a new sampling command yet.

## Consequences

- Text-trained artifacts get their own explicit sampling contract instead of
  overloading the current image/logits inference surface.
- The tokenizer profile and sampling defaults become inspectable artifact data.
- Later text training and eval work can target one explicit artifact/event
  contract instead of inventing local prompt conventions.
