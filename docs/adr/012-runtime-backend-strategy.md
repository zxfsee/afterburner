# ADR-012: Training And Inference Runtime Backend Strategy

## Problem

Afterburner needs one coherent runtime-backend strategy for local and near-term
training and inference work. The repo already supports CPU and `wgpu`, now
exposes `metal`, and may later need a backend-extension path if measured GPU
workloads outgrow backend-provided kernels.

The architectural problem is not "which library looks interesting" in
isolation. It is how to keep backend selection, backend evolution, and future
custom-kernel work aligned with one runtime boundary instead of drifting into
multiple competing stacks.

## Forces / Constraints

- The current runtime boundary is Burn-aligned.
- The current public runtime surface is explicit backend selection through
  `BACKEND=cpu|wgpu|metal`.
- The compiler feature matrix and framework adapter registry must stay explicit.
- Backend evolution should remain measured through the checked backend profile
  and kernel-adoption evidence, not speculative framework churn.
- The repo should avoid widening into a second GPU/runtime stack without a
  verified workload need.
- Any future backend-extension path should preserve the existing core/adapter
  boundaries and public artifact contracts.

## Decision

Use one Burn-aligned runtime-backend strategy for local and near-term training
and inference.

- Keep the current runtime surface centered on CPU plus the Burn GPU path.
- Expose Metal explicitly through the existing Burn stack:
  - enable Burn's `metal` feature,
  - support `BACKEND=metal`,
  - keep the compiler feature matrix and framework adapter registry aligned with
    that explicit backend surface.
- Keep `wgpu` as the default GPU backend for now.
- Treat `CubeCL`/`CubeK` as the first backend-extension path to evaluate if
  checked kernel-adoption evidence justifies custom-kernel work.
- Treat `cutile-rs` as a later NVIDIA/CUDA-oriented candidate only if measured
  backend-extension need is specifically CUDA-shaped and the Burn/CubeCL path is
  insufficient.
- Do not introduce a second top-level runtime architecture or expose any of
  these backend-extension libraries as a public contract.

## Consequences

- Backend exposure stays explicit without fragmenting the runtime boundary.
- Apple Silicon has a real Metal-backed path inside the current Burn-aligned
  stack.
- Future backend-extension work has an ordered decision path instead of an
  open-ended framework search.
- The repo preserves option value for NVIDIA-specific optimization later without
  making it the default assumption now.

## Alternatives Considered

### Keep CPU plus `wgpu` only

Rejected because Burn already exposes `burn::backend::wgpu::Metal`, and
Afterburner benefits from making the Apple Silicon path explicit rather than
forcing it to stay implicit.

### Treat `cutile-rs` as the default next backend-extension path

Rejected because it is a narrower NVIDIA/CUDA-oriented choice and would widen
the stack before the Burn/CubeCL path has been shown insufficient.

### Introduce a separate repo-local GPU/backend subsystem

Rejected because it would break the current Burn-aligned runtime direction and
increase maintenance surface without a proved workload need.

## Current Implementation Mapping

- Current runtime surface: Burn CPU / `wgpu` / `metal`
- Metal path: `burn::backend::wgpu::Metal`
- First backend-extension candidate: `CubeCL` / `CubeK`
- Later conditional candidate: `cutile-rs`

## Revisit Triggers

- Backend profile evidence shows `wgpu` is no longer the right default.
- Kernel-adoption thresholds are intentionally refreshed with measured need.
- NVIDIA/CUDA-specific workloads justify evaluating a narrower extension path.
- Burn's backend surface changes enough that the current CPU / `wgpu` / `metal`
  split is no longer the right public runtime surface.
