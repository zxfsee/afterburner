# ADR-010: Deploy-rs Baseline

## Context

The deployment target profile contract now exists, but there is still no concrete
deployment adapter consuming it. Leaving deploy-rs as a future idea would keep
deployment wiring implicit and unvalidated.

This repository is not a NixOS host configuration. The first deployment baseline
therefore needs to treat Afterburner as an application profile, not as a full
system rebuild.

## Decision

Add a minimal deploy-rs baseline in `flake.nix` that:

- reads `fixtures/deployment_target_profile.example.json` as the source-of-truth
  example profile,
- exposes one deploy node named after `profile_name`,
- uses `hostname`, `sshUser`, and `profilePath` from the profile contract,
- activates the packaged Afterburner application with
  `deploy-rs.lib.<system>.activate.custom ... "./bin/afterburner"`,
- and validates the deployment definition through `deploy-rs.lib.<system>.deployChecks`.

The baseline is intentionally application-scoped. It does not introduce a NixOS
deployment surface, rollout orchestration, or upload logic yet.

## Consequences

- Deploy-rs is now an explicit adapter contract rather than a backlog idea.
- The deployment target profile fixture becomes executable input, not just schema
  documentation.
- Future upload/promote/deploy work can extend one validated deploy-rs surface
  instead of inventing a second deployment vocabulary.
