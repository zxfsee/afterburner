# ADR-012: CubeCL/CubeK Fit

## Context

The repo already carries an explicit custom-kernel adoption threshold in
`artifacts/train/kernel_adoption_thresholds.json` plus backend profiling
artifacts. That means the open question is not whether custom kernels are
allowed in principle, but which implementation path is the best fit if the
current backend-provided kernels stop meeting the checked threshold.

The current Burn stack already brings `cubecl` and `cubek` into `Cargo.lock`
transitively through the GPU backend path. Adding a second custom-kernel
framework or inventing a repo-local runtime before the existing threshold is
crossed would expand the maintenance surface without solving a verified problem.

## Decision

- `CubeCL`/`CubeK` are the first implementation path to evaluate if the checked
  kernel-adoption contract and profiling evidence ever justify direct custom
  kernel work.
- The current repo does not add a direct `cubecl` or `cubek` dependency, and it
  does not expose either library in any public contract.
- Until the threshold is intentionally refreshed with evidence, custom kernels
  remain deferred and backend-provided kernels remain the default execution path.
- If a future prototype is needed, keep it behind the existing Burn-aligned
  adapter/runtime boundary instead of introducing a parallel kernel subsystem.

Current stance: `CubeCL`/`CubeK` are a fit by stack alignment and existing
transitive presence, but not yet by measured need. No direct integration is
added in this change.

## Consequences

- Future kernel work has a preferred candidate path instead of an open-ended
  framework search.
- The repo keeps the option to stay aligned with the current Burn GPU stack
  rather than layering a second kernel runtime.
- Public contracts and architecture boundaries remain unchanged until measured
  evidence justifies a real implementation.
