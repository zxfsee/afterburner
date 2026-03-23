# ADR-057: `cutile-rs` Backend Extension Fit

## Context

The repo already has two explicit decisions in this area:

- custom kernel work is gated by measured need and a checked threshold
- CubeCL/CubeK are the current first candidate path if that threshold is crossed

`cutile-rs` is interesting as a Rust-native NVIDIA-oriented kernel/backend
extension path, but it is also early-stage, CUDA-specific, Linux-oriented, and
does not line up with the current cross-platform Burn/CubeCL preference by
default.

## Decision

`cutile-rs` is not the default next backend-extension path.

- Keep CubeCL/CubeK as the first candidate path for custom-kernel work.
- Treat `cutile-rs` as a later fit to reconsider only if:
  - measured need is specifically NVIDIA/CUDA-oriented,
  - the required hardware and toolchain constraints are acceptable,
  - and CubeCL/CubeK are insufficient for the target workload.

Current stance: `cutile-rs` is a plausible later backend-extension candidate,
but not the preferred current path.

## Consequences

- The repo preserves its current Burn/CubeCL-aligned direction instead of
  widening into a second kernel stack prematurely.
- NVIDIA-specific backend-extension work remains possible later without
  becoming the default assumption today.
- Future `cutile-rs` work must justify why the existing CubeCL/CubeK path is
  insufficient.
