# ADR-033: Burn Dependency Refresh

## Context

The repo pins `burn` and `burn-autodiff` at `0.20.1`. The active question is
whether that pin is already stale enough to justify a refresh now, or whether
the current line is still the right stable baseline while the next pre-release
line settles.

The relevant upstream state today is:

- current Afterburner pin: `0.20.1`
- latest stable Burn line observed via `cargo info burn`: `0.20.1`
- newer upstream pre-release observed via `cargo info burn`: `0.21.0-pre.1`

## Decision

Do not refresh Burn yet.

Keep the repo pinned to Burn `0.20.1` / `burn-autodiff 0.20.1` because that is
still the latest stable release line, while the visible newer line is
`0.21.0-pre.1` and would reopen recorder/storage contract drift before the repo
has chosen the corresponding migration work.

That storage drift is concrete: this repo still treats `.mpk` as an explicit
artifact contract, while upstream Burn is moving toward `.bpk`.

Current stance: remain on the current stable pin, and treat the `.mpk` to
`.bpk` cutover as a separate migration decision instead of smuggling it in under
a dependency bump.

## Consequences

- The repo stays current on the latest stable Burn line without adopting a
  pre-release just for freshness.
- Dependency refresh and artifact-format migration remain decoupled decisions.
- The next Burn refresh should be reconsidered when a newer stable line exists,
  not merely because a pre-release is available.
