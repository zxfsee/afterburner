# ADR-043: Distributed Runtime Profile Schema

## Context

The repo now distinguishes:

- local training throughput (`training_scalability_contract.json`),
- deployment-side HTTP/load behavior (`distributed_load_profile.json`),
- and the current distributed runtime capability boundary.

What is still missing is one explicit artifact contract for future executed
distributed runtime profile trials. Without that contract, model-size and
node-count comparisons would drift into ad hoc benchmark outputs and make
parallelism choices hard to review or compare mechanically.

## Decision

Define one explicit distributed runtime profile artifact:

- `distributed_runtime_profile.schema.json`

The minimum profile contract records, per model-size and node-count cell:

- identity:
  - `artifact_version`
  - `model_size`
  - `node_count`
  - `gpus_per_node`
  - `world_size`
- runtime parallelism:
  - `dp`
  - `tp`
  - `pp`
  - `sp_cp`
  - `ep`
  - `gas`
  - `microbatch_size`
  - `zero_stage`
- performance and memory:
  - `mfu`
  - `step_time_ms`
  - `tokens_per_sec`
  - `max_memory_bytes`
- runtime settings:
  - `backend`
  - `weights_dtype`
  - `activation_dtype`
  - `quantization`
- environment fingerprint:
  - `system`
  - `gpu_model`
  - `burn_version`

Current stance: this ADR defines the artifact schema only. It does not claim
the repo can execute all listed parallelism modes today.

## Consequences

- Future distributed runtime trials get one stable artifact shape instead of ad
  hoc benchmark outputs.
- The profile schema stays separate from both the local throughput contract and
  the deployment-side load profile contract.
- Layout-feasibility and benchmark-harness work can reuse one explicit profile
  shape once those follow-on gates are implemented.
