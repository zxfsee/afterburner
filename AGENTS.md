# AGENTS.md

## References
- [README](./README.md)
- [Architecture](./ARCHITECTURE.md)
- [ADRs](./docs/adr/)
- [Changelog](./CHANGELOG.md)
- [Notes](./NOTE.md)

## Project-specific constraints
- When changing behavior, update the relevant docs in the references list (including external interfaces and architecture invariants).
- Core must not depend on adapters (training, HTTP, telemetry, etc.); adapters depend on core.
- Public contracts (artifact/CLI/HTTP/event schema) must be explicit, versioned when needed, and protected by compatibility tests (golden fixtures).
- Prefer standards at boundaries; keep exporters/SDKs (if any) in adapters only.
- ADRs live in `docs/adr/` and use sequential numeric filenames: `docs/adr/NNN-title.md` (e.g. `docs/adr/001-training-vs-inference.md`). One decision per ADR.
- Keep a decisions index at `ARCHITECTURE.md#decisions` and link new ADRs there.
- Tooling: prefer `just --list` to discover workflows and keep recipes up to date.
- Integration policy:
  - Prefer stacked diffs that land via the merge queue.
  - Each diff must be independently reviewable and independently verifiable (own checks/evidence).
  - The stack must not break trunk/CI; rebase/update as needed to stay current.
- `CHANGELOG.md` is generated; treat it as derived output (edit `Cargo.toml` `[package.metadata.git-cliff.*]`, then regenerate).
- “Evolving organism” loop:
  - Each completed TODO must add at least one of:
    - a new gate (test/CI check), or
    - a new inspectable artifact/event (structured output under `artifacts/`).
  - Any new gate must fail before implementation and pass after (prove via tests/CI output).
  - After completing a TODO, append the next smallest unlocked TODO to the template/source TODO list.
- Maintain human-level review bar for agent output; reject functional-but-poorly-maintainable code.
- When introducing a new subsystem (e.g. UI, persistence layer, RPC/transport, orchestration),
  do not invent a framework or choose a technology implicitly.
  First:
    - check whether a stack is already established in the repo,
    - if not, ask the user to confirm the desired stack before introducing one.
