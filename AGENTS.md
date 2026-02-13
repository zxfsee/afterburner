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
- Keep ADRs/architecture docs current-state by default; avoid superseded/historical layering unless explicitly requested.
- Tooling: prefer `just --list` to discover workflows and keep recipes up to date.
- Recipes/docs must use the canonical contract-resolution mechanism — no hardcoded artifact paths or legacy compatibility shortcuts that bypass active contract selection.
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
- TODOs must be tagged with one or more focus areas (see list below) to keep the queue aligned with project goals.
- Focus areas (research infra domains) for TODO tagging:
  - Numerics
  - Kernels
  - RL Infra
  - Pre-training
  - Post-training
  - Inference
  - Distributed Training
  - Frameworks
  - Compilers
  - Runtime Infra
  - Data Infra
  - Experimentation/Eval Infra
  - Serving/Deployment Infra
- When adding the next TODO, prefer an underrepresented focus area unless blocked by an explicit dependency. If blocked, state the dependency in the TODO.
- Maintain human-level review bar for agent output; reject functional-but-poorly-maintainable code.
- When introducing a new subsystem (e.g. UI, persistence layer, RPC/transport, orchestration):
  - Do not invent a framework or choose a technology implicitly.
  - First check whether a stack is already established in the repo.
  - If not, ask the user to confirm the desired stack before introducing one.
