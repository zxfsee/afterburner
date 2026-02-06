# AGENTS.md

## References
- [README](./README.md)
- [Architecture](./ARCHITECTURE.md)
- [ADRs](./docs/adr/)
- [Changelog](./CHANGELOG.md)

## Project-specific constraints
- Keep changes small and reviewable.
- When changing behavior, update the relevant docs in the references list (including external interfaces and architecture invariants).
- ADRs live in `docs/adr/` and use sequential numeric filenames: `docs/adr/NNN-title.md` (e.g. `docs/adr/001-training-vs-inference.md`). One decision per ADR.
- Keep a decisions index at `ARCHITECTURE.md#decisions` and link new ADRs there.
- Tooling: prefer `just --list` to discover workflows and keep recipes up to date.
- `CHANGELOG.md` is generated; treat it as derived output (edit the template/source, then regenerate).
- “Evolving organism” loop:
  - Each completed TODO must add at least one of:
    - a new gate (test/CI check), or
    - a new inspectable artifact/event (structured output under `artifacts/`).
  - After completing a TODO, append the next smallest unlocked TODO to the template/source TODO list.
