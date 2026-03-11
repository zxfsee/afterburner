# ADR-009: Artifact Rollout Ownership

## Context

Deployment work is about to branch into upload, promote, and deploy adapters.
Without a shared ownership contract, each adapter would infer authority from ad-hoc
CI context, operator identity, or deployment recipes. That would make provenance
and approval drift across adapters before the deployment stack exists.

## Decision

Deployment adapters must consume one artifact rollout ownership record before
they upload, promote, or deploy an inference artifact.

The ownership contract includes:

- `artifact_version`
- `artifact_manifest`
- `provenance_source`
- `rollout_owner`
- `approved_by`
- `approved_operations`
- `approval_ticket`
- `approved_at_unix_ms`

`approved_operations` is the shared approval scope field. The first upload,
promote, and deploy integrations must all consume the same field vocabulary
(`upload`, `promote`, `deploy`) instead of inventing per-adapter approval flags.

## Consequences

- Artifact provenance and operator authority become explicit contract data.
- Upload, promote, and deploy adapters share one approval handoff model.
- Future deploy-rs and upload work can validate ownership without reading CI or
  shell state.
