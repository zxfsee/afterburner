# Afterburner

## What this is
Minimal Rust-based ML system demonstrating training, inference,
and reproducible infrastructure using Burn and Nix.

## Design goals
- Reproducibility over convenience
- Explicit boundaries between training and inference
- Inspectable artifacts
- Minimal but intentional infrastructure

## Training
How training works, where artifacts go.

## Inference
How inference is invoked and why CLI-first.

Training and inference are exposed as separate binaries to make system boundaries explicit.

## Infrastructure choices
Why Rust, Burn, Nix.
What trade-offs were made.
