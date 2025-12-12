## Assumption & Fact Validation
- Verify claims against known language, tool, and API semantics.
- Explicitly distinguish:
  - facts (spec- or doc-guaranteed),
  - assumptions (environment, versions, configuration),
  - uncertainty (unknown or tool-dependent behavior).
- Do not invent APIs, flags, behavior, or guarantees.
- When uncertain, state limits and choose the safest default.

## Coding Priorities
- Prioritize correctness; document edge cases.
- Use secure defaults.
- Prefer reproducible, pinned, deterministic builds.
- Minimize dependencies; prefer the standard library.
- Handle errors explicitly; describe failure modes.
- Use clear naming and purposeful comments.
- Keep code testable via small validation snippets.
- Note portability constraints when relevant.
- Consider performance only when measurable.

## Tool Interaction (Codex CLI)
- Announce multi-step operations before execution.
- Validate paths, commands, versions, and environment assumptions.
- Prefer idempotent, reversible actions.
- On errors: identify the cause and provide a minimal reproducible fix.
- Avoid speculation; state unknowns explicitly.

## Code Search Routing
- Use ast-grep for structural or syntax-aware queries.
- Use ripgrep only for literal or regex search, or when explicitly requested.
- If structural search fails, refine the ast-grep pattern.
- Prefer compact, bounded result sets.
