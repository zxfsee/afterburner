# ADR-065: Gateway API Inference Extension Fit

## Context

Afterburner already has an explicit deployment stack contract, a deploy-rs
baseline, and a kube-rs-compatible cluster path. The open question is whether
the Kubernetes Gateway API inference extension should become part of the current
cluster-serving boundary, or whether it should remain only a later reference
point.

The Gateway API inference extension matters because it could provide a
traffic-management integration surface for cluster serving. But adopting it too
early would risk distorting the current runtime/scheduler design around ingress
and routing concerns before the repo has a concrete cluster-serving workload
that needs it.

## Decision

Do not adopt the Gateway API inference extension now.

- Keep current deployment and cluster-serving work anchored on the existing
  deploy-rs baseline, deployment stack contract, and kube-rs-compatible cluster
  path.
- Treat the Gateway API inference extension only as a later traffic-management reference or thin adapter target if concrete cluster-serving pressure appears.
- Keep any later Gateway API integration adapter-side.
- Do not reshape the current runtime, scheduler, or deployment stack contracts
  around ingress or traffic-management semantics.

## Consequences

- The current runtime/scheduler boundary stays focused on execution and
  lifecycle rather than ingress policy.
- The deployment stack contract stays explicit without inheriting Gateway API
  semantics prematurely.
- Option space stays open for a later thin Gateway API adapter if measured
  serving pressure justifies it.
