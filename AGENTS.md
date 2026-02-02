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
- Tooling: prefer `just --list` to discover workflows and keep recipes up to date; the changelog is generated, so treat `CHANGELOG.md` as derived output.
