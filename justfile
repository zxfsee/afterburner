set shell := ["nu", "-c"]
set dotenv-load := true

profile-infer-artifact := "artifacts/inference/0.1.0/model.mpk"
profile-infer-flamegraph := "artifacts/profiling/infer_flamegraph.svg"
profile-infer-summary := "artifacts/profiling/infer_hotspot_summary.json"
profile-infer-backend := "wgpu"
profile-infer-command := "env -u DEVELOPER_DIR -u SDKROOT XCTRACE=/usr/bin/xctrace cargo flamegraph --dev --deterministic --bin afterburner -o artifacts/profiling/infer_flamegraph.svg -- infer artifacts/inference/0.1.0/model.mpk"

# build the project via nix (reproducible)
build:
    nix build

# run all checks (fmt, clippy, audit, tests)
check:
    nix flake check

# quick Rust typecheck (uses Cargo cache)
cargo-check:
    cargo check --locked

# enter development shell
dev:
    nix develop

# format rust + toml + nix
fmt:
    nix fmt

# train model and produce artifacts
train:
    cargo run --locked --bin afterburner -- train

# run inference using trained artifact
infer:
    cargo run --locked --bin afterburner -- infer

# run deterministic MNIST eval and write JSON summary
eval:
    cargo run --locked --bin afterburner -- eval --seed 42 --batch-size 128 --max-batches 8 --out artifacts/eval/mnist_eval_summary.json

# CI-friendly eval regression gate
eval-gate:
    cargo run --locked --bin afterburner -- eval --seed 42 --batch-size 128 --max-batches 8 --min-accuracy 0.98925781 --out artifacts/eval/mnist_eval_summary.json

# validate backend performance decision thresholds
backend-profile-gate:
    cargo test --test perf_backend_profile

# validate deploy-rs baseline deployment definitions
deploy-check:
    cargo run --locked --bin afterburner -- deploy stack-check --target-profile fixtures/deployment_target_profile.example.json --stack-profile fixtures/deployment_stack_profile.example.json --out artifacts/deploy/deployment_stack_check.json
    nix eval .#checks.aarch64-darwin.deploy-activate.drvPath
    nix eval .#checks.aarch64-darwin.deploy-schema.drvPath

# validate a candidate artifact before promotion
rollout-check candidate_artifact candidate_manifest ownership provider destination:
    cargo run --locked --bin afterburner -- eval --artifact {{ candidate_artifact }} --seed 42 --batch-size 128 --max-batches 8 --min-accuracy 0.98925781 --out artifacts/eval/mnist_eval_summary.json
    cargo run --locked --bin afterburner -- deploy upload --manifest {{ candidate_manifest }} --ownership {{ ownership }} --provider {{ provider }} --destination {{ destination }} --out artifacts/deploy/candidate_upload_request.json
    just deploy-check

single-node-scheduler job inventory:
    cargo run --locked --bin afterburner -- deploy single-node-scheduler --job {{ job }} --inventory {{ inventory }} --out-lease artifacts/deploy/gpu_scheduler_lease.json --out-unit artifacts/deploy/afterburner-job.service

distributed-load-profile addr requests concurrency latency_budget_ms_p99 error_budget_ratio:
    cargo run --locked --bin afterburner -- deploy load-profile --addr {{ addr }} --requests {{ requests }} --concurrency {{ concurrency }} --latency-budget-ms-p99 {{ latency_budget_ms_p99 }} --error-budget-ratio {{ error_budget_ratio }} --out artifacts/deploy/distributed_load_profile.json

# package receipt-referenced deployment verification evidence into one bundle artifact
deployment-verification-bundle receipt:
    cargo run --locked --bin afterburner -- deploy verification-bundle --receipt {{ receipt }} --out artifacts/deploy/deployment_verification_evidence_bundle.json

# inventory retained paths and clear prune candidates before planning cleanup
cleanup-inventory:
    cargo run --locked --bin afterburner -- cleanup inventory --artifacts-root artifacts --out artifacts/deploy/artifact_cleanup_inventory.json

# write one named cleanup policy profile for later dry-run and execution receipts
cleanup-policy profile:
    cargo run --locked --bin afterburner -- cleanup policy --profile {{ profile }} --out artifacts/deploy/artifact_cleanup_policy.json

# compare candidate and current infer drift summaries using a named policy profile
drift-receipt candidate_summary current_summary policy:
    cargo run --locked --bin afterburner -- drift receipt --candidate {{ candidate_summary }} --current {{ current_summary }} --policy {{ policy }} --out artifacts/eval/infer_output_drift_receipt.json

# capture a checked baseline snapshot from an approved infer drift receipt
drift-baseline summary receipt:
    cargo run --locked --bin afterburner -- drift baseline --summary {{ summary }} --receipt {{ receipt }} --out artifacts/eval/infer_output_drift_baseline.json

# approve a baseline snapshot for rollout decisions
drift-approve-baseline baseline approved_by approval_ticket approved_at_unix_ms:
    cargo run --locked --bin afterburner -- drift approve-baseline --baseline {{ baseline }} --approved-by {{ approved_by }} --approval-ticket {{ approval_ticket }} --approved-at-unix-ms {{ approved_at_unix_ms }} --out artifacts/eval/infer_output_drift_baseline_approval.json

# point rollout tooling at the currently approved baseline approval record
drift-point-approved-baseline approval:
    cargo run --locked --bin afterburner -- drift point-approved-baseline --approval {{ approval }} --out artifacts/eval/infer_output_drift_baseline_pointer.json

# append one approved-baseline pointer change to the compact history artifact
drift-record-approved-baseline-history pointer event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- drift record-approved-baseline-history --pointer {{ pointer }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/eval/infer_output_drift_baseline_history.json

# capture a compact checkpoint of the current approved-baseline pointer plus recent history
drift-checkpoint-baseline pointer history:
    cargo run --locked --bin afterburner -- drift checkpoint-baseline --pointer {{ pointer }} --history {{ history }} --out artifacts/eval/infer_output_drift_baseline_checkpoint.json

# export one packaged baseline bundle for deployment-side consumers
drift-export-baseline-bundle pointer history:
    cargo run --locked --bin afterburner -- drift export-baseline-bundle --pointer {{ pointer }} --history {{ history }} --out artifacts/eval/infer_output_drift_baseline_bundle.json

# export a stable handoff manifest from the approved baseline bundle
drift-export-baseline-handoff bundle:
    cargo run --locked --bin afterburner -- drift export-baseline-handoff --bundle {{ bundle }} --out artifacts/eval/infer_output_drift_baseline_handoff.json

# point transport-facing workflows at the current baseline handoff manifest
drift-point-baseline-transport-locator handoff:
    cargo run --locked --bin afterburner -- drift point-baseline-transport-locator --handoff {{ handoff }} --out artifacts/eval/infer_output_drift_baseline_transport_locator.json

# restore the approved baseline pointer to a previous approval and write a rollback record
drift-rollback-approved-baseline current_pointer restored_approval rolled_back_at_unix_ms:
    cargo run --locked --bin afterburner -- drift rollback-approved-baseline --current-pointer {{ current_pointer }} --restored-approval {{ restored_approval }} --rolled-back-at-unix-ms {{ rolled_back_at_unix_ms }} --out-pointer artifacts/eval/infer_output_drift_baseline_pointer.json --out-record artifacts/eval/infer_output_drift_baseline_rollback.json

# supersede an older baseline approval with a newer approved baseline
drift-supersede-baseline-approval previous_approval next_approval superseded_at_unix_ms:
    cargo run --locked --bin afterburner -- drift supersede-baseline-approval --previous-approval {{ previous_approval }} --next-approval {{ next_approval }} --superseded-at-unix-ms {{ superseded_at_unix_ms }} --out artifacts/eval/infer_output_drift_baseline_supersession.json

# refresh the approved drift baseline, archiving the old one and writing a refresh receipt
drift-refresh-baseline summary receipt current_baseline:
    cargo run --locked --bin afterburner -- drift refresh-baseline --summary {{ summary }} --receipt {{ receipt }} --current-baseline {{ current_baseline }} --out artifacts/eval/infer_output_drift_baseline.json --archive artifacts/eval/infer_output_drift_baseline.previous.json --refresh-receipt artifacts/eval/infer_output_drift_baseline_refresh.json

# promote a vetted artifact version by updating the current pointer and saving the previous one
rollout-promote artifact_version:
    mkdir artifacts/deploy
    open artifacts/inference/current | str trim | save --force artifacts/deploy/previous_current_version.txt
    "{{ artifact_version }}" | save --force artifacts/inference/current

# verify the promoted current pointer still resolves and passes the eval gate
rollout-verify candidate_artifact:
    cargo run --locked --bin afterburner -- infer
    cargo run --locked --bin afterburner -- eval --artifact {{ candidate_artifact }} --seed 42 --batch-size 128 --max-batches 8 --min-accuracy 0.98925781 --out artifacts/eval/mnist_eval_summary.json

# restore the previous current pointer after a failed rollout or explicit rollback
rollout-rollback:
    open artifacts/deploy/previous_current_version.txt | str trim | save --force artifacts/inference/current

# capture a deterministic infer flamegraph into artifacts/profiling
# on macOS this requires `xcrun xctrace version` under full Xcode. The repo
# shell clears Nix Apple SDK overrides and forces `/usr/bin/xctrace` so the

# system Instruments templates win over the xcbuild wrapper environment.
profile-environment-snapshot:
    cargo run --locked --bin afterburner -- profile environment-snapshot --profile-kind infer --profiler cargo-flamegraph --profiler-path cargo-flamegraph --out artifacts/profiling/profiling_environment_snapshot.json

profile-refresh-environment-snapshot current_snapshot profiler_path captured_at_unix_ms:
    cargo run --locked --bin afterburner -- profile refresh-environment-snapshot --current-snapshot {{ current_snapshot }} --profiler-path {{ profiler_path }} --captured-at-unix-ms {{ captured_at_unix_ms }} --out artifacts/profiling/profiling_environment_snapshot.json --refresh-receipt artifacts/profiling/profiling_environment_snapshot_refresh.json

profile-infer:
    mkdir artifacts/profiling
    rm -rf cargo-flamegraph.trace
    {{ profile-infer-command }}
    cargo run --locked --bin afterburner_profile_summary -- --input {{ profile-infer-flamegraph }} --output {{ profile-infer-summary }} --weights-artifact {{ profile-infer-artifact }} --backend {{ profile-infer-backend }} --profile-command '{{ profile-infer-command }}'
    just profile-environment-snapshot

# open the terminal dashboard with profiling summary context
dashboard:
    cargo run --locked --bin afterburner-dashboard -- --input artifacts/train/observability.jsonl --profiling-summary artifacts/profiling/infer_hotspot_summary.json

# validate workspace/core dependency boundaries
workspace-gate:
    cargo test --test workspace_dependency_gate
    cargo test --test build_graph_guard
    cargo test -p afterburner-core

# run training by default
run:
    just train

# clean build + artifacts
clean:
    rm -rf target artifacts result

# update flake inputs
update:
    nix flake update

# update Rust dependencies
cargo-update:
    cargo update

# run tests via nextest (fast, consistent)
test:
    cargo nextest run

# run unit/integration tests with cargo (fallback)
test-cargo:
    cargo test

# regenerate CHANGELOG.md from git history
changelog:
    git-cliff -o CHANGELOG.md
