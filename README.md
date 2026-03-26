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
The canonical workflow surface is `just`; use `just --list` or `just workflows`
to discover repo-managed entrypoints. The `afterburner <...>` CLI remains the
underlying contract surface behind those recipes.

## At a glance

- Rust-first ML systems repo focused on boundaries, contracts, and reproducible workflows.
- Uses Burn for model/runtime work, Nix for reproducible environments, and `just` as the workflow surface.
- Optimized for clarity of training, artifact, deployment, and observability contracts over model quality benchmarks.

## System shape

- Training/runtime: model execution, artifact export, checkpointing, and training-side observability.
- Control plane: rollout, upload, cleanup, and deployment-side verification artifacts.
- Platform: local MacBook workflows first, then NixOS nodes and later cluster adapters.

## Read next

- [Architecture](./ARCHITECTURE.md): boundaries, invariants, and the current system shape
- [Workflow reference](./docs/workflows.md): operational recipes and artifact/receipt paths
- [Reference index](./docs/reference.md): contract and capability pointers that do not belong on the frontpage
- [ADRs](./docs/adr/): concrete architectural and contract decisions
- [Changelog](./CHANGELOG.md): active TODO queue and recent landed work

## What this is

Afterburner is a Rust-first ML systems repo built around explicit boundaries:
training, artifacts, inference, deployment, and observability each get their
own contract surface.

It uses Burn for model/runtime work, Nix for reproducible environments, and
`just` as the workflow surface. The example workloads stay small on purpose,
but the repo is structured to reflect how larger ML systems avoid blurring
training code, runtime code, and deployment contracts together.

## What this is not

This is not a production benchmark repo or a full serving platform.

It is a boundary-focused systems repo. The point is to make system shape and
contract edges explicit and mechanically checkable, not to maximize model
quality on MNIST.

## Why Afterburner

Most ML repos are clear about models and unclear about contracts. Afterburner
optimizes for the opposite.

The repo exists to show how training can stay experimental while inference,
deployment, and audit surfaces stay explicit. That is the main value here.

In practice, that means:
- training stays mutable
- inference stays stable and deployable
- infrastructure choices stay explicit and reproducible
- evolution happens behind inspectable contract boundaries

Future extensions are listed in [Changelog](./CHANGELOG.md).

## Design goals

- Reproducibility over convenience
- Explicit boundaries between training and inference
- Inspectable, versioned artifacts
- Minimal but intentional infrastructure
- The system must remain incrementally expandable across all defined research infrastructure domains without requiring architectural rewrites, favoring evolution behind stable contract boundaries.

If you are orienting in the repo, read in this order:
- this README for project shape and workflow entrypoints
- [Architecture](./ARCHITECTURE.md) for boundaries and invariants
- [ADRs](./docs/adr/) for individual decisions
- [Changelog](./CHANGELOG.md) for the active TODO horizon

As the repo grows toward distributed training, it keeps three layers separate:
the distributed training runtime decides how one job uses GPUs, the scheduler
decides who gets GPUs and when, and the platform layer provides the execution
substrate on MacBook, NixOS, or Kubernetes. See
[ADR-034: Distributed Runtime, Scheduler, And Platform Separation](./docs/adr/034-distributed-runtime-scheduler-platform-separation.md).
For modest clusters, scheduler policy is intentionally conservative: queueing
and priority come first, supported preemption is cooperative checkpoint/resume
only, and GPU colocation is treated as a separate resource-management mode
rather than transparent pause/resume.
If GPU colocation is added later, it should stay opt-in and limited to known
low-saturation workloads with measured interference bounds; it is not meant to
be the default scheduler policy.
The runtime/scheduler lifecycle boundary is explicit as well: scheduler-side
messages carry `START`/`STOP`/`KILL`, runtime-side messages carry
`READY`/`CHECKPOINTED`/`FAILED`/`HEARTBEAT`, and `START` includes explicit
lease-owned resources and rank assignments. See
[ADR-045: GPU Scheduler Boundary And Lifecycle](./docs/adr/045-gpu-scheduler-boundary-and-lifecycle.md).
The current single-node scheduler path is intentionally bounded: `afterburner deploy single-node-scheduler`
admits one-GPU jobs onto the first free local GPU, writes a lease artifact plus
a systemd service unit, and uses `SIGTERM` for graceful stop. It is a
single-node adapter, not a queue daemon or cluster scheduler.
For MacBook-local multi-process orchestration, `process-compose-flake` is the
current best-fit substrate candidate because it matches the repo's `flake-parts`
setup. It is a local process substrate only, not a replacement for `systemd`,
Kubernetes, or the scheduler layer.
Distributed-training work is also intentionally workload-driven: the goal is the
minimum useful capability subset for repo workloads, not parity with a general
DeepSpeed-class framework. See
[ADR-035: Distributed Training Capability Subset Scope](./docs/adr/035-distributed-training-capability-subset-scope.md).

## Training

Training is executed via the `afterburner train` subcommand.
Use `--batch-size <N>` and `--num-workers <N>` to exercise the explicit training scalability controls.
Use `--task text --dataset-manifest <path> --tokenizer-profile <path> --token-cache <path>`
to run the bounded text-pretraining adapter over a deterministic local token cache.
`just train-text <dataset-manifest> <tokenizer-profile> <token-cache>` is the
canonical harness entrypoint for that path.

It produces artifacts under `artifacts/`, including:
- a trained model output under `artifacts/train/`
- a stable, inference-ready model artifact under `artifacts/inference/`
- a versioned training scalability contract at `artifacts/train/training_scalability_contract.json`
  capturing `batch_size`, `worker_parallelism`, `planned_samples`, and `throughput_samples_per_sec`
- a kernel adoption threshold contract at `artifacts/train/kernel_adoption_thresholds.json`
  capturing the current convolution footprint and the evidence threshold for custom kernel work

For `--task text`, the adapter also writes:
- `artifacts/train/text_pretraining_run.json`
- Burn learner checkpoints under `artifacts/train/text/checkpoint/`
- `artifacts/eval/text_pretraining_eval_summary.json`
- `artifacts/text_inference/<version>/model.mpk`
- `artifacts/text_inference/<version>/text_inference_profile.json`

Training code owns experimentation and optimization, but does **not** define the inference contract or runtime behavior.
For distributed-training planning, keep one distinction explicit: `worker_parallelism` is only a
local throughput knob in the current trainer. Future Burn-side distributed execution should align
with Burn's strategy-oriented training model rather than stretching `worker_parallelism` into a
distributed topology surrogate. See
[ADR-038: Burn Distributed Learning Strategy Fit](./docs/adr/038-burn-distributed-learning-strategy-fit.md).
That also means topology stays explicit: future distributed execution should
declare `world_size`, ranks, and `device_group` metadata directly instead of
inferring them from shard ownership or local worker ids. See
[ADR-039: Distributed World And Rank Topology](./docs/adr/039-distributed-world-rank-topology.md).
The current distributed runtime capability surface is still narrower than Burn's
longer-term strategy vocabulary: Afterburner supports single-device execution
today, treats DP as the first candidate expansion, and does not claim ZeRO,
TP, PP, SP/CP, or EP support yet. See
[ADR-041: Distributed Runtime Capability Surface](./docs/adr/041-distributed-runtime-capability-surface.md).
Candidate distributed layouts should also be screened through one explicit
feasibility artifact before benchmark or training launch, so invalid DP/TP/PP
and inventory combinations get rejected before runtime work starts. See
[ADR-054: Distributed Runtime Layout Feasibility](./docs/adr/054-distributed-runtime-layout-feasibility.md).
When the repo starts recording executed distributed runtime trials, use one
explicit profile artifact for model-size and node-count cells rather than
stretching either `training_scalability_contract.json` or
`distributed_load_profile.json`. That profile should record parallelism choices,
MFU/memory metrics, runtime settings, and environment fingerprint explicitly.
See [ADR-043: Distributed Runtime Profile Schema](./docs/adr/043-distributed-runtime-profile-schema.md).

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
Detailed operational workflow reference lives in [docs/workflows.md](./docs/workflows.md).

<details>
<summary>Operational workflow index</summary>

- Deployment:
  `fixtures/deployment_stack_profile.example.json` defines the stack profile contract;
  `just deploy-check` -> `deployment_stack_check.json`
  `just deploy-launch-plan` -> `deployment_stack_launch_plan.json`
  `just deploy-launch-receipt` -> `deployment_stack_launch_receipt.json`
  `just deploy-launch-bundle` -> `deployment_stack_launch_evidence_bundle.json`
  `just distributed-load-profile` -> `distributed_load_profile.json`
  `afterburner deploy hf-publish --request <path>` -> `huggingface_publish_receipt.json`
- Rollout verification:
  `just rollout-check`, `just rollout-promote`, `just rollout-verify`, `just rollout-rollback`
  `just deployment-verification-receipt` -> `deployment_verification_receipt.json`
  `just deployment-verification-bundle` -> `deployment_verification_evidence_bundle.json`
  `just deployment-verification-handoff` -> `deployment_verification_evidence_handoff.json`
- Cleanup:
  `just cleanup-inventory` -> `artifact_cleanup_inventory.json`
  `just cleanup-policy` -> `artifact_cleanup_policy.json`
  `just cleanup-dry-run` -> `artifact_cleanup_dry_run_receipt.json`
  `just cleanup-execute` -> `artifact_cleanup_execution_receipt.json`
  `just cleanup-evidence-bundle` -> `artifact_cleanup_evidence_bundle.json`
- Profiling:
  `just profile-infer` -> `infer_hotspot_summary.json`
  `just profile-environment-snapshot` -> `profiling_environment_snapshot.json`
  `just profile-refresh-environment-snapshot` -> `profiling_environment_snapshot_refresh.json`
  `just profile-provenance-receipt` -> `profiling_provenance_receipt.json`
  `just profile-provenance-bundle` -> `profiling_provenance_evidence_bundle.json`
- Data provenance:
  `just pretraining-source-approval-receipt` -> `pretraining_source_approval_receipt.json`
  `just pretraining-source-provenance-receipt` -> `pretraining_source_provenance_receipt.json`
  `just pretraining-source-provenance-evidence-bundle` -> `pretraining_source_provenance_evidence_bundle.json`
  `just distributed-shard-lineage-receipt` -> `distributed_shard_lineage_receipt.json`
  `just distributed-shard-lineage-evidence-bundle` -> `distributed_shard_lineage_evidence_bundle.json`
  `just distributed-shard-lineage-handoff` -> `distributed_shard_lineage_evidence_handoff.json`
</details>

For pretraining data, the repo now distinguishes per-sample metadata from dataset-level manifests:
`pretraining_sample_metadata.schema.json` stays the sample contract, while
`pretraining_dataset_manifest.schema.json` pins corpus revision, shard inventory, split counts, and checksum rollups for larger dataset refreshes.

Deeper contract and capability detail now lives in [docs/reference.md](./docs/reference.md).

<details>
<summary>Contract and capability index</summary>

## Capability index

- Training-side observability stays explicit: `artifacts/train/observability.jsonl`, `train_start`, `train_done`, and `artifact_exported` keep stable fields such as `backend`, `artifact_version`, `artifact_path`, `manifest_path`, and `current_path`. `afterburner train --num-epochs 1` remains the short contract-refresh path.
- Inference-side contract surfaces stay inspectable: inference binaries consume the immutable versioned artifact only, may surface `calibration_artifact_metadata.json`, honor `AFTERBURNER_OBS_JSONL_PATH`, and emit normalized event envelopes with `ts_ms`, `level`, `source`, `event`, and `fields`.
- Data and text pretraining stay bounded: the MacBook text-pretraining target remains a bounded decoder-only language model over `fineweb-edu/slice`; `afterburner train --task text`, `text_token_cache.schema.json`, `artifacts/text_inference/<version>/`, `text_pretraining_run.json`, `artifacts/eval/text_pretraining_eval_summary.json`, `just train-text-smoke`, and the tokenizer and packing side keep the current text path explicit. Text-trained models should not reuse the current MNIST/logits infer surface.
- Source identity stays explicit: `source` and `source_revision` remain stable dataset identifiers, with `pretraining_source_approval_receipt.json`, the source provenance receipt `pretraining_source_provenance_receipt.json`, `pretraining_source_provenance_evidence_bundle.json`, and the future pretraining source registry contract keeping `upstream_locator`, `license`, and `approval_status` visible.
- Drift and eval are approval-driven: `infer_output_drift_summary.json`, `infer_output_drift_receipt.json`, `infer_output_drift_baseline.json`, `infer_output_drift_baseline_approval.json`, `infer_output_drift_baseline_pointer.json`, `infer_output_drift_baseline_history.json`, `infer_output_drift_baseline_checkpoint.json`, `infer_output_drift_baseline_bundle.json`, `infer_output_drift_baseline_handoff.json`, `infer_output_drift_baseline_transport_locator.json`, `infer_output_drift_baseline_rollback.json`, `infer_output_drift_baseline_supersession.json`, and `infer_output_drift_baseline_refresh.json` remain explicit rollout artifacts. `just eval-gate`, `just drift-receipt`, `just drift-baseline`, `just drift-approve-baseline`, `just drift-point-approved-baseline`, `just drift-record-approved-baseline-history`, `just drift-checkpoint-baseline`, `just drift-export-baseline-bundle`, `just drift-export-baseline-handoff`, `just drift-point-baseline-transport-locator`, `just drift-rollback-approved-baseline`, `just drift-supersede-baseline-approval`, and `just drift-refresh-baseline` stay as the operator workflows.
- Artifact lifecycle is explicit: the current artifact retention envelope keeps `artifacts/inference/current`, the active `artifacts/inference/<version>/` directory, current rollout evidence, and current train/eval state; `artifacts/profiling/` remains the profiling retention policy boundary for prune candidates.
- Profiling keeps one explicit local-first contract: `just profile-infer` follows the current pointer and active `BACKEND` contract; it writes `infer_hotspot_summary.json`, `profiling_environment_snapshot.json`, `profiling_environment_snapshot_refresh.json`, `profiling_provenance_receipt.json`, `profiling_provenance_evidence_bundle.json`, `distributed_runtime_benchmark_run.json`, and `distributed_runtime_profile.json`. The profiling environment provenance contract stays explicit through `host_os`, `host_arch`, `profiler_version`, and `profiler_path`. On macOS this still requires `xcrun xctrace version` under full Xcode, and the recipe forces `XCTRACE=/usr/bin/xctrace` while clearing `DEVELOPER_DIR` and `SDKROOT`. The profiling hotspot taxonomy remains execution/framework/compiler-runtime/incidental, and OpenTelemetry linkage stays a later adapter-only fit.
- Operator CLI families stay grouped: `afterburner deploy <subcommand>`, `afterburner drift <subcommand>`, `afterburner cleanup <subcommand>`, and `afterburner profile <subcommand>` remain the canonical grouped command surface instead of flat operator verbs.
- Runtime and backend decisions stay measured: `backend_performance_profile.json` and `just backend-profile-gate` gate CPU vs `wgpu`; `just dashboard` remains the terminal observability entrypoint; `kernel_adoption_thresholds.json` guards custom kernels; `CubeCL` and `CubeK` remain the first backend-extension path if that threshold is crossed; `cutile-rs` stays parked as a later NVIDIA-specific backend-extension candidate; Burn 0.20.1 stays pinned until a separate `.mpk` to `.bpk` migration is justified.
- Deployment and remote artifact movement stay adapter-first: `artifact_upload_request.json` remains the provider-neutral upload artifact, `afterburner deploy hf-publish --request <path>` writes `huggingface_publish_receipt.json`, `AFTERBURNER_TRACEPARENT` remains the tracing correlation propagation hook for deployment-side workflows, and any future remote save/load path must stay behind a provider-neutral remote locator contract rather than a storage-SDK-specific path.
- Deployment verification evidence provenance stays explicit through `evidence_sources` on the deployment verification receipt, including `artifact_path`, `event_name`, and `observed_at_unix_ms`.
- Optimization work stays orthogonal to scale: quantization, compression, export, and packaging are a separate post-training pipeline, not runtime/scheduler logic.
- The distributed shard lineage review path stays explicit: `distributed_shard_lineage_receipt.json`, `distributed_shard_lineage_evidence_bundle.json`, and `distributed_shard_lineage_evidence_handoff.json` define the current lineage review path; lineage evidence provenance stays explicit through `evidence_sources`, `metadata_path`, `checkpoint_root`, and `observed_at_unix_ms`; and the future distributed checkpoint index contract should keep `artifact_version`, `checkpoint_root`, `shard_count`, and `shard_metadata_path` explicit.
- Future envelopes stay contract-first: `fixtures/rl_rollout_metadata.schema.json`, the vectorized-environment stance, the multibillion-scale target envelope, the shard metadata contract `distributed_shard_metadata.schema.json`, `Parquet` as the likely first larger data format, and parked `DataFusion`/`Ballista` fit decisions all remain explicit planning surfaces rather than implied implementation scope.

</details>

See [ADR-002: Artifact Contract](./docs/adr/002-artifact-contract.md) for rationale.

## CPU vs GPU execution

Training and inference are backend-agnostic.

The same model artifact can run on CPU or GPU without retraining:
- GPU is intended for training and throughput-oriented inference
- CPU is supported for development, debugging, and environments without accelerators

Backend selection is explicit (`BACKEND=cpu|wgpu|metal`), reflecting production systems where execution targets vary by cost, latency, and scale.
On Apple hardware, `BACKEND=metal` uses Burn's Metal-backed GPU path under the existing Burn GPU stack.
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
