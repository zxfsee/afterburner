# NOTE.md

Reflective memory only.

Only record behavior patterns and heuristics that:
* are new,
* have been observed repeatedly,
* are not official policy.

Do *not* restate or reformulate existing rules from `AGENTS.md`.

## Heuristics

- If the user gives a process reminder during active changes (for example commit hygiene), treat it as a request to execute the missing step now, not as documentation work.
- If proposing additional work outside the active TODO queue, either add it to TODO immediately or do not mention it.
- If a requested skill appears unavailable from session discovery, verify `~/.config/codex/skills/<skill>/SKILL.md` before declaring it missing.
- In a `jj` workspace path, run `direnv allow` for that workspace before toolchain commands; for non-interactive execution, prefer `direnv exec <workspace> <command>` so flake-provided tools are used.
- If a subagent stops reporting after repeated waits, interrupt once, then inspect the assigned workspace state directly and recover from recorded commits/diffs instead of waiting indefinitely.
- If `NOTE.md` changes during active work, keep it attached to that same change/commit stream rather than leaving it as an unrelated local edit.
- In the main repo workspace, assume the shell already inherits required tooling; run `just`/tooling directly first, and only use `direnv exec` for isolated workspace contexts that actually lack deps.
- For changelog regeneration in this environment, prefer `git-cliff --offline` (or equivalent recipe behavior) to avoid remote metadata/rate-limit/cache-path failures.
