set shell := ["nu", "-c"]
set dotenv-load := true

profile-infer-flamegraph := "artifacts/profiling/infer_flamegraph.svg"
profile-infer-summary := "artifacts/profiling/infer_hotspot_summary.json"
afterburner-deploy := "cargo run --locked --bin afterburner -- deploy"
afterburner-debug-deploy := "cargo run --locked --bin afterburner -- debug deploy"
afterburner_repo_workflow := "cargo run --locked -p afterburner-repo-workflow --bin"

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

# list the canonical repo-managed workflow surface
workflows:
    just --list

# run the deterministic scheduler/runtime control-logic simulation harness
scheduler-runtime-simulate scenario:
    cargo run --locked --bin afterburner -- debug simulate-scheduler-runtime --scenario {{ scenario }} --out artifacts/simulation/scheduler_runtime_simulation_report.json

# inventory current `.mpk` and `.bpk` migration touchpoints across the repo
burn-bpk-migration-surface-report:
    cargo run --locked --bin afterburner -- debug inventory-burn-bpk-surface --repo-root . --out artifacts/report/burn_bpk_migration_surface_inventory.json

# pin the current queue-maintenance objective before editing queue files
objective-lock-pin-queue:
    {{ afterburner_repo_workflow }} workflow_objective_lock -- pin --objective queue-only --expected-action queue-refresh

# pin the top-scope-repair objective before fixing stale top TODO scope metadata
objective-lock-pin-top-scope-fix:
    {{ afterburner_repo_workflow }} workflow_objective_lock -- pin --objective top-scope-fix --expected-action top-scope-fix

# pin the current backlog-maintenance objective before editing docs/backlog.md
objective-lock-pin-backlog:
    {{ afterburner_repo_workflow }} workflow_objective_lock -- pin --objective backlog-only --expected-action backlog-edit

# pin the current docs-only objective before editing docs/reference surfaces
objective-lock-pin-docs:
    {{ afterburner_repo_workflow }} workflow_objective_lock -- pin --objective docs-only --expected-action docs-edit

# pin the current review-only objective; any repo mutation will fail the guard
objective-lock-pin-review:
    {{ afterburner_repo_workflow }} workflow_objective_lock -- pin --objective review-only --expected-action review-pass

# validate queue freshness, then pin the top active TODO scope as the current execute objective
objective-lock-pin-execute-top-item:
    {{ afterburner_repo_workflow }} workflow_queue_snapshot -- check-completion-boundary --cargo-toml Cargo.toml --repo-root .
    {{ afterburner_repo_workflow }} workflow_queue_snapshot -- verify-current-lineage --cargo-toml Cargo.toml --changelog CHANGELOG.md --repo-root .
    {{ afterburner_repo_workflow }} workflow_objective_lock -- pin --objective execute-top-item --cargo-toml Cargo.toml --expected-action execute-top-item

# verify current repo mutations against the active objective lock
objective-lock-check-worktree action:
    {{ afterburner_repo_workflow }} workflow_objective_lock -- check-worktree --action {{ action }}

# detect if the current top TODO appears landed without queue advancement
queue-completion-boundary-check:
    {{ afterburner_repo_workflow }} workflow_queue_snapshot -- check-completion-boundary --cargo-toml Cargo.toml --repo-root .

# fail if the current top active TODO is blocked and report the next runnable item
queue-top-runnable-check:
    {{ afterburner_repo_workflow }} workflow_queue_snapshot -- check-top-runnable --cargo-toml Cargo.toml

# clear the current objective lock before switching objective classes
objective-lock-clear:
    {{ afterburner_repo_workflow }} workflow_objective_lock -- clear

# remove an aged stale repo metadata lock through the typed helper path
repo-lock-repair stale_after_seconds='300':
    {{ afterburner_repo_workflow }} workflow_objective_lock -- repair-repo-locks --repo-root . --stale-after-seconds {{ stale_after_seconds }}

# fail fast if the repo still carries a live metadata lock
repo-lock-check:
    {{ afterburner_repo_workflow }} workflow_objective_lock -- check-repo-locks --repo-root .

# refresh the active queue through the canonical queue-only guarded path
queue-refresh: repo-lock-repair repo-lock-check objective-lock-pin-queue changelog queue-top-runnable-check queue-snapshot-check objective-lock-clear

# repair stale top TODO scope metadata through a dedicated queue-only path
queue-fix-top-scope: repo-lock-repair repo-lock-check objective-lock-pin-top-scope-fix queue-top-scope-worktree-check changelog-top-scope-fix queue-top-runnable-check queue-snapshot-check objective-lock-clear

queue-top-scope-worktree-check:
    {{ afterburner_repo_workflow }} workflow_objective_lock -- check-worktree --action top-scope-fix

# promote the next runnable active TODO above a blocked top item through a queue-only repair path
queue-promote-next-runnable: repo-lock-repair repo-lock-check objective-lock-pin-queue queue-promote-next-runnable-step changelog queue-top-runnable-check queue-snapshot-check objective-lock-clear

queue-promote-next-runnable-step:
    {{ afterburner_repo_workflow }} workflow_queue_snapshot -- promote-next-runnable --cargo-toml Cargo.toml

# start top-item execution through the canonical execute-side guarded path
queue-execute-preflight repair='':
    {{ afterburner_repo_workflow }} workflow_queue_snapshot -- execute-preflight --cargo-toml Cargo.toml --changelog CHANGELOG.md --repo-root . {{ repair }}

# explicitly repair stale queue snapshot lineage and resume execute preflight
queue-resume:
    {{ afterburner_repo_workflow }} workflow_queue_snapshot -- execute-preflight --cargo-toml Cargo.toml --changelog CHANGELOG.md --repo-root . --repair-stale-snapshot

# format rust + toml + nix
fmt:
    nix fmt

# train model and produce artifacts
train:
    cargo run --locked --bin afterburner -- train

train-text dataset_manifest tokenizer_profile token_cache:
    cargo run --locked --bin afterburner -- train --task text --dataset-manifest {{ dataset_manifest }} --tokenizer-profile {{ tokenizer_profile }} --token-cache {{ token_cache }}

train-text-smoke:
    cargo run --locked --bin afterburner -- train --task text --dataset-manifest fixtures/pretraining_dataset_manifest.example.json --tokenizer-profile fixtures/text_tokenizer_packing_profile.example.json --token-cache fixtures/text_token_cache.example.json --batch-size 2 --num-epochs 1 --max-validation-loss 4.0 --max-validation-perplexity 80.0

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
    {{ afterburner-deploy }} stack-check --target-profile fixtures/deployment_target_profile.example.json --stack-profile fixtures/deployment_stack_profile.example.json --out artifacts/deploy/deployment_stack_check.json
    nix eval .#checks.aarch64-darwin.deploy-activate.drvPath
    nix eval .#checks.aarch64-darwin.deploy-schema.drvPath

deploy-launch-plan target_profile stack_profile:
    {{ afterburner-debug-deploy }} launch plan --target-profile {{ target_profile }} --stack-profile {{ stack_profile }} --out artifacts/deploy/deployment_stack_launch_plan.json

deploy-launch-receipt plan port launched_at_unix_ms:
    {{ afterburner-debug-deploy }} launch receipt --plan {{ plan }} --port {{ port }} --launched-at-unix-ms {{ launched_at_unix_ms }} --out artifacts/deploy/deployment_stack_launch_receipt.json

deploy-launch-bundle receipt:
    {{ afterburner-debug-deploy }} launch bundle --receipt {{ receipt }} --out artifacts/deploy/deployment_stack_launch_evidence_bundle.json

deploy-launch-bundle-reconcile receipt bundle:
    {{ afterburner-debug-deploy }} launch bundle reconcile --receipt {{ receipt }} --bundle {{ bundle }} --out artifacts/deploy/deployment_stack_launch_evidence_bundle_reconciliation.json

deploy-launch-handoff bundle:
    {{ afterburner-debug-deploy }} launch handoff --bundle {{ bundle }} --out artifacts/deploy/deployment_stack_launch_evidence_handoff.json

deploy-launch-handoff-reconcile bundle handoff:
    {{ afterburner-debug-deploy }} launch handoff reconcile --bundle {{ bundle }} --handoff {{ handoff }} --out artifacts/deploy/deployment_stack_launch_evidence_handoff_reconciliation.json

# validate a candidate artifact before promotion
rollout-check candidate_artifact candidate_manifest ownership provider destination:
    cargo run --locked --bin afterburner -- deploy rollout-check --artifact {{ candidate_artifact }} --manifest {{ candidate_manifest }} --ownership {{ ownership }} --provider {{ provider }} --destination {{ destination }} --target-profile fixtures/deployment_target_profile.example.json --stack-profile fixtures/deployment_stack_profile.example.json --eval-out artifacts/eval/mnist_eval_summary.json --upload-out artifacts/deploy/candidate_upload_request.json --stack-check-out artifacts/deploy/deployment_stack_check.json --out-record artifacts/deploy/rollout_check.json

huggingface-publish request:
    {{ afterburner-deploy }} hf-publish --request {{ request }}

kube-rs-lease-reconcile lease namespace resource_name:
    {{ afterburner-debug-deploy }} lease reconcile --lease {{ lease }} --namespace {{ namespace }} --resource-name {{ resource_name }} --out artifacts/deploy/kube_rs_gpu_lease_reconciliation.json

single-node-scheduler job inventory:
    {{ afterburner-deploy }} single-node-scheduler --job {{ job }} --inventory {{ inventory }} --out-lease artifacts/deploy/gpu_scheduler_lease.json --out-unit artifacts/deploy/afterburner-job.service

scheduler-heartbeat job_id lease_id worker_id state observed_at_unix_ms:
    {{ afterburner-deploy }} scheduler-heartbeat --job-id {{ job_id }} --lease-id {{ lease_id }} --worker-id {{ worker_id }} --state {{ state }} --observed-at-unix-ms {{ observed_at_unix_ms }} --out artifacts/deploy/gpu_scheduler_heartbeat.json

distributed-load-profile addr requests concurrency latency_budget_ms_p99 error_budget_ratio:
    {{ afterburner-deploy }} load-profile --addr {{ addr }} --requests {{ requests }} --concurrency {{ concurrency }} --latency-budget-ms-p99 {{ latency_budget_ms_p99 }} --error-budget-ratio {{ error_budget_ratio }} --out artifacts/deploy/distributed_load_profile.json

# package receipt-referenced deployment verification evidence into one bundle artifact
deployment-verification-bundle receipt:
    cargo run --locked --bin afterburner -- verify bundle --receipt {{ receipt }} --out artifacts/deploy/deployment_verification_evidence_bundle.json

deployment-verification-bundle-reconcile receipt bundle:
    cargo run --locked --bin afterburner -- verify bundle reconcile --receipt {{ receipt }} --bundle {{ bundle }} --out artifacts/deploy/deployment_verification_evidence_bundle_reconciliation.json

deployment-verification-handoff bundle:
    cargo run --locked --bin afterburner -- verify handoff --bundle {{ bundle }} --out artifacts/deploy/deployment_verification_evidence_handoff.json

deployment-verification-handoff-reconcile bundle handoff:
    cargo run --locked --bin afterburner -- verify handoff reconcile --bundle {{ bundle }} --handoff {{ handoff }} --out artifacts/deploy/deployment_verification_evidence_handoff_reconciliation.json

# write one deployment verification receipt with explicit evidence-source references
deployment-verification-receipt artifact_version profile_name verification_status verified_at_unix_ms evidence evidence_source_1 evidence_source_2:
    cargo run --locked --bin afterburner -- verify receipt --artifact-version {{ artifact_version }} --profile-name {{ profile_name }} --verification-status {{ verification_status }} --verified-at-unix-ms {{ verified_at_unix_ms }} --evidence {{ evidence }} --evidence-source '{{ evidence_source_1 }}' --evidence-source '{{ evidence_source_2 }}' --out artifacts/deploy/deployment_verification_receipt.json

deployment-verification-receipt-reconcile receipt artifact_version profile_name verification_status verified_at_unix_ms evidence evidence_source_1 evidence_source_2 +evidence_sources:
    cargo run --locked --bin afterburner -- verify receipt reconcile --receipt {{ receipt }} --artifact-version {{ artifact_version }} --profile-name {{ profile_name }} --verification-status {{ verification_status }} --verified-at-unix-ms {{ verified_at_unix_ms }} --evidence {{ evidence }} --evidence-source '{{ evidence_source_1 }}' --evidence-source '{{ evidence_source_2 }}' {{ evidence_sources }} --out artifacts/deploy/deployment_verification_receipt_reconciliation.json

distributed-shard-lineage-receipt metadata shard_id source source_revision checkpoint_group checkpoint_root checked_at_unix_ms:
    cargo run --locked --bin afterburner -- debug lineage receipt --metadata {{ metadata }} --shard-id {{ shard_id }} --source {{ source }} --source-revision {{ source_revision }} --checkpoint-group {{ checkpoint_group }} --checkpoint-root {{ checkpoint_root }} --checked-at-unix-ms {{ checked_at_unix_ms }} --out artifacts/train/distributed_shard_lineage_receipt.json

distributed-shard-lineage-bundle receipt:
    cargo run --locked --bin afterburner -- debug lineage bundle --receipt {{ receipt }} --out artifacts/train/distributed_shard_lineage_evidence_bundle.json

distributed-shard-lineage-bundle-reconcile receipt bundle:
    cargo run --locked --bin afterburner -- debug lineage bundle reconcile --receipt {{ receipt }} --bundle {{ bundle }} --out artifacts/train/distributed_shard_lineage_evidence_bundle_reconciliation.json

distributed-shard-lineage-bundle-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug lineage bundle history --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/train/distributed_shard_lineage_evidence_bundle_reconciliation_history.json

distributed-shard-lineage-handoff bundle:
    cargo run --locked --bin afterburner -- debug lineage handoff --bundle {{ bundle }} --out artifacts/train/distributed_shard_lineage_evidence_handoff.json

distributed-shard-lineage-handoff-reconcile bundle handoff:
    cargo run --locked --bin afterburner -- debug lineage handoff reconcile --bundle {{ bundle }} --handoff {{ handoff }} --out artifacts/train/distributed_shard_lineage_evidence_handoff_reconciliation.json

distributed-shard-lineage-handoff-history handoff event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug lineage handoff history record --handoff {{ handoff }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/train/distributed_shard_lineage_evidence_handoff_history.json

distributed-shard-lineage-handoff-reconciliation-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug lineage handoff history reconciliation --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/train/distributed_shard_lineage_evidence_handoff_reconciliation_history.json

distributed-shard-lineage-locator-pointer locator:
    cargo run --locked --bin afterburner -- debug lineage locator point --locator {{ locator }} --out artifacts/train/distributed_shard_lineage_locator_pointer.json

distributed-shard-lineage-locator-history pointer event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug lineage locator history --pointer {{ pointer }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/train/distributed_shard_lineage_locator_history.json

distributed-shard-lineage-transport-locator handoff:
    cargo run --locked --bin afterburner -- debug lineage locator transport point --handoff {{ handoff }} --out artifacts/train/distributed_shard_lineage_transport_locator.json

distributed-shard-lineage-transport-locator-reconcile handoff locator:
    cargo run --locked --bin afterburner -- debug lineage locator transport reconcile --handoff {{ handoff }} --locator {{ locator }} --out artifacts/train/distributed_shard_lineage_transport_locator_reconciliation.json

distributed-shard-lineage-transport-locator-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug lineage locator transport history --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/train/distributed_shard_lineage_transport_locator_reconciliation_history.json

pretraining-source-approval source source_revision approval_status approved_by approval_ticket approved_at_unix_ms:
    cargo run --locked --bin afterburner -- source approval receipt --source {{ source }} --source-revision {{ source_revision }} --approval-status {{ approval_status }} --approved-by {{ approved_by }} --approval-ticket {{ approval_ticket }} --approved-at-unix-ms {{ approved_at_unix_ms }}

pretraining-source-provenance-receipt approval_receipt registry_entry_path upstream_locator reviewed_metadata_sha256:
    cargo run --locked --bin afterburner -- source provenance receipt --approval-receipt {{ approval_receipt }} --registry-entry-path {{ registry_entry_path }} --upstream-locator {{ upstream_locator }} --reviewed-metadata-sha256 {{ reviewed_metadata_sha256 }}

pretraining-source-provenance-evidence-bundle provenance_receipt:
    cargo run --locked --bin afterburner -- source provenance bundle --provenance-receipt {{ provenance_receipt }}

# inventory retained paths and clear prune candidates before planning cleanup
cleanup-inventory:
    cargo run --locked --bin afterburner -- cleanup inventory --artifacts-root artifacts --out artifacts/deploy/artifact_cleanup_inventory.json

# write one named cleanup policy profile for later dry-run and execution receipts
cleanup-policy profile:
    cargo run --locked --bin afterburner -- cleanup policy --profile {{ profile }} --out artifacts/deploy/artifact_cleanup_policy.json

# write a cleanup dry-run receipt from the current inventory and policy artifacts
cleanup-dry-run inventory policy generated_at_unix_ms:
    cargo run --locked --bin afterburner -- cleanup dry-run --inventory {{ inventory }} --policy {{ policy }} --generated-at-unix-ms {{ generated_at_unix_ms }} --out artifacts/deploy/artifact_cleanup_dry_run_receipt.json

# execute one approved cleanup dry-run receipt and write the execution audit artifact
cleanup-execute dry_run_receipt artifacts_root executed_at_unix_ms:
    cargo run --locked --bin afterburner -- cleanup execute --dry-run-receipt {{ dry_run_receipt }} --artifacts-root {{ artifacts_root }} --executed-at-unix-ms {{ executed_at_unix_ms }} --out artifacts/deploy/artifact_cleanup_execution_receipt.json

# package cleanup receipts and their referenced evidence into one bundle artifact
cleanup-evidence-bundle execution_receipt:
    cargo run --locked --bin afterburner -- cleanup evidence-bundle --execution-receipt {{ execution_receipt }} --out artifacts/deploy/artifact_cleanup_evidence_bundle.json

# compare candidate and current infer drift summaries using a named policy profile
drift-receipt candidate_summary current_summary policy:
    cargo run --locked --bin afterburner -- drift receipt --candidate {{ candidate_summary }} --current {{ current_summary }} --policy {{ policy }} --out artifacts/eval/infer_output_drift_receipt.json

# capture a checked baseline snapshot from an approved infer drift receipt
drift-baseline summary receipt:
    cargo run --locked --bin afterburner -- drift baseline --summary {{ summary }} --receipt {{ receipt }} --out artifacts/eval/infer_output_drift_baseline.json

# approve a baseline snapshot for rollout decisions
drift-approve-baseline baseline approved_by approval_ticket approved_at_unix_ms:
    cargo run --locked --bin afterburner -- drift approve-baseline --baseline {{ baseline }} --approved-by {{ approved_by }} --approval-ticket {{ approval_ticket }} --approved-at-unix-ms {{ approved_at_unix_ms }} --out artifacts/eval/infer_output_drift_baseline_approval.json

# refresh the approved drift baseline, archiving the old one and writing a refresh receipt
drift-refresh-baseline summary receipt current_baseline:
    cargo run --locked --bin afterburner -- drift refresh-baseline --summary {{ summary }} --receipt {{ receipt }} --current-baseline {{ current_baseline }} --out artifacts/eval/infer_output_drift_baseline.json --archive artifacts/eval/infer_output_drift_baseline.previous.json --refresh-receipt artifacts/eval/infer_output_drift_baseline_refresh.json

# promote a vetted artifact version by updating the current pointer and saving the previous one
rollout-promote artifact_version:
    cargo run --locked --bin afterburner -- deploy promote-current --artifact-version {{ artifact_version }} --previous-version-file artifacts/deploy/previous_current_version.txt --out-record artifacts/deploy/inference_current_pointer_promotion.json

# verify the promoted current pointer still resolves and passes the eval gate
rollout-verify candidate_artifact:
    cargo run --locked --bin afterburner -- verify rollout --artifact {{ candidate_artifact }} --eval-out artifacts/eval/mnist_eval_summary.json --out-record artifacts/deploy/rollout_verify.json

# restore the previous current pointer after a failed rollout or explicit rollback
rollout-rollback:
    cargo run --locked --bin afterburner -- rollback current-pointer --previous-version-file artifacts/deploy/previous_current_version.txt --out-record artifacts/deploy/inference_current_pointer_rollback.json

# capture a deterministic infer flamegraph into artifacts/profiling
# on macOS this requires `xcrun xctrace version` under full Xcode. The repo
# shell clears Nix Apple SDK overrides and forces `/usr/bin/xctrace` so the

# system Instruments templates win over the xcbuild wrapper environment.
profile-environment-snapshot:
    cargo run --locked --bin afterburner -- profile environment-snapshot --profile-kind infer --profiler cargo-flamegraph --profiler-path cargo-flamegraph --out artifacts/profiling/profiling_environment_snapshot.json

profile-refresh-environment-snapshot current_snapshot profiler_path captured_at_unix_ms:
    cargo run --locked --bin afterburner -- profile refresh-environment-snapshot --current-snapshot {{ current_snapshot }} --profiler-path {{ profiler_path }} --captured-at-unix-ms {{ captured_at_unix_ms }} --out artifacts/profiling/profiling_environment_snapshot.json --refresh-receipt artifacts/profiling/profiling_environment_snapshot_refresh.json

profile-provenance-receipt snapshot captured_at_unix_ms:
    cargo run --locked --bin afterburner -- profile provenance-receipt --snapshot {{ snapshot }} --captured-at-unix-ms {{ captured_at_unix_ms }} --out artifacts/profiling/profiling_provenance_receipt.json

profile-provenance-bundle snapshot summary captured_at_unix_ms:
    cargo run --locked --bin afterburner -- profile provenance-bundle --snapshot {{ snapshot }} --summary {{ summary }} --captured-at-unix-ms {{ captured_at_unix_ms }} --out artifacts/profiling/profiling_provenance_evidence_bundle.json

distributed-runtime-benchmark artifact_version model_size node_count gpus_per_node world_size dp tp pp sp_cp ep gas microbatch_size zero_stage backend system gpu_model burn_version seed measurement_window_steps benchmark_command step_time_ms tokens_per_sec mfu max_memory_bytes:
    cargo run --locked --bin afterburner -- profile distributed-runtime-benchmark --artifact-version {{ artifact_version }} --model-size {{ model_size }} --node-count {{ node_count }} --gpus-per-node {{ gpus_per_node }} --world-size {{ world_size }} --dp {{ dp }} --tp {{ tp }} --pp {{ pp }} --sp-cp {{ sp_cp }} --ep {{ ep }} --gas {{ gas }} --microbatch-size {{ microbatch_size }} --zero-stage {{ zero_stage }} --backend {{ backend }} --system {{ system }} --gpu-model {{ gpu_model }} --burn-version {{ burn_version }} --seed {{ seed }} --measurement-window-steps {{ measurement_window_steps }} --benchmark-command '{{ benchmark_command }}' --step-time-ms {{ step_time_ms }} --tokens-per-sec {{ tokens_per_sec }} --mfu {{ mfu }} --max-memory-bytes {{ max_memory_bytes }} --out artifacts/train/distributed_runtime_benchmark_run.json

distributed-runtime-profile benchmark_run:
    cargo run --locked --bin afterburner -- profile distributed-runtime-profile --benchmark-run {{ benchmark_run }} --out artifacts/train/distributed_runtime_profile.json

optimized-model-local-profile package_contract quality_metric quality_value minimum_quality_value observed_latency_ms_p99 observed_max_memory_bytes observed_package_bytes:
    cargo run --locked --bin afterburner -- profile optimized-model-local-profile --package-contract {{ package_contract }} --quality-metric {{ quality_metric }} --quality-value {{ quality_value }} --minimum-quality-value {{ minimum_quality_value }} --observed-latency-ms-p99 {{ observed_latency_ms_p99 }} --observed-max-memory-bytes {{ observed_max_memory_bytes }} --observed-package-bytes {{ observed_package_bytes }} --out artifacts/eval/optimized_model_local_profile.json

profile-infer:
    rm -rf cargo-flamegraph.trace
    cargo run --locked --bin afterburner -- profile infer --flamegraph-out {{ profile-infer-flamegraph }} --summary-out {{ profile-infer-summary }} --snapshot-out artifacts/profiling/profiling_environment_snapshot.json --profiler-path cargo-flamegraph

# open the terminal dashboard with profiling summary context
dashboard:
    cargo run --locked --bin afterburner-dashboard -- --input artifacts/train/observability.jsonl --profiling-summary artifacts/profiling/infer_hotspot_summary.json

# validate workspace/core dependency boundaries
workspace-gate:
    cargo test --test repo_structure_catalog
    cargo test -p afterburner-core

# run training by default
run: train

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

# fast compile-only smoke for the root package integration tests
test-root-compile-smoke:
    cargo test -p afterburner --tests --locked --no-run --quiet

# fast compile-only smoke for repo workflow tooling after crate-local helper changes
test-repo-workflow-compile-smoke:
    cargo test -p afterburner-repo-workflow --tests --locked --no-run --quiet

# fast compile-only smoke for integration tests after broad helper/layout changes
test-compile-smoke: test-root-compile-smoke test-repo-workflow-compile-smoke

# grouped docs/reference validators
test-doc-gates:
    cargo nextest run --locked --test doc_test_admission --test readme_reference_split --test reference_doc_validation --test workflow_reference

# grouped queue/workflow guards
test-repo-workflow-gates:
    cargo nextest run --locked -p afterburner-repo-workflow --test objective_lock --test queue_snapshot

# grouped queue/workflow guards
test-queue-gates: test-repo-workflow-gates
    cargo nextest run --locked -p afterburner --test developer_workflows

# grouped deploy/workflow-family guards
test-deploy-family:
    cargo nextest run --locked --test deployment_verification_workflow_surface_catalog --test deployment_surface_catalog --test distributed_load_profile

# grouped drift/workflow-family guards
test-drift-family:
    cargo nextest run --locked --test workflow_family_surface_catalog --test infer_drift_receipt --test infer_drift_baseline --test infer_drift_baseline_approval --test infer_drift_baseline_refresh

# regenerate CHANGELOG.md from git history
changelog:
    {{ afterburner_repo_workflow }} workflow_objective_lock -- check-paths --action queue-refresh --path CHANGELOG.md
    git-cliff -o CHANGELOG.md
    {{ afterburner_repo_workflow }} workflow_queue_snapshot -- stamp-current-parent --cargo-toml Cargo.toml --changelog CHANGELOG.md --repo-root .

changelog-top-scope-fix:
    {{ afterburner_repo_workflow }} workflow_objective_lock -- check-paths --action top-scope-fix --path CHANGELOG.md
    git-cliff -o CHANGELOG.md
    {{ afterburner_repo_workflow }} workflow_queue_snapshot -- stamp-current-parent --cargo-toml Cargo.toml --changelog CHANGELOG.md --repo-root .

queue-snapshot-check:
    {{ afterburner_repo_workflow }} workflow_queue_snapshot -- verify-current-lineage --cargo-toml Cargo.toml --changelog CHANGELOG.md --repo-root .

workflow-surface-check-deployment-verification:
    cargo nextest run --locked --test deployment_verification_workflow_surface_catalog

workflow-surface-check-routing-orchestration:
    cargo nextest run --locked --test rollout_orchestration_catalog --test routing_orchestration_semantic_regression --test cli_dispatch_decomposition

workflow-surface-check-scheduler-heartbeat:
    cargo nextest run --locked --test workflow_family_surface_catalog

workflow-surface-check-distributed-shard-lineage:
    cargo nextest run --locked --test workflow_family_surface_catalog

workflow-surface-check-pretraining-source:
    cargo nextest run --locked --test workflow_family_surface_catalog

workflow-surface-check-drift:
    cargo nextest run --locked --test workflow_family_surface_catalog

workflow-surface-check-cleanup:
    cargo nextest run --locked --test workflow_family_surface_catalog

workflow-surface-check-profiling:
    cargo nextest run --locked --test workflow_family_surface_catalog

workflow-surface-check-deployment-stack:
    cargo nextest run --locked --test deployment_surface_catalog

workflow-surface-check-deployment-utility:
    cargo nextest run --locked --test deployment_surface_catalog

workflow-surface-check-deployment-matrix:
    cargo nextest run --locked --test deployment_surface_catalog

workflow-surface-check-public-cli:
    cargo nextest run --locked --test cli_surface_catalog

workflow-surface-check-operator-grammar:
    cargo nextest run --locked --test cli_surface_catalog

workflow-surface-check-justfile-thinness:
    cargo nextest run --locked --test justfile_thinness_gate

workflow-surface-check-reference-docs:
    cargo nextest run --locked --test reference_doc_validation
