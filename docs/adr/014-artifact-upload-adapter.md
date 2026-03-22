# ADR-014: Artifact Upload Adapter Contract

## Context

The repo already defines versioned inference artifacts plus rollout ownership
approval data, but there is still no concrete upload adapter between those
contracts and a future provider implementation. Adding a vendor SDK now would
collapse the contract decision into one provider choice before the upload shape
is validated.

## Decision

Add an `afterburner deploy upload` CLI adapter that:

- reads an explicit artifact manifest path,
- validates that the referenced artifact file matches the checked manifest,
- requires an explicit artifact rollout ownership record with `upload` approval,
- accepts provider and destination as provider-neutral strings,
- and writes a structured `artifact_upload_request.json` artifact instead of
  making a network call.

The upload request artifact is the first provider-agnostic upload contract. It
captures the validated artifact identity plus the rollout approval data needed by
future provider-specific adapters. Optional Hugging Face or other upload adapters
must consume this contract rather than inventing new approval or artifact
vocabulary.

## Consequences

- Upload planning becomes an explicit adapter contract without coupling core code
- to storage or vendor SDKs.
- Future provider implementations can stay thin and consume one validated upload
  request shape.
- The first upload slice is inspectable and testable offline because it materializes
  an artifact instead of requiring network access.
