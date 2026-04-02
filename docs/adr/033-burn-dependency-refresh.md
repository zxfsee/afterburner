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

Checked availability state for the active queue is explicit:

- checked latest stable Burn release: `0.20.1`
- newer stable Burn release beyond `0.20.1`: `no`
- result: keep the `.bpk` migration blocked until this checked state changes

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

The immediate runnable prerequisite is now explicit too:
`burn_bpk_migration_surface_inventory.json` records the current `.mpk`
touchpoints and cutover-sensitive surfaces so the later `.bpk` migration can
land as one tracked contract change when a newer stable Burn release exists.

## Consequences

- The repo stays current on the latest stable Burn line without adopting a
  pre-release just for freshness.
- Dependency refresh and artifact-format migration remain decoupled decisions.
- The next Burn refresh should be reconsidered when a newer stable line exists,
  not merely because a pre-release is available.
