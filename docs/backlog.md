# Backlog

Parked items live here until promoted into the active TODO queue.

## Direction

- Target system shape is a contract-first end-to-end MLOps pipeline: `data -> train -> eval -> promote -> deploy -> observe -> rollback`.
- This is a narrative model, not a priority rule. Active work is still driven by the runnable TODO queue and dependency unlocks.
- Promotion rule (default): promote the highest-priority runnable backlog item into the active TODO horizon.

Rules:
- Keep this list priority-ranked within backlog.
- Every item must declare metadata: `Goal`, `Scope`, `Contracts`, `Boundary`, `Kind`, and `Blocked-by` when applicable.
- `Blocked-by` should reference exact active TODO/backlog item titles (or stable IDs/pointers if available); include short inline context only when needed.
- Items must exist in exactly one place: active TODO queue or backlog (not both).
- Promotion into active TODO queue must preserve priority order and dependency constraints.

## Items

- Burn dependency refresh gate [Frameworks, Runtime Infra]
  - Goal: Keep the pinned Burn stack reasonably current and record any recorder/storage contract drift before it leaks into Afterburner artifact decisions by surprise.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `none`
  - Scope: `Cargo.toml`, `Cargo.lock`, `tests/`, `README.md`, `docs/adr/`

- Burn `.bpk` artifact migration contract [Frameworks, Runtime Infra]
  - Goal: Migrate the repo's inference artifact contract from `.mpk` to `.bpk` only after a pinned Burn refresh confirms the target APIs and the repo is ready to cut over docs, fixtures, CLI paths, and event payloads together.
  - Kind: `mixed`
  - Boundary: `core-contract`
  - Contracts: `artifact`, `cli`, `event`
  - Scope: `Cargo.toml`, `src/`, `fixtures/`, `tests/`, `README.md`, `ARCHITECTURE.md`, `docs/adr/`
  - Blocked-by: Burn dependency refresh gate.

- JSON to RON format fit investigation gate [Data Infra, Runtime Infra]
  - Goal: Evaluate whether any current JSON-based artifact or event surfaces should migrate to RON, and whether the readability or ergonomics gains would justify the contract churn and tooling impact.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `artifact`, `event`
  - Scope: `docs/adr/`, `README.md`, `tests/`

- CLI subcommand hierarchy migration contract [Runtime Infra, Serving/Deployment Infra]
  - Goal: Collapse the flat `afterburner` command namespace into grouped subcommands where it improves typical operator use, and cut over docs, tests, and workflow recipes in one explicit CLI contract change instead of accreting more top-level verbs.
  - Kind: `mixed`
  - Boundary: `adapter-cli`
  - Contracts: `cli`
  - Scope: `src/`, `tests/`, `README.md`, `justfile`, `docs/adr/`

- Remote model save/load fit gate [Serving/Deployment Infra, Runtime Infra]
  - Goal: Define whether Afterburner should support remote model artifact save/load beyond local filesystem paths, and if so, keep the contract adapter-first so artifact resolution and transfer do not couple core logic to one storage backend or SDK.
  - Kind: `gate`
  - Boundary: `adapter-deployment`
  - Contracts: `artifact`, `cli`, `ops`
  - Scope: `docs/adr/`, `README.md`, `tests/`, `src/`

- Burn distributed learning-strategy fit gate [Distributed Training, Frameworks]
  - Goal: Evaluate whether Afterburner should mirror Burn's distributed learning-strategy model instead of continuing to treat `worker_parallelism` as only a local throughput knob.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `none`
  - Scope: `docs/adr/`, `README.md`, `tests/`
  - Blocked-by: Burn dependency refresh gate.

- Distributed world and rank topology contract [Distributed Training, Runtime Infra]
  - Goal: Define explicit world-size, rank, and device-group metadata for future distributed training so execution topology is not inferred from local worker IDs or shard filenames alone.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `artifact`, `ops`
  - Scope: `docs/adr/`, `README.md`, `tests/`
  - Blocked-by: Burn dependency refresh gate.

- Collective synchronization boundary gate [Distributed Training, Frameworks]
  - Goal: Define where gradient synchronization and collective communication live relative to core training logic so future Burn collective adoption does not leak backend/runtime details across core boundaries.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `none`
  - Scope: `docs/adr/`, `ARCHITECTURE.md`, `tests/`
  - Blocked-by: Burn dependency refresh gate.

- Distributed optimizer and checkpoint state contract [Distributed Training, Runtime Infra]
  - Goal: Define the minimum optimizer-state and checkpoint-group contract needed for distributed recovery so future multi-device training can resume consistently across shards and ranks.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `artifact`
  - Scope: `docs/adr/`, `README.md`, `tests/`
  - Blocked-by: Burn dependency refresh gate.
