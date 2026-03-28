# ADR-064: LWS Cluster Workload Fit

## Context

Afterburner already has an explicit scheduler/runtime boundary, a kube-rs-compatible
cluster path, and a workload-driven scheduler stance. The open question is whether
Kubernetes `lws` should become part of the current cluster control-plane surface, or
whether it should remain only a later reference point.

`lws` is relevant because it offers a cluster workload-management model for
distributed jobs. But adopting it too early would risk turning the repo into a
subset-of-Kubernetes project before the current kube-rs-compatible scheduler model
has a concrete workload pressure that needs it.

## Decision

Do not adopt `lws` now.

- Keep the current cluster path `kube-rs`-compatible.
- Keep scheduler policy, lease ownership, and lifecycle semantics anchored to the
  existing Afterburner scheduler model.
- Treat `lws` only as a later workload-management reference or thin adapter target
  if concrete cluster-path pressure appears.
- Do not reshape the current scheduler boundary or control-plane contracts around
  `lws`.
- Do not turn the repo into a subset-of-Kubernetes project.

## Consequences

- The current scheduler/runtime boundary stays stable.
- The cluster path remains adapter-side instead of inheriting Kubernetes-specific
  workload semantics as core policy.
- Option space stays open for a later thin `lws` adapter if measured workload needs
  justify it.
- kube-rs remains the current cluster integration surface.
