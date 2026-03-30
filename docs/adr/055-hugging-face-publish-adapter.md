# ADR-055: Hugging Face Publish Adapter

## Context

The repo already has a provider-neutral upload request artifact. The next step
for a concrete provider is a thin adapter that consumes that request and maps it
onto the provider's own publishing flow without introducing a second approval or
artifact vocabulary.

For Hugging Face specifically, the practical fit is their `hf upload` CLI model:
push local files into a model repository by repo id and in-repo path.

## Decision

Add a thin Hugging Face publish adapter:

- `afterburner deploy hf-publish --request PATH`

The command:

- reads an existing `artifact_upload_request.json`,
- requires `provider = huggingface`,
- shells out to `hf upload` for the artifact file, manifest, request artifact,
  and any provider-neutral optimized package metadata listed in the upload request,
- and writes a `huggingface_publish_receipt.json` receipt.

Current stance: this is a thin provider adapter. It does not replace the
provider-neutral upload request contract and it does not move Hugging Face API
choices into core artifact logic.

## Consequences

- Hugging Face publishing now has one explicit adapter on top of the existing
  upload request contract.
- The adapter remains thin and provider-specific while approval and artifact
  identity stay provider-neutral.
- Future provider adapters can follow the same pattern instead of inventing
  bespoke core-facing contracts.
