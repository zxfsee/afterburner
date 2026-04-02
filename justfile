set shell := ["nu", "-c"]
set dotenv-load := true

profile-infer-flamegraph := "artifacts/profiling/infer_flamegraph.svg"
profile-infer-summary := "artifacts/profiling/infer_hotspot_summary.json"

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

# pin the current queue-maintenance objective before editing queue files
objective-lock-pin-queue:
    cargo run --locked --bin workflow_objective_lock -- pin --objective queue-only --expected-action queue-refresh

# pin the current backlog-maintenance objective before editing docs/backlog.md
objective-lock-pin-backlog:
    cargo run --locked --bin workflow_objective_lock -- pin --objective backlog-only --expected-action backlog-edit

# pin the current docs-only objective before editing docs/reference surfaces
objective-lock-pin-docs:
    cargo run --locked --bin workflow_objective_lock -- pin --objective docs-only --expected-action docs-edit

# pin the current review-only objective; any repo mutation will fail the guard
objective-lock-pin-review:
    cargo run --locked --bin workflow_objective_lock -- pin --objective review-only --expected-action review-pass

# validate queue freshness, then pin the top active TODO scope as the current execute objective
objective-lock-pin-execute-top-item:
    just queue-snapshot-check
    cargo run --locked --bin workflow_objective_lock -- pin --objective execute-top-item --cargo-toml Cargo.toml --expected-action execute-top-item

# verify current repo mutations against the active objective lock
objective-lock-check-worktree action:
    cargo run --locked --bin workflow_objective_lock -- check-worktree --action {{ action }}

# clear the current objective lock before switching objective classes
objective-lock-clear:
    cargo run --locked --bin workflow_objective_lock -- clear

# refresh the active queue through the canonical queue-only guarded path
queue-refresh:
    just objective-lock-pin-queue
    just changelog
    just queue-snapshot-check
    just objective-lock-clear

# start top-item execution through the canonical execute-side guarded path
queue-execute-preflight:
    just objective-lock-pin-execute-top-item
    just objective-lock-check-worktree execute-top-item

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
    cargo run --locked --bin afterburner -- deploy stack-check --target-profile fixtures/deployment_target_profile.example.json --stack-profile fixtures/deployment_stack_profile.example.json --out artifacts/deploy/deployment_stack_check.json
    nix eval .#checks.aarch64-darwin.deploy-activate.drvPath
    nix eval .#checks.aarch64-darwin.deploy-schema.drvPath

deploy-launch-plan target_profile stack_profile:
    cargo run --locked --bin afterburner -- deploy stack-launch-plan --target-profile {{ target_profile }} --stack-profile {{ stack_profile }} --out artifacts/deploy/deployment_stack_launch_plan.json

deploy-launch-receipt plan port launched_at_unix_ms:
    cargo run --locked --bin afterburner -- deploy stack-launch-receipt --plan {{ plan }} --port {{ port }} --launched-at-unix-ms {{ launched_at_unix_ms }} --out artifacts/deploy/deployment_stack_launch_receipt.json

deploy-launch-bundle receipt:
    cargo run --locked --bin afterburner -- deploy stack-launch-bundle --receipt {{ receipt }} --out artifacts/deploy/deployment_stack_launch_evidence_bundle.json

deploy-reconcile-launch-bundle receipt bundle:
    cargo run --locked --bin afterburner -- deploy reconcile-launch-bundle --receipt {{ receipt }} --bundle {{ bundle }} --out artifacts/deploy/deployment_stack_launch_evidence_bundle_reconciliation.json

deploy-record-launch-bundle-reconciliation-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- deploy record-launch-bundle-reconciliation-history --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_stack_launch_evidence_bundle_reconciliation_history.json

deploy-launch-handoff bundle:
    cargo run --locked --bin afterburner -- deploy stack-launch-handoff --bundle {{ bundle }} --out artifacts/deploy/deployment_stack_launch_evidence_handoff.json

deploy-reconcile-launch-handoff bundle handoff:
    cargo run --locked --bin afterburner -- deploy reconcile-launch-handoff --bundle {{ bundle }} --handoff {{ handoff }} --out artifacts/deploy/deployment_stack_launch_evidence_handoff_reconciliation.json

deploy-record-launch-handoff-history handoff event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- deploy record-launch-handoff-history --handoff {{ handoff }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_stack_launch_evidence_handoff_history.json

deploy-record-launch-handoff-reconciliation-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- deploy record-launch-handoff-reconciliation-history --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_stack_launch_evidence_handoff_reconciliation_history.json

deploy-point-launch-transport-locator handoff:
    cargo run --locked --bin afterburner -- deploy point-launch-transport-locator --handoff {{ handoff }} --out artifacts/deploy/deployment_stack_launch_transport_locator.json

deploy-record-launch-transport-locator-history locator event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- deploy record-launch-transport-locator-history --locator {{ locator }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_stack_launch_transport_locator_history.json

deploy-record-launch-transport-locator-reconciliation-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- deploy record-launch-transport-locator-reconciliation-history --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_stack_launch_transport_locator_reconciliation_history.json

deploy-reconcile-launch-transport-locator handoff locator:
    cargo run --locked --bin afterburner -- deploy reconcile-launch-transport-locator --handoff {{ handoff }} --locator {{ locator }} --out artifacts/deploy/deployment_stack_launch_transport_locator_reconciliation.json

deploy-point-launch-locator locator:
    cargo run --locked --bin afterburner -- deploy point-launch-locator --locator {{ locator }} --out artifacts/deploy/deployment_stack_launch_locator_pointer.json

deploy-reconcile-launch-locator plan pointer:
    cargo run --locked --bin afterburner -- deploy reconcile-launch-locator --plan {{ plan }} --pointer {{ pointer }} --out artifacts/deploy/deployment_stack_launch_locator_reconciliation.json

deploy-record-launch-locator-history pointer event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- deploy record-launch-locator-history --pointer {{ pointer }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_stack_launch_locator_history.json

deploy-record-launch-locator-reconciliation-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- deploy record-launch-locator-reconciliation-history --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_stack_launch_locator_reconciliation_history.json

# validate a candidate artifact before promotion
rollout-check candidate_artifact candidate_manifest ownership provider destination:
    cargo run --locked --bin afterburner -- eval --artifact {{ candidate_artifact }} --seed 42 --batch-size 128 --max-batches 8 --min-accuracy 0.98925781 --out artifacts/eval/mnist_eval_summary.json
    cargo run --locked --bin afterburner -- deploy upload --manifest {{ candidate_manifest }} --ownership {{ ownership }} --provider {{ provider }} --destination {{ destination }} --out artifacts/deploy/candidate_upload_request.json
    just deploy-check

huggingface-publish request:
    cargo run --locked --bin afterburner -- deploy hf-publish --request {{ request }}

kube-rs-lease-reconcile lease namespace resource_name:
    cargo run --locked --bin afterburner -- deploy kube-rs-lease-reconcile --lease {{ lease }} --namespace {{ namespace }} --resource-name {{ resource_name }} --out artifacts/deploy/kube_rs_gpu_lease_reconciliation.json

kube-rs-lease-point reconciliation:
    cargo run --locked --bin afterburner -- deploy point-kube-rs-lease --reconciliation {{ reconciliation }} --out artifacts/deploy/kube_rs_gpu_lease_pointer.json

kube-rs-lease-record-history pointer event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- deploy record-kube-rs-lease-history --pointer {{ pointer }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/kube_rs_gpu_lease_history.json

kube-rs-lease-record-reconciliation-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- deploy record-kube-rs-lease-reconciliation-history --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/kube_rs_gpu_lease_reconciliation_history.json

single-node-scheduler job inventory:
    cargo run --locked --bin afterburner -- deploy single-node-scheduler --job {{ job }} --inventory {{ inventory }} --out-lease artifacts/deploy/gpu_scheduler_lease.json --out-unit artifacts/deploy/afterburner-job.service

scheduler-heartbeat job_id lease_id worker_id state observed_at_unix_ms:
    cargo run --locked --bin afterburner -- deploy scheduler-heartbeat --job-id {{ job_id }} --lease-id {{ lease_id }} --worker-id {{ worker_id }} --state {{ state }} --observed-at-unix-ms {{ observed_at_unix_ms }} --out artifacts/deploy/gpu_scheduler_heartbeat.json

scheduler-heartbeat-point heartbeat:
    cargo run --locked --bin afterburner -- debug deploy point-scheduler-heartbeat --heartbeat {{ heartbeat }} --out artifacts/deploy/gpu_scheduler_heartbeat_pointer.json

scheduler-heartbeat-point-record-history pointer event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-scheduler-heartbeat-pointer-history --pointer {{ pointer }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/gpu_scheduler_heartbeat_pointer_history.json

scheduler-heartbeat-record-history heartbeat event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-scheduler-heartbeat-history --heartbeat {{ heartbeat }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/gpu_scheduler_heartbeat_history.json

scheduler-heartbeat-reconcile heartbeat current_heartbeat:
    cargo run --locked --bin afterburner -- debug deploy scheduler-heartbeat-reconcile --heartbeat {{ heartbeat }} --current-heartbeat {{ current_heartbeat }} --out artifacts/deploy/gpu_scheduler_heartbeat_reconciliation.json

scheduler-heartbeat-record-reconciliation-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-scheduler-heartbeat-reconciliation-history --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/gpu_scheduler_heartbeat_reconciliation_history.json

scheduler-heartbeat-supersede previous_heartbeat next_heartbeat superseded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy scheduler-heartbeat-supersede --previous-heartbeat {{ previous_heartbeat }} --next-heartbeat {{ next_heartbeat }} --superseded-at-unix-ms {{ superseded_at_unix_ms }} --out artifacts/deploy/gpu_scheduler_heartbeat_supersession.json

scheduler-heartbeat-supersession-reconcile supersession current_supersession:
    cargo run --locked --bin afterburner -- debug deploy scheduler-heartbeat-supersession-reconcile --supersession {{ supersession }} --current-supersession {{ current_supersession }} --out artifacts/deploy/gpu_scheduler_heartbeat_supersession_reconciliation.json

scheduler-heartbeat-record-supersession-reconciliation-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-scheduler-heartbeat-supersession-reconciliation-history --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/gpu_scheduler_heartbeat_supersession_reconciliation_history.json

scheduler-heartbeat-record-supersession-history supersession event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-scheduler-heartbeat-supersession-history --supersession {{ supersession }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/gpu_scheduler_heartbeat_supersession_history.json

distributed-load-profile addr requests concurrency latency_budget_ms_p99 error_budget_ratio:
    cargo run --locked --bin afterburner -- deploy load-profile --addr {{ addr }} --requests {{ requests }} --concurrency {{ concurrency }} --latency-budget-ms-p99 {{ latency_budget_ms_p99 }} --error-budget-ratio {{ error_budget_ratio }} --out artifacts/deploy/distributed_load_profile.json

# package receipt-referenced deployment verification evidence into one bundle artifact
deployment-verification-bundle receipt:
    cargo run --locked --bin afterburner -- verify bundle --receipt {{ receipt }} --out artifacts/deploy/deployment_verification_evidence_bundle.json

deployment-verification-record-bundle-history bundle event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-bundle-history --bundle {{ bundle }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_evidence_bundle_history.json

deployment-verification-point-bundle-transport-locator bundle:
    cargo run --locked --bin afterburner -- debug deploy point-verification-bundle-transport-locator --bundle {{ bundle }} --out artifacts/deploy/deployment_verification_evidence_bundle_transport_locator.json

deployment-verification-point-bundle-locator locator:
    cargo run --locked --bin afterburner -- debug deploy point-verification-bundle-locator --locator {{ locator }} --out artifacts/deploy/deployment_verification_evidence_bundle_locator_pointer.json

deployment-verification-record-bundle-locator-history pointer event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-bundle-locator-history --pointer {{ pointer }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_evidence_bundle_locator_pointer_history.json

deployment-verification-reconcile-bundle-locator locator pointer:
    cargo run --locked --bin afterburner -- debug deploy reconcile-verification-bundle-locator --locator {{ locator }} --pointer {{ pointer }} --out artifacts/deploy/deployment_verification_evidence_bundle_locator_pointer_reconciliation.json

deployment-verification-record-bundle-locator-reconciliation-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-bundle-locator-reconciliation-history --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_evidence_bundle_locator_pointer_reconciliation_history.json

deployment-verification-rollback-bundle-locator current_pointer restored_pointer rolled_back_at_unix_ms:
    cargo run --locked --bin afterburner -- rollback verification-bundle-locator --current-pointer {{ current_pointer }} --restored-pointer {{ restored_pointer }} --rolled-back-at-unix-ms {{ rolled_back_at_unix_ms }} --out-pointer artifacts/deploy/deployment_verification_evidence_bundle_locator_pointer.json --out-record artifacts/deploy/deployment_verification_evidence_bundle_locator_pointer_rollback.json

deployment-verification-record-bundle-locator-rollback-history rollback event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-bundle-locator-rollback-history --rollback {{ rollback }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_evidence_bundle_locator_pointer_rollback_history.json

deployment-verification-rollback-bundle current_bundle restored_bundle rolled_back_at_unix_ms:
    cargo run --locked --bin afterburner -- rollback verification-bundle --current-bundle {{ current_bundle }} --restored-bundle {{ restored_bundle }} --rolled-back-at-unix-ms {{ rolled_back_at_unix_ms }} --out-bundle artifacts/deploy/deployment_verification_evidence_bundle.json --out-record artifacts/deploy/deployment_verification_evidence_bundle_rollback.json

deployment-verification-record-bundle-rollback-history rollback event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-bundle-rollback-history --rollback {{ rollback }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_evidence_bundle_rollback_history.json

deployment-verification-reconcile-bundle-rollback rollback current_rollback:
    cargo run --locked --bin afterburner -- debug deploy reconcile-verification-bundle-rollback --rollback {{ rollback }} --current-rollback {{ current_rollback }} --out artifacts/deploy/deployment_verification_evidence_bundle_rollback_reconciliation.json

deployment-verification-record-bundle-rollback-reconciliation-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-bundle-rollback-reconciliation-history --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_evidence_bundle_rollback_reconciliation_history.json

deployment-verification-supersede-bundle-rollback previous_rollback next_rollback superseded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy supersede-verification-bundle-rollback --previous-rollback {{ previous_rollback }} --next-rollback {{ next_rollback }} --superseded-at-unix-ms {{ superseded_at_unix_ms }} --out artifacts/deploy/deployment_verification_evidence_bundle_rollback_supersession.json

deployment-verification-reconcile-bundle-rollback-supersession supersession current_supersession:
    cargo run --locked --bin afterburner -- debug deploy reconcile-verification-bundle-rollback-supersession --supersession {{ supersession }} --current-supersession {{ current_supersession }} --out artifacts/deploy/deployment_verification_evidence_bundle_rollback_supersession_reconciliation.json

deployment-verification-record-bundle-rollback-supersession-reconciliation-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-bundle-rollback-supersession-reconciliation-history --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_evidence_bundle_rollback_supersession_reconciliation_history.json

deployment-verification-record-bundle-rollback-supersession-history supersession event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-bundle-rollback-supersession-history --supersession {{ supersession }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_evidence_bundle_rollback_supersession_history.json

deployment-verification-record-bundle-transport-locator-history locator event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-bundle-transport-locator-history --locator {{ locator }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_evidence_bundle_transport_locator_history.json

deployment-verification-reconcile-bundle-transport-locator bundle locator:
    cargo run --locked --bin afterburner -- debug deploy reconcile-verification-bundle-transport-locator --bundle {{ bundle }} --locator {{ locator }} --out artifacts/deploy/deployment_verification_evidence_bundle_transport_locator_reconciliation.json

deployment-verification-record-bundle-transport-locator-reconciliation-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-bundle-transport-locator-reconciliation-history --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_evidence_bundle_transport_locator_reconciliation_history.json

deployment-verification-reconcile-bundle receipt bundle:
    cargo run --locked --bin afterburner -- debug deploy reconcile-verification-bundle --receipt {{ receipt }} --bundle {{ bundle }} --out artifacts/deploy/deployment_verification_evidence_bundle_reconciliation.json

deployment-verification-record-bundle-reconciliation-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-bundle-reconciliation-history --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_evidence_bundle_reconciliation_history.json

deployment-verification-handoff bundle:
    cargo run --locked --bin afterburner -- verify handoff --bundle {{ bundle }} --out artifacts/deploy/deployment_verification_evidence_handoff.json

deployment-verification-record-handoff-history handoff event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-handoff-history --handoff {{ handoff }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_evidence_handoff_history.json

deployment-verification-point-handoff-transport-locator handoff:
    cargo run --locked --bin afterburner -- debug deploy point-verification-handoff-transport-locator --handoff {{ handoff }} --out artifacts/deploy/deployment_verification_evidence_handoff_transport_locator.json

deployment-verification-record-handoff-transport-locator-history locator event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-handoff-transport-locator-history --locator {{ locator }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_evidence_handoff_transport_locator_history.json

deployment-verification-reconcile-handoff-transport-locator handoff locator:
    cargo run --locked --bin afterburner -- debug deploy reconcile-verification-handoff-transport-locator --handoff {{ handoff }} --locator {{ locator }} --out artifacts/deploy/deployment_verification_evidence_handoff_transport_locator_reconciliation.json

deployment-verification-reconcile-handoff bundle handoff:
    cargo run --locked --bin afterburner -- debug deploy reconcile-verification-handoff --bundle {{ bundle }} --handoff {{ handoff }} --out artifacts/deploy/deployment_verification_evidence_handoff_reconciliation.json

deployment-verification-record-handoff-reconciliation-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-handoff-reconciliation-history --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_evidence_handoff_reconciliation_history.json

# write one deployment verification receipt with explicit evidence-source references
deployment-verification-receipt artifact_version profile_name verification_status verified_at_unix_ms evidence evidence_source_1 evidence_source_2:
    cargo run --locked --bin afterburner -- verify receipt --artifact-version {{ artifact_version }} --profile-name {{ profile_name }} --verification-status {{ verification_status }} --verified-at-unix-ms {{ verified_at_unix_ms }} --evidence {{ evidence }} --evidence-source '{{ evidence_source_1 }}' --evidence-source '{{ evidence_source_2 }}' --out artifacts/deploy/deployment_verification_receipt.json

deployment-verification-point-receipt-transport-locator receipt:
    cargo run --locked --bin afterburner -- debug deploy point-verification-receipt-transport-locator --receipt {{ receipt }} --out artifacts/deploy/deployment_verification_receipt_transport_locator.json

deployment-verification-point-receipt-locator locator:
    cargo run --locked --bin afterburner -- debug deploy point-verification-receipt-locator --locator {{ locator }} --out artifacts/deploy/deployment_verification_receipt_locator_pointer.json

deployment-verification-record-receipt-locator-history pointer event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-receipt-locator-history --pointer {{ pointer }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_receipt_locator_pointer_history.json

deployment-verification-reconcile-receipt-locator locator pointer:
    cargo run --locked --bin afterburner -- debug deploy reconcile-verification-receipt-locator --locator {{ locator }} --pointer {{ pointer }} --out artifacts/deploy/deployment_verification_receipt_locator_pointer_reconciliation.json

deployment-verification-record-receipt-locator-reconciliation-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-receipt-locator-reconciliation-history --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_receipt_locator_pointer_reconciliation_history.json

deployment-verification-rollback-receipt-locator current_pointer restored_pointer rolled_back_at_unix_ms:
    cargo run --locked --bin afterburner -- rollback verification-receipt-locator --current-pointer {{ current_pointer }} --restored-pointer {{ restored_pointer }} --rolled-back-at-unix-ms {{ rolled_back_at_unix_ms }} --out-pointer artifacts/deploy/deployment_verification_receipt_locator_pointer.json --out-record artifacts/deploy/deployment_verification_receipt_locator_pointer_rollback.json

deployment-verification-record-receipt-locator-rollback-history rollback event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-receipt-locator-rollback-history --rollback {{ rollback }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_receipt_locator_pointer_rollback_history.json

deployment-verification-reconcile-receipt-rollback rollback current_rollback:
    cargo run --locked --bin afterburner -- debug deploy reconcile-verification-receipt-rollback --rollback {{ rollback }} --current-rollback {{ current_rollback }} --out artifacts/deploy/deployment_verification_receipt_rollback_reconciliation.json

deployment-verification-record-receipt-rollback-reconciliation-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-receipt-rollback-reconciliation-history --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_receipt_rollback_reconciliation_history.json

deployment-verification-supersede-receipt-rollback previous_rollback next_rollback superseded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy supersede-verification-receipt-rollback --previous-rollback {{ previous_rollback }} --next-rollback {{ next_rollback }} --superseded-at-unix-ms {{ superseded_at_unix_ms }} --out artifacts/deploy/deployment_verification_receipt_rollback_supersession.json

deployment-verification-reconcile-receipt-rollback-supersession supersession current_supersession:
    cargo run --locked --bin afterburner -- debug deploy reconcile-verification-receipt-rollback-supersession --supersession {{ supersession }} --current-supersession {{ current_supersession }} --out artifacts/deploy/deployment_verification_receipt_rollback_supersession_reconciliation.json

deployment-verification-record-receipt-rollback-supersession-reconciliation-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-receipt-rollback-supersession-reconciliation-history --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_receipt_rollback_supersession_reconciliation_history.json

deployment-verification-record-receipt-rollback-supersession-history supersession event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-receipt-rollback-supersession-history --supersession {{ supersession }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_receipt_rollback_supersession_history.json

deployment-verification-record-receipt-transport-locator-history locator event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-receipt-transport-locator-history --locator {{ locator }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_receipt_transport_locator_history.json

deployment-verification-reconcile-receipt-transport-locator receipt locator:
    cargo run --locked --bin afterburner -- debug deploy reconcile-verification-receipt-transport-locator --receipt {{ receipt }} --locator {{ locator }} --out artifacts/deploy/deployment_verification_receipt_transport_locator_reconciliation.json

deployment-verification-record-receipt-transport-locator-reconciliation-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-receipt-transport-locator-reconciliation-history --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_receipt_transport_locator_reconciliation_history.json

deployment-verification-record-receipt-history receipt event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-receipt-history --receipt {{ receipt }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_receipt_history.json

deployment-verification-reconcile-receipt receipt artifact_version profile_name verification_status verified_at_unix_ms evidence evidence_source_1 evidence_source_2:
    cargo run --locked --bin afterburner -- debug deploy reconcile-verification-receipt --receipt {{ receipt }} --artifact-version {{ artifact_version }} --profile-name {{ profile_name }} --verification-status {{ verification_status }} --verified-at-unix-ms {{ verified_at_unix_ms }} --evidence {{ evidence }} --evidence-source '{{ evidence_source_1 }}' --evidence-source '{{ evidence_source_2 }}' --out artifacts/deploy/deployment_verification_receipt_reconciliation.json

deployment-verification-record-receipt-reconciliation-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- debug deploy record-verification-receipt-reconciliation-history --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/deploy/deployment_verification_receipt_reconciliation_history.json

distributed-shard-lineage-receipt metadata shard_id source source_revision checkpoint_group checkpoint_root checked_at_unix_ms:
    cargo run --locked --bin afterburner -- lineage receipt --metadata {{ metadata }} --shard-id {{ shard_id }} --source {{ source }} --source-revision {{ source_revision }} --checkpoint-group {{ checkpoint_group }} --checkpoint-root {{ checkpoint_root }} --checked-at-unix-ms {{ checked_at_unix_ms }} --out artifacts/train/distributed_shard_lineage_receipt.json

distributed-shard-lineage-evidence-bundle receipt:
    cargo run --locked --bin afterburner -- lineage evidence-bundle --receipt {{ receipt }} --out artifacts/train/distributed_shard_lineage_evidence_bundle.json

distributed-shard-lineage-reconcile-evidence-bundle receipt bundle:
    cargo run --locked --bin afterburner -- lineage reconcile-evidence-bundle --receipt {{ receipt }} --bundle {{ bundle }} --out artifacts/train/distributed_shard_lineage_evidence_bundle_reconciliation.json

distributed-shard-lineage-record-evidence-bundle-reconciliation-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- lineage record-evidence-bundle-reconciliation-history --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/train/distributed_shard_lineage_evidence_bundle_reconciliation_history.json

distributed-shard-lineage-handoff bundle:
    cargo run --locked --bin afterburner -- lineage handoff --bundle {{ bundle }} --out artifacts/train/distributed_shard_lineage_evidence_handoff.json

distributed-shard-lineage-reconcile-handoff bundle handoff:
    cargo run --locked --bin afterburner -- lineage reconcile-handoff --bundle {{ bundle }} --handoff {{ handoff }} --out artifacts/train/distributed_shard_lineage_evidence_handoff_reconciliation.json

distributed-shard-lineage-record-handoff-history handoff event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- lineage record-handoff-history --handoff {{ handoff }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/train/distributed_shard_lineage_evidence_handoff_history.json

distributed-shard-lineage-record-handoff-reconciliation-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- lineage record-handoff-reconciliation-history --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/train/distributed_shard_lineage_evidence_handoff_reconciliation_history.json

distributed-shard-lineage-point-transport-locator handoff:
    cargo run --locked --bin afterburner -- lineage point-transport-locator --handoff {{ handoff }} --out artifacts/train/distributed_shard_lineage_transport_locator.json

distributed-shard-lineage-reconcile-transport-locator handoff locator:
    cargo run --locked --bin afterburner -- lineage reconcile-transport-locator --handoff {{ handoff }} --locator {{ locator }} --out artifacts/train/distributed_shard_lineage_transport_locator_reconciliation.json

distributed-shard-lineage-record-transport-locator-reconciliation-history reconciliation event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- lineage record-transport-locator-reconciliation-history --reconciliation {{ reconciliation }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/train/distributed_shard_lineage_transport_locator_reconciliation_history.json

distributed-shard-lineage-point-locator locator:
    cargo run --locked --bin afterburner -- lineage point-locator --locator {{ locator }} --out artifacts/train/distributed_shard_lineage_locator_pointer.json

distributed-shard-lineage-record-locator-history pointer event recorded_at_unix_ms:
    cargo run --locked --bin afterburner -- lineage record-locator-history --pointer {{ pointer }} --event {{ event }} --recorded-at-unix-ms {{ recorded_at_unix_ms }} --out artifacts/train/distributed_shard_lineage_locator_history.json

pretraining-source-approval-receipt source source_revision approval_status approved_by approval_ticket approved_at_unix_ms:
    cargo run --locked --bin afterburner -- source approval-receipt --source {{ source }} --source-revision {{ source_revision }} --approval-status {{ approval_status }} --approved-by {{ approved_by }} --approval-ticket {{ approval_ticket }} --approved-at-unix-ms {{ approved_at_unix_ms }} --out artifacts/data/pretraining_source_approval_receipt.json

pretraining-source-provenance-receipt approval_receipt registry_entry_path upstream_locator reviewed_metadata_sha256:
    cargo run --locked --bin afterburner -- source provenance-receipt --approval-receipt {{ approval_receipt }} --registry-entry-path {{ registry_entry_path }} --upstream-locator {{ upstream_locator }} --reviewed-metadata-sha256 {{ reviewed_metadata_sha256 }} --out artifacts/data/pretraining_source_provenance_receipt.json

pretraining-source-provenance-evidence-bundle provenance_receipt:
    cargo run --locked --bin afterburner -- source provenance-evidence-bundle --provenance-receipt {{ provenance_receipt }} --out artifacts/data/pretraining_source_provenance_evidence_bundle.json

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
    mkdir artifacts/profiling
    rm -rf cargo-flamegraph.trace
    let artifact_version = (open artifacts/inference/current | str trim)
    let artifact = $"artifacts/inference/($artifact_version)/model.mpk"
    let backend = (if ("BACKEND" in $env) { $env.BACKEND | str downcase } else { "wgpu" })
    let profile_command = $"env -u DEVELOPER_DIR -u SDKROOT XCTRACE=/usr/bin/xctrace cargo flamegraph --dev --deterministic --bin afterburner -o {{ profile-infer-flamegraph }} -- infer ($artifact)"
    env -u DEVELOPER_DIR -u SDKROOT XCTRACE=/usr/bin/xctrace cargo flamegraph --dev --deterministic --bin afterburner -o {{ profile-infer-flamegraph }} -- infer $artifact
    cargo run --locked --bin afterburner_profile_summary -- --input {{ profile-infer-flamegraph }} --output {{ profile-infer-summary }} --weights-artifact $artifact --backend $backend --profile-command $profile_command
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
    cargo run --locked --bin workflow_objective_lock -- check-paths --action queue-refresh --path CHANGELOG.md
    git-cliff -o CHANGELOG.md
    let parent = (jj log --ignore-working-copy -r @- --no-graph -T 'commit_id' | str trim); cargo run --locked --bin workflow_queue_snapshot -- stamp --cargo-toml Cargo.toml --changelog CHANGELOG.md --parent-commit $parent

queue-snapshot-check:
    let parent = (jj log --ignore-working-copy -r @- --no-graph -T 'commit_id' | str trim); let previous = (^jj log --ignore-working-copy -r @-- --no-graph -T 'commit_id' | complete); if $previous.exit_code == 0 { let previous_parent = ($previous.stdout | str trim); cargo run --locked --bin workflow_queue_snapshot -- verify --cargo-toml Cargo.toml --changelog CHANGELOG.md --parent-commit $parent --previous-parent-commit $previous_parent } else { cargo run --locked --bin workflow_queue_snapshot -- verify --cargo-toml Cargo.toml --changelog CHANGELOG.md --parent-commit $parent }

workflow-surface-check-deployment-verification:
    cargo nextest run --locked --test deployment_verification_workflow_surface
