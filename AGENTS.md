# AGENTS.md

Normative rules only.

Only include rules and constraints that apply to all agent sessions.
Do not include temporary heuristics that reflect a single session correction.

## References

- [README](./README.md)
- [Architecture](./ARCHITECTURE.md)
- [ADRs](./docs/adr/)
- [Changelog](./CHANGELOG.md)
- [Backlog](./docs/backlog.md)
- [Notes](./NOTE.md)

## Project-specific constraints

- When changing behavior, update the relevant docs in the references list (including external interfaces and architecture invariants).
- Core must not depend on adapters (training, HTTP, telemetry, etc.); adapters depend on core.
- Core must not perform external IO (filesystem/network/process). Time/IDs/randomness must be injected via ports.
- Prefer explicit state transitions over implicit hidden state. Avoid hidden global state.
- Public contracts (artifact/CLI/HTTP/event schema) must be explicit, versioned when needed, and protected by compatibility tests (golden fixtures).
- Cutover is the default: do not add fallback paths, compatibility shims, feature flags, or dual-read/dual-write unless explicitly requested.
- Public contracts: once declared stable, backward compatibility is required within the same major/version; breaking changes require an explicit version bump or new vN surface, and documentation (plus compatibility tests as applicable).
- Prefer standards at boundaries; keep exporters/SDKs (if any) in adapters only.
- When feasible, enforce core/adapters dependency rules with an automated gate (crate boundaries + lint/test/CI); do not rely on convention alone.
- Prefer invariants that can be mechanically enforced (types, tests, CI checks, lints) over conventions or comments.
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
  - For ephemeral parallel workspaces, create them under `/tmp/<repo>/workspaces/<name>/` (stable parent directory), not as top-level `/tmp/...` directories.
  - Tasks that modify the same public contract, architectural boundary, shared files, or the TODO queue must not execute in parallel. Serialize them or land as stacked diffs.
- Verification model:
  - Spec-first gate:
    - For non-trivial TODOs, define explicit acceptance criteria and deterministic verification commands in the TODO entry or checkpoint before modifying code.
    - Acceptance criteria must be:
      - observable (produce measurable output: test result, artifact, event, exit code),
      - deterministic (same input -> same result),
      - executable via explicit commands,
      - tied to the contract or invariant being modified.
  - Verification is the review:
    - Prefer fixture/schema/tests as the primary acceptance mechanism.
    - Narrative or stylistic review is secondary to pass/fail evidence.
  - Verifier separation:
    - When using subagents, implementation and verification must be separate roles.
    - The verifier must validate the same revision/workspace snapshot.
    - The verifier must not modify code.
    - The verifier reports only: pass/fail and concrete mismatches.
- `CHANGELOG.md` is generated; treat it as derived output (edit `Cargo.toml` `[package.metadata.git-cliff.*]`, then regenerate).
- “Evolving organism” loop:
  - Each completed TODO must add at least one of:
    - a new gate (test/CI check), or
    - a new inspectable artifact/event (structured output under `artifacts/`).
  - Any new gate must fail before implementation and pass after (prove via tests/CI output).
  - After completing a TODO, append the next smallest unlocked TODO to the template/source TODO list.
  - Queue balance rule:
    - In any top-3 TODO window, at least one TODO must be `Kind: behavior` or `Kind: mixed`.
    - Do not queue more than two consecutive `Kind: gate` TODOs unless explicitly requested by the user.
    - Any `Kind: behavior` TODO must introduce or strengthen at least one gate that fails before the change and passes after.
  - TODO Kind classification (required):
    - Every TODO must declare `Kind: gate | behavior | mixed`.
    - `Kind: gate` = verification-only (tests/fixtures/contracts/CI).
    - `Kind: behavior` = runtime or training behavior change.
    - `Kind: mixed` = behavior change plus gate in the same TODO.
  - TODO decomposition discipline:
    - Avoid splitting TODOs into separate implementation and gate tasks when both can be completed within the same scope and contracts.
    - Split only when the follow-on gate or contract work is independently valuable, parallelizable, or blocked by an external decision.
  - Queue priority discipline:
    - Keep the TODO queue strictly priority-ranked.
    - Maintain a stable visible horizon (default: 6 items).
    - Park parked/blocked long-horizon items in `docs/backlog.md` (not in the active TODO queue).
    - Backlog items must keep full TODO metadata and use `Blocked-by:` where applicable.
    - Promote backlog items into the active TODO queue only when they become priority-relevant and unblocked.
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
- TODO title conventions:
  - If `Kind: gate`, include “gate/CI/fixture” in the title.
  - If `Kind: behavior` or `Kind: mixed`, do not label the title as “gate”.
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

### Governance edits

- Changes to `AGENTS.md`, TODO policy rules, or governance constraints require explicit user approval.
- Agents may propose governance changes but must not apply them automatically.
