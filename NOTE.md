# NOTE.md

Reflective memory only.

Only record behavior patterns and heuristics that:
* are new,
* have been observed repeatedly,
* are not official policy.

Do *not* restate or reformulate existing rules from `AGENTS.md`.

## Heuristics

- If the next phase will use `jj` repo metadata operations (`status`, `diff`, `log`, `commit`, workspace management) in a sandboxed repo, request `.git` read/write permission up front instead of waiting for the first lock failure.
- If the user gives a process reminder during active changes (for example commit hygiene), treat it as a request to execute the missing step now, not as documentation work.
- If proposing additional work outside the active TODO queue, either add it to TODO immediately or do not mention it.
- If the user asks to append or reprioritize TODO items, treat that as queue-only work; do not start implementing the queued item in the same pass unless they explicitly ask for execution too.
- Do not mark a TODO complete on a partial improvement. If the stated goal is only partly satisfied, keep the item open or immediately queue the exact remainder instead of silently advancing the horizon.
- If a requested skill appears unavailable from session discovery, verify `~/.config/codex/skills/<skill>/SKILL.md` before declaring it missing.
- In a `jj` workspace path, run `direnv allow` once, then run toolchain commands directly; use `direnv exec <workspace> <command>` only if direct execution proves the environment is missing required tools.
- On macOS, use the canonical physical path (for example via `pwd -P`) for temp workspaces when running `direnv` or tooling that tracks approval/state by path. Mixing symlink paths (for example `/tmp` vs `/private/tmp`) can cause duplicate approval states.
- If a subagent stops reporting after repeated waits, interrupt once, then inspect the assigned workspace state directly and recover from recorded commits/diffs instead of waiting indefinitely.
- If `NOTE.md` changes during active work, keep it attached to that same change/commit stream rather than leaving it as an unrelated local edit.
- In the main repo workspace, assume the shell already inherits required tooling; run `just`/tooling directly first, and only use `direnv exec` for isolated workspace contexts that actually lack deps.
- In repo workflows with a canonical `just` surface, dogfood the `just` recipe by default. Drop to narrower underlying tool commands only for genuine special cases where the recipe scope is wrong for the verification or fix being performed.
- Do not leave iterative work implicit. If a turn uncovers meaningful follow-up, unresolved cleanup, or context worth preserving, either complete it in the same turn or record the next step explicitly in the correct planning surface before stopping: top TODO queue when it is priority and runnable now, otherwise `docs/backlog.md`.
- Treat dependency and toolchain refresh work as preventative maintenance, not optional side work. If the user raises update cadence or asks for refreshes, follow through to the full repo verification path (`just check`) because delaying it increases drift in tests, fixtures, and the Nix/Cargo harness.
- Prefer `cargo nextest` (or `just test` when it maps to nextest) over plain `cargo test` for verification unless a specific test flow requires `cargo test` semantics.
- Keep strict active-queue order by default. After completing the top TODO, the next pre-existing queued item should become top unless there is a real blocker or the user explicitly approves reprioritization.
- If a same-theme follow-on mixed item is worth tracking, prefer adding it below the already-queued next item or parking it in backlog; do not silently insert it at the top just to satisfy queue-balance preferences.
- Before announcing or starting the next queue item, re-read the live `CHANGELOG.md` TODO block and use it as the source of truth rather than relying on prior mental state or earlier intermediate snapshots.
- Treat queue freshness as part of completing each queue item: after landing a TODO, refresh the active queue immediately in the same change stream, and before continuing confirm the top visible item is not already represented by a freshly landed commit. Keep the dependency explicit by running the canonical queue command and re-reading the resulting files instead of relying on hidden bookkeeping state.
- If a follow-up change only refreshes queue/changelog/bookkeeping state, give it a bookkeeping-specific commit message; do not reuse the feature commit subject/body from the implementation change.
- Shape commit subjects for fast visual distinction: keep only one primary dimension in the subject, push secondary detail into the body, and avoid adjacent same-family commits whose subjects differ only by one noun like `bundle` vs `receipt` or `history` vs `reconciliation`. Fold purely derived changelog refreshes into the functional commit unless they are independently meaningful.
- Keep commit bodies focused on the change itself: problem, rationale, constraints, and non-obvious effects. Do not spend body space on queue/changelog bookkeeping unless that bookkeeping is independently meaningful to the landed change.
- Do not volunteer half-finished reminders (optional cleanup/follow-up) in normal status updates; report only completed state and required blockers unless the user asks for open items.
- If a repo mutation is rejected as `unacceptable risk`, treat it as a fresh policy gate: ask again with an explicit approval prompt instead of relying on prior or plain-text approval.
- Do not add environment-preflight or host-availability gate recipes to `justfile`; keep `just` focused on the composable workflow entrypoints and let prerequisites fail naturally or live in tests/TODOs.
- Do not split one environment/tooling blocker into multiple active gate TODOs. If the issue is host-specific or transient, keep one contract/workflow gate at most, then record the remaining problem as an inline `Blocked-by:` note on the real TODO.
- Park profiling/flamegraph work unless a near-term runtime/kernel/quantization decision actually depends on measured hotspot evidence; otherwise it is easy to over-invest in tooling before the next leverage point is ready.
- When passing commit messages through `shell_command`, avoid unescaped backticks in the shell string; they trigger command substitution and silently corrupt the commit body.
- If consecutive TODOs are only refining adjacent contract/receipt/provenance layers with little new runtime or decision leverage, stop and call that out instead of continuing mechanically. Park or regroup low-leverage follow-ons before they turn into busywork.
- When a gate can be folded into a nearby mixed item without losing clarity, prefer folding it; if a gate only restates an already-decided boundary, park or remove it instead of adding another ADR.
- In constrained time, prioritize the highest-ROI item that still benefits from an explicit spec; keep enough architecture context to avoid drift, but do not expand documentation or ADR surface beyond what materially preserves the direction of travel.
- Prefer the lowest-energy change that preserves architectural direction, avoids drift, and unlocks the next real capability; do not spend complexity, process, or implementation effort unless it materially increases leverage.
- If public CLI names start mirroring internal artifact taxonomy, stop and collapse them behind `just` or `afterburner debug ...`; keep operator commands intent-first and proactive.
- When the repo shows repeated command paths or near-duplicate recipes for the same workflow, prefer consolidating toward one canonical execution surface instead of adding more process around the divergence.
- When artifact writing plus event emission repeats across multiple commands, treat that as a signal to extract a small shared helper layer before drift in fields, paths, or guarantees spreads further.
- Keep `just` as the canonical orchestration surface, not a policy engine. When reconciliation/history/protocol logic starts repeating across recipes, consolidate the repeated logic into a small typed helper instead of growing recipe-local protocol code or embedding more semantic decision logic in recipes.
- Treat this repo as a monorepo for workflow purposes: monorepo-local helper binaries, Nushell scripts, and Nix-integrated harness code are acceptable implementation tools as long as they stay behind the canonical `just` entry surface instead of creating parallel user-facing workflows.
- In this repo, use a short commit body by default when a change touches public contracts, queue/changelog state, or architecture-adjacent behavior; subject-only commits are often too thin for later review.
- When a user correction changes the real success criterion for a doc or queue item, stop queue advancement, move any stale test/assertion ownership to the correct surface, and revalidate against the corrected document purpose before continuing.
- If the user states a queue prioritization rule, treat it as a live ordering constraint: reorder active and backlog items by ROI instead of preserving thematic or historical sequencing.
