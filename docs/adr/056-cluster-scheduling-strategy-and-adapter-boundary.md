# ADR-056: Cluster Scheduling Strategy And Adapter Boundary

## Problem

Afterburner already separates runtime execution semantics from scheduler
control-plane policy. The remaining cluster-path question is how placement,
colocation, and Kubernetes-facing integration should evolve without turning the
repo into either:

- a generic cluster scheduler product, or
- a Kubernetes-specific workload subsystem.

The architecture needs one explicit decision for:

- when GPU colocation is even allowed,
- what the current cluster integration surface is,
- and how later cluster-workload adapters such as `lws` should be treated.

## Forces / Constraints

- Scheduler policy must stay separate from runtime execution semantics.
- Colocation is not the same thing as preemption.
- Cluster placement must reuse the same lease, admission, and lifecycle model as
  the single-node scheduler path.
- Kubernetes integration should stay adapter-side, not become the source of
  core scheduler semantics.
- The repo is workload-driven; it should not expand into a subset-of-Kubernetes
  control-plane project without concrete pressure.

## Decision

Use one constrained cluster-scheduling strategy with explicit adapter
boundaries.

### Colocation policy

- GPU colocation is only a constrained scheduling mode.
- It is only a fit for explicitly approved low-saturation workloads.
- It must be opt-in and workload-scoped, not the default scheduler policy.
- It must be evaluated against explicit interference limits:
  - latency regression
  - throughput regression
  - memory pressure
- It is not a substitute for checkpoint-based preemption.

### Current cluster integration path

- Treat `kube-rs` as the current cluster-path integration surface for the
  existing scheduler model.
- Keep the scheduler contract responsible for:
  - node inventory
  - queue admission
  - topology-aware GPU placement
  - lease ownership
  - job lifecycle reconciliation
- Reuse the same lease and lifecycle boundary model as the single-node
  scheduler.
- Keep `kube-rs` as an adapter layer, not a separate scheduler design.

### Later cluster workload adapters

- Do not adopt `lws` now.
- Keep the current cluster path `kube-rs`-compatible.
- Treat `lws` only as a later workload-management reference or thin adapter target
  if concrete cluster-path pressure appears.
- Do not reshape current scheduler boundaries or contracts around `lws`.
- Do not turn the repo into a subset-of-Kubernetes project.

## Consequences

- Colocation remains a narrow optimization path rather than a default policy.
- The cluster path stays aligned with the existing scheduler/runtime boundary.
- Placement and reconciliation logic can evolve without coupling training-state
  semantics into Kubernetes objects.
- Option space stays open for later thin cluster adapters without making them
  core architecture assumptions today.

## Alternatives Considered

### Default-on GPU sharing

Rejected because it would treat all workloads as shareable and blur the
constraint boundary between queueing, interference control, and execution.

### Kubernetes as the scheduler model

Rejected because it would replace the repo’s explicit scheduler contract with
cluster-product semantics and fragment the single-node and cluster paths.

### Early `lws` adoption

Rejected because the current `kube-rs`-compatible scheduler path has not yet
hit workload pressure that justifies another cluster workload layer.

## Current Implementation Mapping

- current colocation stance: opt-in, low-saturation only
- current cluster adapter: `kube-rs`
- current lease/lifecycle model: shared with the single-node scheduler path
- later candidate adapter: `lws`, only if concrete pressure appears

## Revisit Triggers

- measured workloads justify opt-in GPU colocation in a real scheduler path
- cluster placement needs more than the current kube-rs-compatible lease model
- cluster workload pressure makes a thin `lws` adapter materially useful
