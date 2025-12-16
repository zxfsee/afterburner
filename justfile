set shell := ["nu", "-c"]
set dotenv-load := true

# build the project via nix (reproducible)
build:
	nix build

# run all checks (fmt, clippy, audit, tests)
check:
	nix flake check

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
	cargo run --locked --bin infer -- artifacts/inference/model.mpk

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
