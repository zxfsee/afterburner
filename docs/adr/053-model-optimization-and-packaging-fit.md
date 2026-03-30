# ADR-053: Model Optimization And Packaging Fit

## Context

The repo already has an explicit runtime/backend path and an explicit deployment
artifact path, but it still lacks one statement that post-training optimization
is its own leverage axis. Without that boundary, later quantization,
compression, export, and packaging work could blur into runtime scaling or
deployment orchestration.

## Decision

Model optimization and packaging are a fit, but as a separate post-training
artifact pipeline.

Define one explicit planning artifact:

- `model_optimization_profile.schema.json`
- `optimized_model_package_contract.schema.json`

The minimum profile records:

- `input_artifact_version`
- `target_environment`
- `optimization_steps`
- `output_constraints`

The package contract records:

- `export_format`
- `packaging_inputs`
- `package_layout`
- runtime precision compatibility for the emitted optimized artifact

Current stance: this decision defines the fit and the planning profile only. It
does not add optimization code or change the current runtime precision support.
`optimized_model_capability_surface.schema.json` makes that split explicit:
quantization, compression, export, and packaging are planning-only steps today,
while reduced-precision and compressed optimized artifacts remain unsupported
until dedicated contracts land.

## Consequences

- Quantization/compression/export/package work can evolve as a dedicated
  artifact pipeline instead of leaking into runtime-scale decisions.
- Later optimization capability, packaging, and eval gates can target one
  explicit profile shape.
- The repo can support both scale-up and optimize-down paths without confusing
  them.
