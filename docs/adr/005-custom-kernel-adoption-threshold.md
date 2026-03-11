# ADR-005: Custom Kernel Adoption Threshold

## Context

The current model runs on backend-provided kernels and already has a checked
backend performance profile. Adding custom kernels would introduce a new
execution path, extra compatibility surface, and ongoing maintenance cost.
Without an explicit threshold, kernel work would be speculative and hard to
revert.

## Decision

Training writes `artifacts/train/kernel_adoption_thresholds.json` as the
decision contract for custom kernel work.

The contract records:

- the current convolution footprint (`1x1`, `3x3`, `5x5`) and per-site inventory
- the current input dtype (`f32`)
- compatibility constraints that must remain true for the contract to apply
  (`groups = 1`, no dilation, declared padding modes only)
- the supporting backend-profile evidence required before introducing custom kernels

Custom kernels remain deferred until the contract is intentionally refreshed with
current model coverage and sustained backend-profile evidence.

## Consequences

- Kernel optimization work stays evidence-driven instead of speculative.
- Model architecture changes that alter kernel coverage now require an explicit
  contract refresh.
- Future custom kernel proposals have a concrete compatibility and measurement
  baseline to satisfy before adding a new runtime path.
