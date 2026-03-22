# ADR-046: Single-Node GPU Lease And Systemd Adapter

## Context

The repo now has explicit scheduler lifecycle contracts, but there is still no
concrete single-node scheduling path that admits a job onto a free GPU and
materializes the runtime-side launch surface. Without one adapter, single-node
scheduler work would remain conceptual and not exercise queue admission, lease
ownership, or graceful stop wiring at all.

## Decision

Add a minimal single-node scheduler adapter:

- `afterburner deploy single-node-scheduler`

The command:

- reads a scheduler job request contract,
- reads a single-node GPU inventory contract,
- admits a one-GPU job onto the first free GPU,
- writes a lease artifact,
- writes a systemd service unit file,
- and emits a matching scheduler event.

Current stance: this adapter is intentionally bounded to `gpu_count = 1` and a
single-node systemd-oriented launch surface. It does not add a queue daemon or
multi-node control plane.

## Consequences

- Single-node scheduler work now has one executable admission path instead of
  only abstract lifecycle messages.
- Lease ownership and graceful stop wiring become inspectable artifacts.
- Future systemd or process-compose substrate work can extend one explicit
  single-node scheduler surface instead of inventing a second local scheduler
  vocabulary.
