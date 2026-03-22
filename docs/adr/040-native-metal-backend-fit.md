# ADR-040: Native Metal Backend Fit

## Context

The repo currently defaults to `wgpu` for GPU work on Apple hardware. Burn
0.20.1 already exposes a Metal-backed runtime through the `metal` feature and
`burn::backend::wgpu::Metal`, so a Metal path exists today inside the existing
Burn GPU stack.

The open question is how Afterburner should surface that path without inventing
another repo-local backend family or changing artifact precision semantics.

## Decision

Expose Metal explicitly as a supported backend under the existing Burn GPU stack.

- Enable Burn's `metal` feature in the repo.
- Support `BACKEND=metal` for train, infer, and `afterburner-http`.
- Extend the compiler feature matrix and framework adapter registry to include
  `metal`.
- Keep `wgpu` as the default backend for now.
- Treat `metal` as the explicit Apple Silicon candidate path that can later
  become preferred if the checked backend-performance profile justifies it.

Current stance: this is a Burn-aligned backend exposure change, not a new
top-level runtime architecture. The repo is not introducing a second GPU stack
or changing the current artifact precision contract in this ADR.

## Consequences

- Apple Silicon now has an explicit Metal-backed runtime option without a
  separate repo-local GPU abstraction.
- Backend selection stays aligned with Burn terminology and the existing adapter
  registry and compiler feature matrix contracts.
- The default remains conservative until backend-profile evidence justifies a
  broader cutover.
