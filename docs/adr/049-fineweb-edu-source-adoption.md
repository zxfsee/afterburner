# ADR-049: FineWeb-Edu Source Adoption

## Context

The repo now has a bounded MacBook-local text-pretraining target, but the
current example pretraining dataset manifest still points at a generic `c4/en`
source. That leaves the source side of the local-first text path underspecified
and does not reflect the intended FineWeb-Edu direction.

The source gate should stay narrow: define the approved source key, revision
shape, shard/checksum expectations, and a local size budget, without choosing
the tokenizer or final text artifact format yet.

## Decision

Adopt a bounded FineWeb-Edu slice as the current local-first text source fit.

- `source`: `fineweb-edu/slice`
- `source_revision`: a pinned bounded local slice snapshot identifier
- shard/checksum expectations remain explicit through
  `pretraining_dataset_manifest.schema.json`
- local size budget should stay small enough for one MacBook workflow:
  aim for tens of gigabytes at most, not full-corpus mirroring

Current stance: this ADR records source fit and naming only. It does not yet
add source approval receipts, tokenizer choices, or a concrete download tool.

## Consequences

- The repo's example pretraining dataset contract now points at the intended
  FineWeb-Edu direction instead of a generic legacy corpus.
- Later tokenizer and text-artifact work can target one bounded source family.
- The local-first text path stays explicit about using a slice, not the full
  upstream corpus.
