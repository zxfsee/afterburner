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
- In a `jj` workspace path, run `direnv allow` once, then run toolchain commands directly; use `direnv exec <workspace> <command>` only if direct execution proves the environment is missing required tools.
- On macOS, use the canonical physical path (for example via `pwd -P`) for temp workspaces when running `direnv` or tooling that tracks approval/state by path. Mixing symlink paths (for example `/tmp` vs `/private/tmp`) can cause duplicate approval states.
- If `jj`/`git` fails with sandbox-denied writes under `.git/*`, stop and request approval to rerun that command with `.git` write permission (permissions are per-command).
- If a subagent stops reporting after repeated waits, interrupt once, then inspect the assigned workspace state directly and recover from recorded commits/diffs instead of waiting indefinitely.
- If `NOTE.md` changes during active work, keep it attached to that same change/commit stream rather than leaving it as an unrelated local edit.
- In the main repo workspace, assume the shell already inherits required tooling; run `just`/tooling directly first, and only use `direnv exec` for isolated workspace contexts that actually lack deps.
- Do not volunteer half-finished reminders (optional cleanup/follow-up) in normal status updates; report only completed state and required blockers unless the user asks for open items.
