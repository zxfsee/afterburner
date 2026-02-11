set shell := ["nu", "-c"]
set dotenv-load := true

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
    cargo run --locked --bin afterburner

# run inference using trained artifact
infer:
    cargo run --locked --bin infer

# run deterministic MNIST eval and write JSON summary
eval:
    cargo run --locked --bin eval -- --seed 42 --batch-size 128 --max-batches 8 --out artifacts/eval/mnist_eval_summary.json

# CI-friendly eval regression gate
eval-gate:
    just eval

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
    git-cliff -c Cargo.toml -o CHANGELOG.md
