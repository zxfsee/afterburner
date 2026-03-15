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
    nix eval .#checks.aarch64-darwin.deploy-activate.drvPath
    nix eval .#checks.aarch64-darwin.deploy-schema.drvPath

# capture a deterministic infer flamegraph into artifacts/profiling
# on macOS this requires `xcrun xctrace version` under full Xcode. The repo
# shell clears Nix Apple SDK overrides and forces `/usr/bin/xctrace` so the
# system Instruments templates win over the xcbuild wrapper environment.
profile-infer:
    mkdir artifacts/profiling
    rm -rf cargo-flamegraph.trace
    {{profile-infer-command}}
    cargo run --locked --bin afterburner_profile_summary -- --input {{profile-infer-flamegraph}} --output {{profile-infer-summary}} --weights-artifact {{profile-infer-artifact}} --backend {{profile-infer-backend}} --profile-command '{{profile-infer-command}}'

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
