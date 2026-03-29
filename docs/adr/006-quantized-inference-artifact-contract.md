# ADR-006: Quantized Inference Artifact Contract

## Context

The inference artifact manifest previously described model identity, input shape,
and normalization, but it did not state whether the weights or activations were
reduced precision. That left quantized artifacts ambiguous: a future `int8`
artifact could look structurally valid even though the current runtime has no
quantized execution path.

## Decision

The inference manifest includes an explicit `[precision]` section with:

- `weights_dtype`
- `activation_dtype`
- `quantization`

The current supported contract is:

- `weights_dtype = "f32"`
- `activation_dtype = "f32"`
- `quantization = "none"`

Inference adapters must fail fast when the manifest declares any other
precision/quantization combination. For backward compatibility, manifests that
omit `[precision]` are interpreted as the current `f32`/`none` contract.

The repo also publishes `optimized_model_capability_surface.schema.json` to
make the current optimized-model runtime capability surface explicit:

- `weights_dtype = "f32"`
- `activation_dtype = "f32"`
- `quantization = "none"`

## Consequences

- Reduced-precision artifacts are explicit instead of implicit.
- CLI and HTTP adapters reject unsupported quantized artifacts before runtime
  execution begins.
- Future quantized runtime work can extend this contract deliberately instead of
  inferring precision from filenames or backend behavior.
