# ADR-010: Deployment Baseline And Local Orchestration Substrate

## Problem

Afterburner already has an explicit deployment target profile contract, but it
still needs concrete substrate choices for two different environments:

- a real deployment baseline that consumes the target profile contract, and
- a local multi-process substrate for MacBook-local orchestration work.

Without one explicit decision, deployment wiring stays implicit and local
orchestration risks drifting into ad hoc shell glue or collapsing platform
choices into scheduler semantics.

## Forces / Constraints

- This repo is not a NixOS host-configuration repository.
- The deployment baseline should treat Afterburner as an application profile,
  not as a full system rebuild.
- Local orchestration should stay separate from both the scheduler layer and the
  cluster path.
- The current Nix/flake setup should remain the source of truth for local
  substrate composition.
- `systemd` on hosts and Kubernetes on the cluster path remain distinct
  execution substrates.

## Decision

Use one explicit substrate strategy with:

- `deploy-rs` as the application-scoped deployment baseline
- `process-compose-flake` as the MacBook-local multi-process substrate

### Deployment baseline

- Keep `deploy-rs` as the explicit deployment adapter baseline in `flake.nix`.
- Consume `fixtures/deployment_target_profile.example.json` as executable input.
- Expose one deploy node named from the deployment target profile contract.
- Use `deploy-rs.lib.<system>.activate.custom ... "./bin/afterburner"` for the
  baseline application activation path.
- Keep this baseline application-scoped; do not introduce a NixOS deployment
  surface, rollout orchestration layer, or upload logic here.

### Local orchestration substrate

- Treat `process-compose-flake` as a fit for the MacBook-local substrate only.
- Use it only for local multi-process orchestration of the Afterburner runtime
  stack and scheduler prototype.
- Do not treat it as a replacement for `systemd` on NixOS hosts.
- Do not treat it as a replacement for Kubernetes on the cluster path.
- Do not treat it as the scheduler layer itself; it is only a local process
  substrate.

## Consequences

- The deployment target profile fixture becomes executable deployment input
  instead of schema-only documentation.
- The repo gets one validated deployment adapter baseline without pretending to
  be a host-configuration repo.
- Local orchestration can be explored in a reproducible Nix-aligned way without
  collapsing the platform boundary into the scheduler boundary.
- Future deploy, upload, and rollout work can extend one deployment substrate
  decision instead of inventing multiple parallel vocabularies.

## Alternatives Considered

### Leave deploy-rs as a future idea

Rejected because it would keep the deployment target profile contract
unconsumed and deployment wiring implicit.

### Treat deploy-rs as a full NixOS deployment surface

Rejected because this repo is application-scoped, not a full host-configuration
repo.

### Use process-compose-flake as a general deployment substrate

Rejected because it is only a local process substrate and should not replace
`systemd` or Kubernetes.

## Current Implementation Mapping

- deployment baseline: `deploy-rs`
- deployment activation path: `activate.custom`
- local multi-process substrate: `process-compose-flake`
- host substrate remains distinct: `systemd`
- cluster substrate remains distinct: Kubernetes

## Revisit Triggers

- deployment work needs more than the current application-scoped deploy-rs
  baseline
- local orchestration needs capabilities that the current Nix-aligned process
  substrate cannot provide
- host or cluster execution requirements force a different substrate boundary
