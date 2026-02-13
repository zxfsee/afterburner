# Afterburner

## Quick start

```sh
# enter reproducible dev environment
nix develop

# train a model and export inference artifact
just train

# run inference using the exported artifact
just infer

# run deterministic eval guard and emit JSON summary
just eval-gate
```

`just infer` and `just eval` resolve the artifact through `artifacts/inference/current` by default.
Override with `--artifact <path>` (or positional `[artifact_path]` for compatibility).
The canonical CLI is `afterburner <train|infer|eval>`.

## What this is

Afterburner is a minimal Rust-based ML system demonstrating **training, inference, and reproducible infrastructure** using Burn and Nix.
It is intentionally small, but structured to reflect how production ML systems separate concerns between training, artifacts, and runtime.
While the example model uses MNIST, the system boundaries are designed for scientific ML workloads (e.g. molecular, protein, or graph-based models) where inference contracts, auditability, and deployment safety matter more than model accuracy.

## What this is not

This is not a production model, training pipeline, or serving system.
It is a boundary-focused reference implementation intended to demonstrate
system design judgment, not model performance.
It intentionally omits serving layers, rollout tooling, and multi-node scheduling concerns.

## Why this exists

This repository focuses on **system boundaries**, not model quality.

It demonstrates how to design ML systems where:
- training is experimental and mutable
- inference is stable, auditable, and deployable
- infrastructure choices are explicit and reproducible

The goal is to make training, artifacts, and runtime behavior easy to reason about and hard to misuse.

Future extensions are listed in [Changelog](./CHANGELOG.md).

## Design goals

- Reproducibility over convenience
- Explicit boundaries between training and inference
- Inspectable, versioned artifacts
- Minimal but intentional infrastructure
- The system must remain incrementally expandable across all defined research infrastructure domains without requiring architectural rewrites, favoring evolution behind stable contract boundaries.

## Training

Training is executed via the `afterburner train` subcommand.

It produces artifacts under `artifacts/`, including:
- a trained model output under `artifacts/train/`
- a stable, inference-ready model artifact under `artifacts/inference/`

Training code owns experimentation and optimization, but does **not** define the inference contract or runtime behavior.

## Inference

Inference is executed via the `afterburner infer` subcommand.

It consumes **only** the inference artifact and has no access to training internals.  
This enforces a strict boundary between model development and runtime execution.

## HTTP wrapper (optional)

Afterburner includes a minimal HTTP wrapper binary (`afterburner-http`) that exposes:
- `GET /healthz`
- `POST /infer` (accepts single or batched MNIST inputs, returns logits)

`POST /infer` accepted payloads:
- `application/octet-stream`: raw single image bytes (`784` bytes)
- `application/json`: single input (`{"base64":"..."}` or `{"bytes":[...]}`)
- `application/json`: batch input (`{"batch":[{"base64":"..."},{"bytes":[...]}]}`)

Envelope limits (env-configurable):
- `HTTP_MAX_REQUEST_BYTES` (default `65536`)
- `HTTP_MAX_BATCH_SIZE` (default `16`)
- `HTTP_INFER_TIMEOUT_MS` (default `1500`)
- `HTTP_MAX_CONCURRENCY` (default `4`)

Operational behavior:
- `GET /healthz` is handled in the HTTP adapter path (no tensor decode/preprocess/infer).
- `SIGINT`/`SIGTERM` triggers graceful shutdown (workers stop receiving new requests).
- Responses include `X-Request-Id`; logs include matching `request_id` for correlation.

Deterministic failures return JSON with stable `error.kind` values (for example `request_too_large`, `batch_too_large`, `infer_timeout`, `invalid_input`, `server_overloaded`).

The HTTP wrapper is intentionally thin and still uses the same inference path as the CLI.
The CLI remains the canonical interface; the HTTP binary is only a transport adapter.

Rationale is documented in [ADR-001: Training vs Inference Separation](./docs/adr/001-training-vs-inference.md).

## Artifact contract

Training exports versioned inference artifacts under `artifacts/inference/<version>/`, consisting of:
- serialized model weights (Burn `CompactRecorder`)
- a lightweight `manifest.toml` describing input shape/dtype, normalization, model architecture identity, artifact version, checksum, signature placeholders, and canonicalization metadata

The active version is tracked by `artifacts/inference/current`, enabling safe rollout/rollback without retraining.
Set `ARTIFACT_VERSION` to control the exported version.
If `signature.scheme != "none"`, inference requires non-empty `signature.key_id` and
`signature.value = "sha256:<hex>"`, where the digest matches canonicalized manifest input bytes.

Rollback example:
```sh
printf "0.1.0\n" > artifacts/inference/current
```

Inference binaries treat this artifact as immutable and consume it as their sole input.  
Training checkpoints, metrics, and logs are explicitly excluded from the inference contract.
Inference/HTTP/eval adapters emit normalized JSON events on stderr with envelope fields:
`ts_ms`, `level`, `source`, `event`, `fields`.
Set `AFTERBURNER_OBS_JSONL_PATH` to mirror the same event stream into an optional JSONL sink file.
The eval guard writes a deterministic summary to `artifacts/eval/mnist_eval_summary.json` and accepts
an optional artifact override via `afterburner eval --artifact <path>` (or legacy positional artifact).
Use `--min-accuracy <f64>` to turn eval into an acceptance gate; `just eval-gate` applies the repository baseline.
Baseline refresh flow when intended model changes shift deterministic accuracy:
1. Run `just eval`.
2. Review `artifacts/eval/mnist_eval_summary.json` and confirm the change is expected.
3. Update `justfile` `eval-gate` `--min-accuracy` to the approved deterministic value.

Training writes JSONL events to `artifacts/train/observability.jsonl` and Burn persists per-metric logs
under `artifacts/train/` for auditability.

This mirrors real-world model deployment, where training pipelines and serving environments are cleanly separated.

See [ADR-002: Artifact Contract](./docs/adr/002-artifact-contract.md) for rationale.

## CPU vs GPU execution

Training and inference are backend-agnostic.

The same model artifact can run on CPU or GPU without retraining:
- GPU is intended for training and throughput-oriented inference
- CPU is supported for development, debugging, and environments without accelerators

Backend selection is explicit (`BACKEND=cpu`), reflecting production systems where execution targets vary by cost, latency, and scale.
If `BACKEND` is unset, training attempts `wgpu` first and falls back to `cpu` automatically when no compatible GPU adapter is available.

## Infrastructure choices

This project uses **Rust, Burn, and Nix** to favor explicitness, reproducibility, and inspectability over rapid iteration.

The trade-off is slower experimentation in exchange for:
- deterministic builds
- clear system boundaries
- infrastructure that can be reasoned about end-to-end

This mirrors environments where deployment safety and auditability outweigh iteration speed.

## Documentation

- [Architecture overview](./ARCHITECTURE.md)
- [Design decisions (ADRs)](./docs/adr/)
- [Changelog](./CHANGELOG.md)

---

License: MIT
