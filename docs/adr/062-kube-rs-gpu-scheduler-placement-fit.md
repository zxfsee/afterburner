# ADR-062: kube-rs GPU Scheduler Placement Fit

## Context

The repo already has an explicit scheduler/runtime lifecycle boundary and a
single-node GPU lease adapter. The remaining cluster-path question is how to
realize queue admission, node inventory, topology-aware GPU placement, and job
lifecycle reconciliation on Kubernetes without turning Kubernetes itself into a
second scheduler design.

## Decision

Treat `kube-rs` as the cluster-path integration surface for the existing
scheduler model.

- The scheduler contract stays responsible for:
  - node inventory
  - queue admission
  - topology-aware GPU placement
  - lease ownership
  - job lifecycle reconciliation
- The Kubernetes path should reuse the same lease and lifecycle boundary model
  as the single-node scheduler.
- Cluster-path scheduler consumers should resolve the active lease state through
  one stable pointer artifact over the kube-rs reconciliation surface instead of
  depending on local reconciliation artifact paths.
- `kube-rs` is the adapter layer for the cluster path; it is not a separate
  scheduler or a subset-of-Kubernetes product target.

## Consequences

- The cluster path stays aligned with the existing scheduler/runtime boundary
  instead of inventing a second orchestration vocabulary.
- Compact append-only history over kube-rs lease pointer changes can stay
  downstream and audit-focused as long as it remains derived from the explicit
  pointer artifact instead of controller or shell logs.
- Placement and reconciliation logic can evolve without coupling training-state
  semantics into Kubernetes objects.
- The repo keeps one scheduler model across local host and cluster substrates.
