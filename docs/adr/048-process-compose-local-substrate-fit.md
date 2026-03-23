# ADR-048: process-compose-flake Local Substrate Fit

## Context

The repo already has:

- a single-node systemd-oriented scheduler adapter path,
- a separate Kubernetes-path scheduler backlog,
- and a flake-parts based Nix setup for local development.

For MacBook-local multi-process workflows, `process-compose-flake` fits the
existing Nix model well because it can declare reproducible process groups and
dependencies directly in `flake.nix` without inventing another local shell
orchestration layer.

## Decision

`process-compose-flake` is a fit for the MacBook-local substrate only.

- It is suitable for local multi-process orchestration of the Afterburner runtime
  stack and scheduler prototype.
- It is not a replacement for `systemd` on NixOS hosts.
- It is not a replacement for Kubernetes on the cluster path.
- It is not the scheduler layer itself; it is only a local process substrate.

Current stance: this ADR records fit and scope only. It does not add a
`process-compose-flake` configuration yet.

## Consequences

- The repo has a clear local-substrate candidate that matches the current
  flake-parts setup.
- Local orchestration can be explored without collapsing the platform boundary
  into the scheduler boundary.
- Future systemd and Kubernetes work remain valid and separate.
