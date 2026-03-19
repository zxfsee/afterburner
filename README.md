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
Override with `--artifact <path>`, `--artifact=<path>`, or positional `[artifact_path]` for compatibility.
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
Use `--batch-size <N>` and `--num-workers <N>` to exercise the explicit training scalability controls.

It produces artifacts under `artifacts/`, including:
- a trained model output under `artifacts/train/`
- a stable, inference-ready model artifact under `artifacts/inference/`
- a versioned training scalability contract at `artifacts/train/training_scalability_contract.json`
  capturing `batch_size`, `worker_parallelism`, `planned_samples`, and `throughput_samples_per_sec`
- a kernel adoption threshold contract at `artifacts/train/kernel_adoption_thresholds.json`
  capturing the current convolution footprint and the evidence threshold for custom kernel work

Training code owns experimentation and optimization, but does **not** define the inference contract or runtime behavior.

## Inference

Inference is executed via the `afterburner infer` subcommand.

It consumes **only** the inference artifact and has no access to training internals.  
This enforces a strict boundary between model development and runtime execution.
On success, `afterburner infer` writes a JSON object to stdout containing `logits` and `probabilities`.

## HTTP wrapper (optional)

Afterburner includes a minimal HTTP wrapper binary (`afterburner-http`) that exposes:
- `GET /healthz`
- `POST /infer` (accepts single or batched MNIST inputs, returns logits)

`POST /infer` accepted payloads:
- `application/octet-stream`: raw single image bytes (`784` bytes)
- `application/json`: single input (`{"base64":"..."}` or `{"bytes":[...]}`)
- `application/json`: batch input (`{"batch":[{"base64":"..."},{"bytes":[...]}]}`)

Success responses always use the batch-shaped envelope:
- single input: `{"batch_size":1,"logits":[[...]]}`
- batch input: `{"batch_size":N,"logits":[[...], ...]}`

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
- a lightweight `manifest.toml` describing input shape/dtype, precision metadata, normalization, model architecture identity, artifact version, checksum, signature placeholders, and canonicalization metadata

The active version is tracked by `artifacts/inference/current`, enabling safe rollout/rollback without retraining.
Set `ARTIFACT_VERSION` to control the exported version.
If `signature.scheme != "none"`, inference requires non-empty `signature.key_id` and
`signature.value = "sha256:<hex>"`, where the digest matches canonicalized manifest input bytes.
The manifest now also carries a `[precision]` section. The current runtime requires
`weights_dtype = "f32"`, `activation_dtype = "f32"`, and `quantization = "none"`;
unsupported reduced-precision artifacts fail fast at startup in both CLI and HTTP adapters.
Deployment-side work now has a canonical example target profile at
`fixtures/deployment_target_profile.example.json` alongside the schema fixture, so
future deploy-rs wiring can reference one inspectable baseline profile.
`just deploy-check` validates the baseline deploy-rs wiring from that profile contract
without requiring a real deployment target.
`afterburner upload --manifest <path> --ownership <path> --provider <name> --destination <ref>`
validates a versioned inference artifact plus rollout ownership approval and writes a provider-neutral
`artifacts/deploy/<artifact_version>/artifact_upload_request.json` plan artifact instead of talking
to any network service directly.
For local promotion flow, use `just rollout-check`, `just rollout-promote`, `just rollout-verify`,
and `just rollout-rollback` so eval, upload planning, deploy-check, current-pointer update, and
rollback all stay explicit and reversible.
For future data-oriented artifact surfaces, the current fit decision is conservative:
`Parquet` is the likely first columnar storage format if dataset/eval/export artifacts outgrow
JSON/TOML, while `DataFusion` and `Ballista` stay parked until there is a concrete analytical or
distributed query problem. No Arrow/DataFusion/Ballista dependency is added yet.
For pretraining data, the repo now distinguishes per-sample metadata from dataset-level manifests:
`pretraining_sample_metadata.schema.json` stays the sample contract, while
`pretraining_dataset_manifest.schema.json` pins corpus revision, shard inventory, split counts,
and checksum rollups for larger dataset refreshes.

Rollback example:
```sh
printf "0.1.0\n" > artifacts/inference/current
```

Inference binaries treat this artifact as immutable and consume it as their sole input.  
Training checkpoints, metrics, and logs are explicitly excluded from the inference contract.
Inference/HTTP/eval adapters emit normalized JSON events on stderr with envelope fields:
`ts_ms`, `level`, `source`, `event`, `fields`.
`afterburner infer` includes optional `fields.calibration` contract fields (`schema_version`, `calibration_artifact`, `artifact_version`, `method`, `created_at_unix_ms`) when `calibration_artifact_metadata.json` is present beside the selected artifact.
If that sidecar exists but fails schema/parse validation, or its `artifact_version` does not match the selected runtime artifact manifest, infer emits a `calibration_metadata_invalid` event and continues without `fields.calibration`.
If `artifacts/train/calibration_artifact_metadata.json` exists during export, training copies it into the
versioned inference artifact directory and `artifact_exported` surfaces both `calibration_metadata_path`
and the same calibration fields explicitly, so later upload/promote/deploy steps do not have to infer
post-training calibration state from an untracked local side file.
Set `AFTERBURNER_OBS_JSONL_PATH` to mirror the same event stream into an optional JSONL sink file.
`infer_done` now uses the same duration and identity vocabulary as `eval_done`: both infer adapters emit
`artifact_version`, `batch_size`, `duration_ms`, and a `precision` block
(`weights_dtype`, `activation_dtype`, `quantization`); the HTTP adapter also includes `request_id`
for per-request correlation. `artifact_load_ok` in both infer adapters mirrors the same `precision`
block so reduced-precision artifacts are visible at load time, not only after inference completes.
CLI infer now also writes `artifacts/eval/infer_output_drift_summary.json` plus an
`infer_output_drift_summary_written` event, giving promotion and rollback checks a compact
comparison artifact (`predicted_class`, `top_probability`, `margin_to_second`, and output digests)
without persisting the full logits from every run.
Use `just drift-receipt <candidate-summary> <current-summary> <policy>` to materialize
`artifacts/eval/infer_output_drift_receipt.json`, a pass/fail receipt driven by one named
drift-policy artifact instead of ad-hoc CLI thresholds at rollout time.
Use `just drift-baseline <summary> <receipt>` to materialize `artifacts/eval/infer_output_drift_baseline.json`,
the approved reference snapshot that later drift receipts should compare against instead of whichever
current summary happens to be on disk at rollout time.
Use `just drift-approve-baseline <baseline> <approved-by> <approval-ticket> <approved-at-unix-ms>`
to materialize `artifacts/eval/infer_output_drift_baseline_approval.json`, the explicit approval
record that distinguishes a freshly computed baseline from one approved for rollout decisions.
Use `just drift-point-approved-baseline <approval>` to materialize `artifacts/eval/infer_output_drift_baseline_pointer.json`,
the single pointer artifact that identifies which approved baseline rollout tooling should treat as
current.
Use `just drift-record-approved-baseline-history <pointer> <event> <recorded-at-unix-ms>` to append
to `artifacts/eval/infer_output_drift_baseline_history.json`, the compact audit trail of approved-baseline
pointer changes over time.
Use `just drift-rollback-approved-baseline <current-pointer> <restored-approval> <rolled-back-at-unix-ms>`
to restore the approved-baseline pointer to a prior approval and write
`artifacts/eval/infer_output_drift_baseline_rollback.json`, keeping rollback decisions explicit and
auditable.
Use `just drift-supersede-baseline-approval <previous-approval> <next-approval> <superseded-at-unix-ms>`
to materialize `artifacts/eval/infer_output_drift_baseline_supersession.json`, the lineage record
that marks an older baseline approval as superseded by a newer approved baseline.
Use `just drift-refresh-baseline <summary> <receipt> <current-baseline>` to replace an approved
baseline while archiving the previous one and writing `artifacts/eval/infer_output_drift_baseline_refresh.json`,
so summary, receipt, policy, and baseline transitions remain synchronized.
The eval guard writes a deterministic summary to `artifacts/eval/mnist_eval_summary.json` and emits
an `eval_done` event with RED-style monitoring fields (`rate_samples_per_sec`, `error_count`, `duration_ms`)
plus eval context (`accuracy`, `samples`, `artifact_version`, `batches_evaluated`, `batch_size`, `seed`).
It accepts an optional artifact override via `afterburner eval --artifact <path>` (or legacy positional artifact).
Use `--min-accuracy <f64>` to turn eval into an acceptance gate; `just eval-gate` applies the repository baseline.
`artifacts/eval/backend_performance_profile.json` pins the current runtime decision rule for `wgpu`
versus `cpu`, and `just backend-profile-gate` validates the objective thresholds for introducing a
native backend instead of extending the current stack by assumption. The profile now also carries a
parsed `comparison_baseline` block (`seed`, `batch_size`, `max_batches`, `samples_per_profile`) and
explicit `refresh_when` conditions so command drift and backend-decision drift fail mechanically.
For hotspot work, use `just profile-infer` to write a deterministic infer flamegraph
under `artifacts/profiling/` and a parsed hotspot summary at
`artifacts/profiling/infer_hotspot_summary.json`. On macOS, this requires full Xcode
selected so `xcrun xctrace version` succeeds. The recipe clears `DEVELOPER_DIR` and
`SDKROOT` and forces `XCTRACE=/usr/bin/xctrace` so the system Instruments templates
win over the Nix Apple SDK environment.
The current profiling summary intentionally stops short of OpenTelemetry trace/resource
correlation: use its local identity fields (`artifact_version`, `backend`, `weights_artifact`,
`profile_command`) for now, and treat explicit OpenTelemetry linkage as a later adapter-only
extension if a concrete workflow needs it.
`artifacts/train/kernel_adoption_thresholds.json` pins the current custom-kernel decision rule:
the present model requires coverage of the checked `1x1`, `3x3`, and `5x5` convolution footprint,
and backend profile regressions must remain sustained before replacing backend-provided kernels.
If that threshold is ever crossed, `CubeCL`/`CubeK` are the first implementation path to
evaluate because the current Burn GPU stack already carries them transitively; no direct
`cubecl` or `cubek` dependency is added until that measured need exists.
For future RL work, keep the contract surface at `fixtures/rl_rollout_metadata.schema.json`
first: one single-environment metadata artifact with a stable `environment` identity is the
current fit, while simulator bindings, rollout-step payloads, and vectorized environment
orchestration stay deferred until a measured need exists.
For eventual multibillion-scale work, treat the target envelope as contract-first rather than
runtime-first: `training_scalability_contract.json`, `distributed_shard_metadata.schema.json`,
manifest `precision`, `artifact_rollout_ownership.schema.json`,
`artifact_upload_request.schema.json`, and `deployment_target_profile.schema.json` are the
minimum surfaces that must remain explicit before claiming larger-scale training or deployment
readiness.
If distributed checkpoint recovery is added later, the repo expects a checkpoint index contract
that declares `artifact_version`, `checkpoint_root`, `shard_count`, and `shard_metadata_path`
explicitly instead of reconstructing shard membership from directory layout.
Baseline refresh flow when intended model changes shift deterministic accuracy:
1. Run `just eval`.
2. Review `artifacts/eval/mnist_eval_summary.json` and confirm the change is expected.
3. Update `justfile` `eval-gate` `--min-accuracy` to the approved deterministic value.

Training writes JSONL events to `artifacts/train/observability.jsonl` and Burn persists per-metric logs
under `artifacts/train/` for auditability. `train_done` mirrors the training scalability contract
fields (`batch_size`, `worker_parallelism`, `num_epochs`, `planned_samples`, `elapsed_ms`,
`throughput_samples_per_sec`) for stable operator-facing observability.
`artifact_exported` also has a pinned fixture-backed shape: `backend`, `artifact_version`,
`artifact_path`, `manifest_path`, and `current_path` stay stable so promotion-facing tooling can
consume one explicit export event contract.
`train_start` is now fixture-gated as well, pinning the operator-facing start-of-run fields
(`batch_size`, `worker_parallelism`, `num_epochs`, `planned_samples`, `metrics_dir`, `inference_dir`)
before any training work completes.
Use `afterburner train --num-epochs 1` when you need to regenerate the training-side contract
artifacts in a short deterministic CI or local verification run without changing the default
training horizon.
`afterburner-dashboard` provides a terminal adapter over the same structured event stream:
```sh
cargo run --bin afterburner-dashboard -- --input artifacts/train/observability.jsonl
```
Use `just dashboard` to open the same dashboard with the current infer profiling summary
loaded through `--profiling-summary artifacts/profiling/infer_hotspot_summary.json`.
Press `q` or `Esc` to exit live mode. For deterministic CI coverage, use:
```sh
cargo run --bin afterburner-dashboard -- --input fixtures/dashboard_events.jsonl --profiling-summary fixtures/dashboard_profiling_summary.json --snapshot --width 80 --height 18
```

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
