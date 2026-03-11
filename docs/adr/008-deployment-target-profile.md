# ADR-008: Deployment Target Profile

## Context

Future deployment work is blocked on an explicit model for where artifacts are
activated. Without a deployment target profile, deploy-rs or any other adapter
would have to rely on ad-hoc hostnames, users, and paths embedded in recipes or
shell scripts.

## Decision

Deployment adapters must resolve a named deployment target profile before they
attempt rollout.

The profile contract includes:

- `profile_name`
- `deploy_hostname`
- `ssh_user`
- `system`
- `artifact_root`
- `activation_strategy`

The first deploy-rs integration must consume this profile model rather than
inventing a separate host configuration surface.

## Consequences

- Deploy-rs work has a stable input contract before rollout code exists.
- Host-specific deployment data moves into an explicit schema instead of shell
  conventions.
- Future upload/promote/deploy work can share one target profile vocabulary.
