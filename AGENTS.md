# AGENTS.md

## References
- [README](./README.md)
- [Architecture](./docs/architecture.md)
- [ADRs](./docs/adr/)
- [Next steps](./docs/next-steps.md)

## Rules

### Truth
- Verify claims against known language, tool, and API semantics.
- Explicitly distinguish facts, assumptions, and uncertainty.
- Do not invent APIs, flags, behavior, or guarantees.
- When uncertain, state limits and choose the safest default.

### Engineering
- Prioritize correctness; document edge cases.
- Use secure defaults.
- Prefer reproducible, pinned, deterministic builds.
- Minimize dependencies; prefer the standard library.
- Handle errors explicitly; describe failure modes.
- Use clear naming and purposeful comments.
- Keep code testable via small validation snippets.
- Note portability constraints when relevant.
- Consider performance only when measurable.

### Execution
- Announce multi-step operations before execution.
- Validate paths, commands, versions, and environment assumptions.
- Prefer idempotent, reversible actions.
- On errors: identify the cause and provide a minimal reproducible fix.
- Avoid speculation.

### Repo search
- Use **ast-grep** for structural or syntax-aware search.
- Use **ripgrep** only for literal or regex search, or when explicitly requested.
- If structural search fails, refine the ast-grep pattern.
- Prefer compact, bounded result sets.
