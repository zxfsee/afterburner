# AGENTS.md

## References
- [README](./README.md)
- [Architecture](./ARCHITECTURE.md)
- [ADRs](./docs/adr/)
- [Roadmap](./ROADMAP.md)

## Project-specific constraints
- Keep changes small and reviewable.
- When changing behavior (including externally visible behavior or architecture invariants), update the relevant docs in the references list.
- When a decision is recorded in an ADR, link it from `ARCHITECTURE.md`.
- ADRs live in `docs/adr/` and use sequential numeric filenames: `docs/adr/NNN-title.md` (e.g. `docs/adr/001-training-vs-inference.md`). One decision per ADR.
- Keep a decisions index at `ARCHITECTURE.md#decisions` and link new ADRs there.
