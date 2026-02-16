# AGENTS.md

Normative rules only.

Only include rules and constraints that apply to all agent sessions.
Do not include temporary heuristics that reflect a single session correction.

## References

- [README](./README.md)
- [Architecture](./ARCHITECTURE.md)
- [ADRs](./docs/adr/)
- [Changelog](./CHANGELOG.md)
- [Notes](./NOTE.md)

## Project-specific constraints

- When changing behavior, update the relevant docs in the references list (including external interfaces and architecture invariants).
- Core must not depend on adapters (training, HTTP, telemetry, etc.); adapters depend on core.
- Core must not perform external IO (filesystem/network/process). Time/IDs/randomness must be injected via ports.
- Public contracts (artifact/CLI/HTTP/event schema) must be explicit, versioned when needed, and protected by compatibility tests (golden fixtures).
- Prefer standards at boundaries; keep exporters/SDKs (if any) in adapters only.
- When feasible, enforce core/adapters dependency rules with an automated gate (crate boundaries + lint/test/CI); do not rely on convention alone.
- ADRs live in `docs/adr/` and use sequential numeric filenames: `docs/adr/NNN-title.md` (e.g. `docs/adr/001-training-vs-inference.md`). One decision per ADR.
- Keep a decisions index at `ARCHITECTURE.md#decisions` and link new ADRs there.
- ADRs reflect the current architectural decision. Historical evolution lives in VCS history. Do not create “superseded” ADR chains unless explicitly requested.
- Tooling: prefer `just --list` to discover workflows and keep recipes up to date.
- Recipes/docs must use the canonical contract-resolution mechanism — no hardcoded artifact paths or legacy compatibility shortcuts that bypass active contract selection.
- Integration policy:
  - Prefer stacked diffs that land via the merge queue.
  - Each diff must be independently reviewable and independently verifiable (own checks/evidence).
  - The stack must not break trunk/CI; rebase/update as needed to stay current.
  - Parallelize only across independent scopes.
  - If tasks touch the same contract, shared files, or the TODO queue, serialize them or use stacked diffs.
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
- Maintain production-grade maintainability standards; reject code that is correct but structurally poor.
- When introducing a new subsystem (e.g. UI, persistence layer, RPC/transport, orchestration):
  - Do not invent a framework or choose a technology implicitly.
  - First check whether a stack is already established in the repo.
  - If not, ask the user to confirm the desired stack before introducing one.

### Architecture selection

- Follow the architecture already established in-repo and described in `ARCHITECTURE.md`.
- If work would change architecture boundaries, public contracts, persistence model, runtime model, or introduce a new subsystem/pattern:
  - state the trigger and blast radius,
	-	choose the smallest reversible option that satisfies current acceptance criteria,
	-	record trade-offs,
	-	ask the user only if the choice is irreversible, high-cost to change later, or there are multiple options with materially different outcomes.


### Patterns

- Do not introduce new “best practices” or design patterns by default.
- Prefer patterns already established in-repo; extend them consistently.
- Introduce a new pattern only when it solves a concrete problem with evidence (failures, complexity, performance, operability).
- If multiple viable patterns exist with meaningful trade-offs, state the trade-off and why the chosen option wins.
- Any pattern that materially affects architecture boundaries, public contracts, persistence, runtime model, or build/CI strategy requires an ADR.
- If ADR required, the trade-off rationale belongs in the ADR; otherwise in the checkpoint report.
- Avoid pattern-driven refactors unless required to satisfy a TODO or unblock future work.
