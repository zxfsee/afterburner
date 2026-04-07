# Changelog

## TODO

- Distinct-device execution proof for single-node DP path [Distributed Training, Runtime Infra]
  - Goal: Run one real single-node data-parallel training execution on a host or runner with at least 2 concrete adapters and capture proof evidence that multiple distinct participants actually execute under the guarded DP runtime path.
  - Kind: `mixed`
  - Boundary: `distributed-runtime`
  - Contracts: `cli`, `artifact`, `event`
  - Scope: `src/cmd_train.rs`, `src/train/runtime.rs`, `src/train/distributed_metadata.rs`, `tests/distributed_runtime_execution_contract.rs`, `tests/distributed_runtime_fit_catalog.rs`, `docs/reference.md`, `ARCHITECTURE.md`
  - Blocked-by: Host or runner with >=2 concrete adapters for one distinct-device DP execution proof

- DP checkpoint and optimizer resume path [Distributed Training, Runtime Infra]
  - Goal: Make `checkpoint_group`-anchored checkpoint and optimizer resume executable for the first DP runtime path instead of leaving recovery at the contract-only layer.
  - Kind: `mixed`
  - Boundary: `distributed-runtime`
  - Contracts: `artifact`, `event`
  - Scope: `src/train/runtime.rs`, `src/train/artifacts.rs`, `docs/adr/061-distributed-optimizer-and-checkpoint-state.md`, `docs/reference.md`, `ARCHITECTURE.md`, `tests/contract_anchor_catalog.rs`

- Executed DP benchmark and profile artifact [Distributed Training, Experimentation/Eval Infra]
  - Goal: Run one executed single-node DP benchmark from the train path and synthesize both `distributed_runtime_benchmark_run.json` and one measured `distributed_runtime_profile.json` cell from that real run instead of synthetic inputs or schema-only normalization.
  - Kind: `mixed`
  - Boundary: `distributed-runtime`
  - Contracts: `cli`, `artifact`, `event`
  - Scope: `src/cmd_distributed_runtime_benchmark.rs`, `src/cmd_distributed_runtime_profile.rs`, `src/cmd_train.rs`, `tests/distributed_runtime_benchmark_harness.rs`, `tests/distributed_runtime_profile_schema.rs`, `docs/adr/058-distributed-runtime-benchmark-harness.md`, `docs/adr/043-distributed-runtime-profile-schema.md`, `docs/reference.md`
  - Blocked-by: Distinct-device execution proof for single-node DP path

- Scheduler/runtime rank assignment handoff [Distributed Training, Runtime Infra]
  - Goal: Feed explicit `world_size`, `rank_assignments`, and `device_group` semantics from the scheduler/control-plane boundary into the first DP runtime path without widening scheduler policy.
  - Kind: `mixed`
  - Boundary: `runtime-scheduler`
  - Contracts: `artifact`, `event`
  - Scope: `src/cmd_single_node_scheduler.rs`, `src/cmd_scheduler_runtime_simulation.rs`, `tests/gpu_scheduler_lifecycle_message.rs`, `tests/single_node_gpu_scheduler.rs`, `docs/reference.md`, `ARCHITECTURE.md`

- Text DP smoke vertical [Pre-training, Distributed Training]
  - Goal: Exercise the bounded text training path through the first DP runtime slice and keep artifact, eval, and inference outputs valid on a non-MNIST workload.
  - Kind: `mixed`
  - Boundary: `distributed-runtime`
  - Contracts: `cli`, `artifact`, `event`
  - Scope: `src/text_pretrain.rs`, `src/cmd_train.rs`, `tests/text_pretraining_adapter.rs`, `tests/text_pretraining_smoke.rs`, `README.md`, `docs/reference.md`
  - Blocked-by: Distinct-device execution proof for single-node DP path

- Text model deploy and verification vertical [Pre-training, Serving/Deployment Infra]
  - Goal: Run one bounded text model from training artifact through inference, HTTP serving, rollout verification, and deployment-facing evidence so the text path is demonstrable end to end after the DP runtime slices land.
  - Kind: `mixed`
  - Boundary: `runtime-deploy`
  - Contracts: `cli`, `artifact`, `event`, `http`
  - Scope: `src/text_pretrain.rs`, `src/cmd_train.rs`, `src/cmd_infer.rs`, `src/bin/afterburner_http.rs`, `tests/text_pretraining_adapter.rs`, `tests/text_pretraining_smoke.rs`, `tests/http_graceful_shutdown.rs`, `tests/deployment_verification_workflow_surface_catalog.rs`, `README.md`, `docs/reference.md`
  - Blocked-by: Text DP smoke vertical

<!-- queue-snapshot: todo_sha256=dbda51f491908321a8a920226c2c1743de0fb8eb68ce7b13e676ab54569cd6ac parent_commit=262f68e5ee048fe2b9e1e7fdd708c00c26b83689 -->

## [Trunk]

### Added

- Enable Rust and mold in devenv.nix ([94e3119])
- Add Model/ModelConfig and initialize & print model in main ([8a19388])
- Add burn-autodiff integration and training pipeline ([2535151])
- Add inference CLI and move modules to library; bump Cargo.lock ([ac473da])
- Load inference artifact from artifacts/inference/model.mpk and use explicit softmax ([d21a891])
- Set artifacts/train default, copy model to artifacts/inference, rename package to afterburner ([97ef620])
- Load inference contract, validate artifact, honor BACKEND env ([e36bcda])
- Centralize inference artifact path handling and CLI arg parsing ([6050ad6])
- Emit structured stderr events and fail gracefully on missing artifact ([3445782])
- Emit minimal structured JSON events on stderr; add json_escape and update tests/docs/.gitignore ([13ffa8e])
- Harden inference artifact contract ([7f3a4ef])
- Add SHA-256 checksum to artifact manifest and verify integrity ([ae91a99])
- Add minimal HTTP inference wrapper ([b981d60])
- Version inference artifacts ([be3bdce])
- Add observability events ([4506a9a])
- Add deterministic mnist eval regression guard ([438b0e4])
- Add bounded batch inference envelope ([8a1ad14])
- Harden shutdown and request tracing ([719ca73])
- Normalize event schema and add JSONL sink ([5bb4362])
- Add signing-ready placeholders and canonicalization metadata ([6cad4fe])
- Add deterministic accuracy threshold gate ([641e796])
- Add HTTP overload fixture gate ([c0e779b])
- Validate signed manifest digest input ([e70afe8])
- Implement changelog queue contract and adapter gates ([f8f847a])
- Harden metadata contract validation ([30b6aab])
- Enforce backend parity and shard metadata load validation ([9fc29da])
- Add fixture-backed eval/infer/http drift gates ([1cbcdd7])
- Add eval artifact-version parity and fixture gates ([25572b8])
- Define monitoring contract ([e283856])
- Add scalability readiness contract ([5b80b82])
- Harden pretraining metadata contract ([6495bbe])
- Add observability dashboard adapter ([cb0efd1])
- Align infer eval monitoring semantics ([09f053a])
- Align infer event semantics ([183bbd5])
- Split workspace and add dependency gate ([277391f])
- Split workspace and add dependency gate ([5f70ebf])
- Define custom kernel adoption contract\n\nWrite a training-side kernel threshold artifact, gate it against the current model footprint, and document the decision in ADR-005. ([df702e0])
- Advance contract gates and profiling tooling ([53fa3ab])
- Validate calibration sidecar compatibility ([18bc718])
- Add infer hotspot summary artifact ([e3b73e1])
- Surface precision metadata in events ([2f55936])
- Add deploy-rs baseline contract ([d6fb65d])
- Add deploy-rs baseline contract ([8e6c185])
- Unify profiling in dashboard ([6c42dc3])
- Add artifact upload request adapter ([a02470d])
- Export calibration sidecar contract ([66269c9])
- Add pretraining dataset manifest contract ([3619d37])
- Add pretraining dataset manifest contract ([6f93b97])
- Add promotion rollback orchestration ([b30bcb0])
- Add output drift summary contract ([01d23a9])
- Add drift comparison receipt ([96e45ae])
- Add drift baseline snapshot ([71e819e])
- Add drift policy profile contract ([26f24f0])
- Add drift baseline refresh contract ([c3b911e])
- Add drift baseline refresh contract ([d130225])
- Add drift baseline approval contract ([a828c2f])
- Add baseline approval supersession ([d37aeed])
- Add approved baseline pointer ([9baf390])
- Add approved baseline rollback contract ([91dec15])
- Add approved baseline history ([6bbeb18])
- Add baseline checkpoint contract ([4cfb1f9])
- Add baseline export bundle ([7df36b3])
- Add baseline handoff manifest ([d11f075])
- Add drift baseline transport locator ([dbb098f])
- Add artifact cleanup inventory ([f44b6ce])
- Add artifact cleanup policy profile ([47ebf56])
- Add profiling environment snapshot ([ac3cbc5])
- Add deployment verification evidence bundle ([984238a])
- Add profiling environment snapshot refresh receipt ([011b945])
- Add distributed load profile ([d96a0a7])
- Group operator subcommands ([7bf677b])
- Define deployment stack contract ([a4b7835])
- Add tracing correlation contract ([5000595])
- Expose metal runtime option ([583a515])
- Group deployment commands ([563476b])
- Add single-node lease adapter ([007426e])
- Add hugging face publish adapter ([9eaf953])
- Add benchmark harness ([0661586])
- Add runtime profile command ([9d787dc])
- Add macbook text adapter ([9f8e082])
- Add text smoke eval gate ([31cc68e])
- Add dry-run receipt ([11a4225])
- Add execution receipt adapter ([792f803])
- Add execution provenance receipt ([91325b2])
- Add evidence bundle adapter ([49cfe2f])
- Add provenance bundle adapter ([99144c7])
- Add provenance receipt adapter ([a5defc4])
- Add verification receipt adapter ([c4742ee])
- Add shard lineage receipt adapter ([23bac29])
- Add shard lineage evidence bundle adapter ([1654689])
- Add source approval receipt adapter ([392e980])
- Add source provenance receipt adapter ([eafc870])
- Add source provenance evidence bundle ([3d38394])
- Add verification handoff adapter ([1994c27])
- Add stack launch plan adapter ([8d7c2b9])
- Add shard lineage handoff adapter ([d298d6d])
- Add stack launch receipt adapter ([4b2be9e])
- Cut over recipe to current pointer ([c2ceccf])
- Add stack launch evidence bundle adapter ([5e9bc18])
- Add stack launch evidence handoff adapter ([129e859])
- Add stack launch transport locator ([c932cb3])
- Add stack launch locator pointer ([485fbe1])
- Add transport locator adapter ([bfc546e])
- Add kube-rs lease reconciliation adapter ([0e6a59a])
- Add launch locator history adapter ([06fb9e4])
- Add locator pointer adapter ([d8ce479])
- Add kube-rs lease pointer adapter ([980f670])
- Add locator history adapter ([be1ae26])
- Add kube-rs lease history adapter ([344d77a])
- Add launch locator reconciliation adapter ([e693670])
- Add launch handoff history adapter ([c673585])
- Add handoff history adapter ([ee4732b])
- Add launch locator reconciliation history adapter ([60b6fdc])
- Add kube-rs lease reconciliation history adapter ([1bd69b3])
- Add launch transport locator history adapter ([12f5f55])
- Add launch transport locator reconciliation adapter ([4080a84])
- Add launch handoff reconciliation adapter ([2f15119])
- Add handoff reconciliation adapter ([3fec302])
- Add launch handoff reconciliation history adapter ([abea151])
- Add launch transport locator reconciliation history adapter ([5db0739])
- Add handoff reconciliation history adapter ([4a29fa9])
- Add transport locator reconciliation adapter ([c37819b])
- Add launch evidence bundle reconciliation adapter ([8fee285])
- Add evidence bundle reconciliation adapter ([9b3a4fa])
- Add transport locator reconciliation history adapter ([e5dd680])
- Add launch evidence bundle reconciliation history adapter ([26bb73c])
- Add evidence bundle reconciliation history adapter ([8ad8913])
- Add queue snapshot invalidation guard ([59dc9a5])
- Add objective lock guard ([60b0aad])
- Add verification bundle reconciliation adapter ([65ad9f1])
- Add verification handoff reconciliation adapter ([4e148ff])
- Add canonical queue refresh helper ([2343f36])
- Add canonical queue execute preflight ([9850781])
- Align helper tooling with workflow boundaries ([ed6b43f])
- Complete targeted harness hardening ([0d15881])
- Add verification handoff reconciliation history adapter ([fbeb94f])
- Add verification bundle reconciliation history adapter ([bc4a279])
- Add verification receipt reconciliation adapter ([50a60bb])
- Add verification receipt reconciliation history adapter ([050cf2c])
- Add verification handoff history adapter ([59f40cb])
- Add verification receipt history adapter ([20f62ad])
- Add verification bundle history adapter ([c04fd1a])
- Add verification handoff transport locator adapter ([2401197])
- Add verification handoff transport locator reconciliation adapter ([c04eb70])
- Add verification handoff transport locator reconciliation adapter ([a5b4077])
- Add verification handoff transport locator history adapter ([f928ace])
- Add verification bundle transport locator adapter ([40e195b])
- Add verification bundle transport locator reconciliation adapter ([9630cf3])
- Add verification bundle transport locator history adapter ([d314992])
- Add verification receipt transport locator adapter ([3697ef0])
- Add verification receipt transport locator reconciliation adapter ([7171d6e])
- Add receipt transport locator history ([9d52b39])
- Add bundle transport locator reconciliation history ([98f8645])
- Add receipt transport locator reconciliation history ([3a2c802])
- Add bundle locator pointer ([bf3ef7a])
- Add receipt locator pointer ([bfb22c4])
- Add bundle locator reconciliation ([23e165c])
- Add receipt locator reconciliation ([c7de99d])
- Add bundle locator history ([f2cba27])
- Add receipt locator history ([d6690bb])
- Add bundle locator reconciliation history ([4eb3d60])
- Add receipt locator reconciliation history ([29a1191])
- Add bundle locator rollback ([2747d3f])
- Add receipt locator rollback ([ceb3be4])
- Add bundle locator rollback history ([2c0aec5])
- Add receipt locator rollback history ([68ea9d9])
- Add bundle rollback adapter ([1a2d92e])
- Add bundle rollback history ([314da90])
- Add bundle rollback reconciliation ([fca8951])
- Add receipt rollback reconciliation ([d1f1d81])
- Add bundle rollback reconciliation history ([89218b5])
- Add receipt rollback reconciliation history ([569854e])
- Add bundle rollback supersession ([1e98527])
- Add receipt rollback supersession ([f7b9bd7])
- Record verification bundle rollback supersession history ([9b7d001])
- Write optimized model local profiles ([8f3b5b9])
- Publish optimized package metadata ([d7aa627])
- Record receipt rollback supersession history ([2191c50])
- Reconcile bundle rollback supersession ([502a747])
- Reconcile receipt rollback supersession ([e98465b])
- Record receipt supersession reconciliation history ([b1fe569])
- Record bundle supersession reconciliation history ([1a3ca9c])
- Add scheduler heartbeat contract ([d954a72])
- Add scheduler heartbeat history ([e1b3201])
- Add scheduler heartbeat reconciliation ([9bf1597])
- Add scheduler heartbeat supersession ([66cd1bc])
- Add scheduler heartbeat reconciliation history ([6f21d24])
- Add scheduler heartbeat supersession reconciliation ([1c05b5e])
- Add scheduler heartbeat supersession history ([f06289e])
- Add scheduler heartbeat supersession reconciliation history ([472db72])
- Add scheduler heartbeat pointer ([152b064])
- Add scheduler heartbeat pointer history ([84f9f99])
- Add scheduler heartbeat pointer reconciliation ([4727bef])
- Add scheduler heartbeat pointer supersession ([82b66cd])
- Add scheduler heartbeat pointer supersession history ([0eab0a8])
- Add scheduler runtime simulation harness ([4cbe5af])
- Add scheduler heartbeat pointer supersession reconciliation ([f266fde])
- Add burn bpk migration surface inventory ([0d85bb3])
- Add scheduler heartbeat pointer supersession reconciliation history ([018bd03])
- Add scheduler heartbeat pointer rollback ([c4a8bab])
- Add scheduler heartbeat pointer rollback history ([166dce2])
- Add scheduler heartbeat pointer rollback reconciliation ([58b003a])
- Add scheduler heartbeat pointer rollback reconciliation history ([24be72e])
- Add scheduler heartbeat pointer rollback supersession ([80d02e9])
- Add scheduler heartbeat pointer rollback supersession history ([aa4c6e7])
- Add heartbeat pointer rollback supersession reconciliation ([356ef50])
- Add rollback supersession reconciliation history ([f85edf5])
- Move infer workflow behind native command ([0d135f5])
- Move rollout pointer mutation behind native commands ([8274014])
- Group distributed shard lineage workflows ([0d7b0b9])
- Move check and verify behind native commands ([1405642])
- Collapse deployment verification manage surfaces ([c2b1fe0])
- Group pretraining source workflows ([28997b0])
- Collapse manage buckets in shard workflows ([cc694ec])
- Collapse scheduler heartbeat manage bucket ([d5357e6])

### Changed

- Standardize artifacts layout and include taplo.toml in flake ([41b6336])
- Rename artifacts/inference to artifacts/infer and update references ([173ebc1])
- Rename artifacts/infer → artifacts/inference (update code/docs/justfile) ([cadcadb])
- Return [1,28,28] tensor from mnist_image_to_tensor ([5ab0036])
- Unify CLI into afterburner subcommands ([13179ee])
- Extract artifact event helper ([93c5056])
- Split runtime artifacts metadata and events ([15485b7])
- Move heartbeat internals under debug deploy ([19f7a23])
- Cut over verification operator intents ([604d90c])
- Move queue lineage checks into rust ([31c5790])
- Extract rollback payload helpers ([3dcf04e])
- Roll out rollback payload helpers ([ff3dd01])
- Consolidate family json helpers ([166a3ee])
- Extract grouped dispatch helpers ([73e0288])
- Regroup debug deploy taxonomy ([a1a8813])
- Roll out launch command input helpers ([24a56d2])
- Roll out lineage command input helpers ([bc6bae6])
- Roll out drift command input helpers ([be1e298])
- Finish durability hardening ([0ffc865])
- Group scheduler heartbeat just surface ([8e73da5])
- Group deployment verification receipt core just surface ([abca69b])
- Group deployment verification receipt locator and rollback just surface ([3e560de])
- Group deployment verification bundle core just surface ([f2f12f0])
- Group deployment verification bundle locator and rollback just surface ([af0da7f])
- Group deployment verification handoff just surface ([70d30b5])
- Collapse deployment-verification action buckets ([a5152ee])
- Collapse distributed shard action buckets ([ae6ab47])
- Collapse scheduler-heartbeat action buckets ([2a0ecb9])
- Collapse provenance action bucket ([fa719e1])
- Normalize launch recipe grammar ([cae2d95])
- Normalize kube lease recipe grammar ([d557728])
- Normalize baseline recipe grammar ([ddcd489])
- Normalize handoff and locator recipe grammar ([ba6bdd9])
- Normalize handoff recipe grammar ([0179925])
- Normalize receipt recipe grammar ([0e9125e])
- Normalize bundle recipe grammar ([1970c2d])
- Normalize receipt rollback entry grammar ([d183107])
- Disambiguate bundle rollback recipe grammar ([1271e03])
- Normalize rollback pointer recipe grammar ([e05bbd8])
- Trim reference-owned catalog ([7169e39])
- Compress remaining fit anchors ([23f0589])
- Shift fit-detail ownership to reference ([fe49697])
- Trim remaining fit anchor churn ([0e1d423])
- Simplify reference validation surface ([8715201])
- Extract shared file helpers ([93595e7])
- Move shared helper module ([2dffe09])
- Dedupe deploy command prefixes ([33c30b0])
- Trim deployment verification surface ([7ba9f01])
- Trim deployment launch surface ([c01b56e])
- Trim scheduler heartbeat surface ([c990787])
- Trim drift and lease surfaces ([012d4f2])
- Trim remaining workflow surfaces and test warnings ([5538479])
- Split shared support helpers ([4abdebd])
- Extract queue metadata helpers ([cf7413a])
- Extract queue text item helpers ([c9b7e43])
- Type queue helper errors ([9ad2b97])
- Unify queue helper item model ([3a557ec])
- Extract shared process helpers ([eaf0c7a])
- Extract objective lock cli parsing ([42d0d0b])
- Extract queue snapshot cli parsing ([89ed136])
- Extract objective lock core ([739a172])
- Expose runtime backend list cleanly ([a7aa812])
- Extract queue snapshot core ([c53e0c7])
- Clean dead-code helper surfaces ([3270de3])
- Split queue snapshot helpers ([9d84f8e])
- Split objective lock scenarios ([20771ce])
- Isolate objective lock state ([3f9bf36])
- Extract queue order logic ([85cc1f0])
- Split repo workflow crate ([5ce1ff7])
- Split queue snapshot cli parser ([9061c6d])
- Split objective lock cli parser ([d29d6b5])
- Move lineage writers behind debug surface ([adf9a4f])
- Move launch writers behind debug surface ([5de3aa2])
- Move baseline writers behind debug surface ([d4ff6e2])
- Move kube lease writers behind debug surface ([cdb8173])
- Guard explicit single-node dp path ([262f68e])

### Chore

- Add initial Rust project scaffold (Cargo.toml, Cargo.lock, .gitignore, src/main.rs) ([118aa3b])
- Add devenv & direnv configs, update .gitignore, add initial model.rs ([249e039])
- Add flake.nix, taplo.toml, deny.toml, .cargo/audit.toml and .gitignore ([c4c241b])
- Add Nix flake and tooling configs (taplo, deny, direnv); update .gitignore and Cargo license ([4e2536e])
- Replace flake-utils with flake-parts, update rust-overlay & lockfile; docs: add Artifact Contract; feat(infer): log artifact loading and load model with recorder & device ([bc679fe])
- Add MIT LICENSE, clean up README, and add direnv to flake.nix ([cc2ad89])
- Require `cargo --locked` in justfile; add `direnv` and package metadata in flake; minor cleanup ([8aeec2f])
- Add nushell to flake packages ([92cf2a9])
- Enable just formatter in flake and reformat justfile ([9d6119d])
- Update burn training API ([9f107fd])
- Add git-cliff changelog ([2916592])
- Add changelog recipe ([dcf4b55])
- Enable mold linker and clang on Linux ([afbf484])
- Fix flake checks after input updates ([621a88b])
- Set git-cliff remote to upstream and clarify NOTE.md guidance ([300dc94])
- Align queue metadata with updated rules ([faba00b])
- Normalize CHANGELOG list formatting, add TODO entry, and update git-cliff config/invocation ([6e763c9])
- Use git-cliff --offline when regenerating CHANGELOG ([d087302])
- Move offline setting to Cargo.toml and remove --offline from justfile ([e3cf388])
- Codify durable tooling guards ([03f664a])
- Clarify profiling blocker ([046aab1])
- Harden crane-backed verification ([83ec760])
- Refresh changelog snapshot ([14cbc70])
- Advance deployment verification TODOs ([c31c94b])
- Reorder TODOs by ROI ([8450451])
- Promote optimized packaging contract ([408ee10])
- Promote optimized model follow-ons ([40ed6f7])
- Refresh changelog snapshot ([df645fa])
- Promote optimized publish adapter ([6ef2373])
- Advance publish adapter follow-ons ([05de1f7])
- Park deterministic simulation harness ([2feefac])
- Refresh changelog snapshot ([7ae9c61])
- Advance tokio fit gate ([b36e66f])
- Advance receipt rollback supersession history ([42cb8d7])
- Advance bundle rollback supersession reconciliation ([7cb56f6])
- Advance receipt rollback supersession reconciliation ([a7b901e])
- Advance receipt supersession reconciliation history ([351ec70])
- Advance bundle supersession reconciliation history ([376e8bf])
- Advance scheduler heartbeat contract ([3a36714])
- Advance scheduler heartbeat history ([b7c84ad])
- Advance scheduler heartbeat reconciliation ([0dd6ed8])
- Advance scheduler heartbeat supersession ([30c0657])
- Advance scheduler heartbeat reconciliation history ([8745849])
- Advance scheduler heartbeat supersession reconciliation ([60f100c])
- Advance scheduler heartbeat supersession history ([a80eaba])
- Advance scheduler heartbeat supersession reconciliation history ([2c75e73])
- Prioritize operator cli cleanup ([904d2a3])
- Refresh changelog snapshot ([3bf2461])
- Advance operator cli collapse ([f9cef32])
- Advance scheduler heartbeat pointer ([541f78f])
- Advance scheduler heartbeat pointer history ([bcc47f6])
- Advance scheduler heartbeat pointer reconciliation ([b81622c])
- Advance scheduler heartbeat pointer supersession ([dfe389d])
- Advance scheduler heartbeat pointer supersession history ([34a4f92])
- Advance scheduler runtime simulation harness ([c0457bf])
- Advance scheduler heartbeat pointer supersession history ([4cf08bb])
- Unblock burn bpk migration planning ([3ce5773])
- Advance burn bpk inventory report ([49a07ce])
- Advance pointer supersession reconciliation history ([bab847b])
- Advance pointer rollback adapter ([18bc297])
- Advance pointer rollback history ([3cfb2c1])
- Advance pointer rollback reconciliation ([cde8cd9])
- Advance pointer rollback reconciliation history ([7461b93])
- Advance pointer rollback supersession ([9bbf1a3])
- Advance pointer rollback supersession history ([cda7a1c])
- Advance pointer rollback supersession history ([6b0da75])
- Add readme front-page cleanup item ([555dd06])
- Advance pointer rollback supersession reconciliation ([d92078e])
- Advance rollback supersession reconciliation history ([5b81d02])
- Advance burn release availability gate ([c1b43b7])
- Advance burn release availability gate ([2ba8ec7])
- Refresh queue snapshot lineage ([c18a0d8])
- Advance rollback helper extraction ([0565323])
- Advance rollback helper rollout ([4a2e1c3])
- Advance rollback helper adoption ([2a95e0a])
- Advance readme front page ([9491b99])
- Advance readme front page ([e580407])
- Advance github metadata candidates ([c559afe])
- Add doc-test snapshot debt ([f069e49])
- Capture repo drift debt ([99a14e9])
- Refine orchestration drift wording ([aff116e])
- Reprioritize cleanup debt ([529561e])
- Advance protocol helper consolidation ([e57b234])
- Insert optimizer checkpoint docs unblocker ([c96ed02])
- Advance optimizer checkpoint docs unblocker ([7e4d9f3])
- Add orchestration semantic guard ([49025c5])
- Refresh queue snapshot lineage ([47006c7])
- Widen orchestration refactor scope ([15bd378])
- Advance dispatch decomposition ([d4952c4])
- Add debug deploy taxonomy regrouping ([6f915fc])
- Advance reference surface consolidation ([6796657])
- Advance debug deploy taxonomy regrouping ([086cfe6])
- Advance doc admission burn-down ([9b7309d])
- Split helper rollout slice ([54d2db1])
- Split helper rollout further ([3d0fd7b])
- Advance launch helper rollout ([775e493])
- Advance lineage helper rollout ([bfb331e])
- Split drift helper rollout ([71d1337])
- Advance drift helper rollout ([f708791])
- Split cleanup provenance rollout ([ffba2cd])
- Promote orchestration extraction ([be2d50b])
- Refine orchestration extraction slices ([ee3289f])
- Split grouped entrypoint extraction ([aba64b4])
- Split deploy-side grouped extraction ([9f6c607])
- Advance heartbeat grouped extraction ([6fc6863])
- Tighten same-commit queue wording ([04d350e])
- Refresh queue snapshot lineage ([94b547b])
- Split deployment verification extraction ([eb36f20])
- Split deployment verification receipt extraction ([9142545])
- Add formatter and search heuristics ([2427295])
- Refresh queue snapshot lineage ([08a6253])
- Split deployment verification bundle extraction ([f2302e4])
- Refresh queue snapshot lineage ([e84b3f6])
- Advance bundle rollback grouping ([aec7d16])
- Prioritize queue hardening ([4b29939])
- Drop stale scheduler heartbeat todo ([a46a6c1])
- Drop stale pretraining provenance todo ([dcbbcb9])
- Formalize dp-first capability queue ([5d7b51a])

### Documentation

- Add AGENTS.md with guidelines on truth, behavior, reasoning, output, coding, tool interaction, safety, and auditability ([283d0fa])
- Clarify and streamline guideline wording ([6f45e66])
- Restructure AGENTS.md to improve clarity and add coding, tool, and search guidance ([1bcccf7])
- Clarify assumptions, facts, and uncertainty; tweak tool/search wording ([3d3181a])
- Add ADR-001 (training vs inference) and ADR-002 (immutable artifact contract); update README and AGENTS.md ([2c824be])
- Clarify artifact wording and update preprocessing manifest TODO ([c14d65b])
- Add trade-offs section and clarify README; minor code tidy ([0c9d78f])
- Add "Assumptions" section and clarify README; style(train): reorder imports ([f1248e5])
- Remove Architecture/ADRs/Next Steps links and add License: MIT ([ea3c8bf])
- Rename architecture & next-steps to ARCHITECTURE.md & ROADMAP.md, update refs and simplify AGENTS.md ([8b74d56])
- Update document title from "Next steps" to "Roadmap" ([55e9131])
- Replace Rules section with Project-specific constraints clarifying ADRs, decision index, and doc-update expectations ([086147c])
- Combine guidelines for changing behavior and clarify ADR linking ([8007d08])
- Clarify that behavior changes require updating docs ([3858b0f])
- Add rollback snippet ([ce6b8a7])
- Refresh changelog TODOs ([ffdc207])
- Clarify just workflows ([aa42936])
- Refresh changelog TODOs ([b635ab7])
- Clarify CHANGELOG handling and add "Evolving organism" TODO loop requiring gates/artifacts and TODO chaining ([57f373e])
- Add invariants preventing core->adapter dependencies ([489ced4])
- Clarify core/adapters boundaries and contract/versioning ([29aeb4f])
- Add integration policy and remove redundant "Keep changes small and reviewable" guideline ([0ae2b91])
- Add Notes link and require human-level review bar ([ff6d689])
- Require confirmation before introducing new subsystems (UI/persistence/RPC) ([1e799f9])
- Clarify guidance for introducing new subsystems ([a7c1975])
- Regenerate ([dc2fef8])
- Require canonical contract-resolution and TODO focus tags; update README & changelog ([1d3ff39])
- Add focus-area list to TODOs and polish README/ARCHITECTURE/ADR text ([33b2324])
- Keep ADRs current-state by default ([dedb601])
- Add eval baseline refresh flow gate ([2361d4a])
- Clarify maintainability guideline ([0fe351c])
- Clarify core constraints and add architecture selection & patterns ([cecd798])
- Clarify serialization rules for conflicting tasks and fix list formatting ([f060b0a])
- Add guidance on explicit state, default backward-compat, and enforceable invariants ([90ec1c4])
- Set cutover as default; require version bump for breaking public contracts ([e092954])
- Refresh TODO queue and regenerate changelog ([36a1d02])
- Refresh TODO queue and regenerate ([900bb60])
- Advance TODO queue and regenerate ([cc117d9])
- Require behavior TODOs to introduce/strengthen gates ([f6b3365])
- Require TODO Kind field and governance approval; refresh TODOs in CHANGELOG/Cargo.toml ([4fb323d])
- Require ephemeral parallel workspaces under /tmp/<repo>/workspaces/ ([2fbdb3c])
- Stabilize TODO horizon and park deploy-rs ([1157d1a])
- Align TODO decomposition policy and metadata ([22a3839])
- Add backlog and park long-horizon items ([bb98e59])
- Add promotion rollback orchestration gate ([c2d8837])
- Align TODO promotion and backlog semantics ([04449f7])
- Remove ephemeral workspace /tmp path recommendation ([97e7620])
- Record arrow and parquet fit ([d697bea])
- Record cubecl kernel fit ([d696c68])
- Record rl environment fit ([dfdb6e2])
- Define multibillion target envelope ([7996475])
- Record otel profiling fit ([ddf1ee5])
- Add burn format migration follow-up ([d0e687f])
- Add ron format investigation ([921e233])
- Define distributed checkpoint index ([87adeaf])
- Park burn distributed alignment work ([3ea5b67])
- Advance active horizon ([604223b])
- Define artifact retention envelope ([4be1f7a])
- Advance retention completion ([b21cbaf])
- Define profiling hotspot taxonomy ([b0c42a7])
- Define pretraining source registry ([7889db8])
- Add deferred cli and remote artifact follow-ups ([86b81ba])
- Define deployment verification receipt ([9366c4e])
- Define distributed shard lineage ([7ed4651])
- Define artifact cleanup dry-run receipt ([8bde466])
- Define profiling environment provenance ([923213f])
- Define pretraining source approval receipt ([5d5f466])
- Define deployment verification evidence provenance ([f3f7840])
- Define distributed shard lineage receipt ([3b8fb49])
- Define artifact cleanup execution receipt ([98f3063])
- Define profiling provenance receipt ([064513d])
- Define pretraining source provenance receipt ([eac0cbd])
- Define distributed shard lineage evidence provenance ([132107b])
- Record low-leverage queue heuristic ([a626d1d])
- Regroup active horizon toward prod leverage ([5d93e84])
- Prioritize production-oriented load and deploy work ([a91fc03])
- Sync regrouped active queue ([81d63a7])
- Record Burn refresh stance ([b625ad2])
- Prioritize distributed parallelism profile ([c4f1441])
- Prioritize distributed tracing and parallelism profile ([fb040e9])
- Split distributed parallelism epic into causal chain ([e0652fe])
- Add blocked-by chain for distributed training profile work ([372f9db])
- Use blocked-by lists for distributed training chain ([76abac1])
- Restore distributed training prerequisite chain ([f000bc8])
- Clarify burn runtime and scheduler tracks ([df8c1e6])
- Add native metal migration todo ([953848f])
- Add hugging face publish todo ([904cc7a])
- Add macbook text pretraining todo chain ([fd35014])
- Separate runtime scheduler and platform layers ([6c3aca6])
- Scope distributed training subset ([486da79])
- Add optimization and packaging track ([5d8881c])
- Add distributed layer diagram ([e66dbe8])
- Add process-compose local substrate fit ([f6a9bb5])
- Add burn learning strategy fit gate ([88b14e1])
- Define world and rank topology gate ([4e567a5])
- Define runtime capability surface ([d708257])
- Define preemption and colocation stance ([55bce19])
- Add runtime profile schema gate ([00e256b])
- Define lifecycle boundary contract ([0c969b5])
- Define macbook text fit ([0020673])
- Add process-compose local fit ([e4e9c1c])
- Adopt fineweb-edu source gate ([b946e88])
- Add tokenizer packing contract ([150abf2])
- Add remote save load fit ([f3aeb32])
- Park blocked bpk migration ([3e3c04a])
- Define text artifact contract ([9001f06])
- Define optimization packaging fit ([e7d722b])
- Add layout feasibility gate ([6e2e7b4])
- Constrain gpu colocation fit ([ed9d7eb])
- Add cutile fit gate ([7b0fb2f])
- Advance past completed load profile ([edd955a])
- Record consolidation heuristics ([1f7a198])
- Record monorepo harness heuristic ([bdfabea])
- Make just canonical surface ([32d63a5])
- Advance to source provenance receipt ([f62c75c])
- Reprioritize train profiling and readme items ([14c151f])
- Sharpen frontpage ([de024d0])
- Prioritize readme frontpage pass ([1637413])
- Sharpen frontpage ([4af8aac])
- Split workflow reference from frontpage ([c289810])
- Split frontpage reference index ([4932265])
- Expand contract coverage index ([e0cfe47])
- Advance past reference coverage gate ([1e507eb])
- Collapse heavy reference sections ([33640a8])
- Move deep contract checks into docs ([27002f3])
- Tighten frontpage scope ([31ce7d7])
- Reduce frontpage to core navigation ([87abf6a])
- Section invariants by concern ([656e2bb])
- Compress fit catalog summaries ([de3db83])
- Complete workflow reference gate ([1f06ca3])
- Keep json over ron ([a1124a2])
- Define optimizer checkpoint state gate ([71f73c7])
- Define kube-rs placement fit ([7c85ef0])
- Reduce recipe duplication ([3b41fab])
- Compress stance catalog ([c361f52])
- Park capnproto investigation ([863d786])
- Refresh stale active horizon ([695cfc0])
- Require queue freshness and objective locks ([5fa5db1])
- Refresh queue snapshot after harness hardening ([d39df9d])
- Refresh queue snapshot after workflow fix ([619cbab])
- Record capnproto wire-format fit ([6eac14b])
- Record helper-surface consolidation heuristic ([6cdcb37])
- Refresh queue snapshot after docs updates ([f9e57f7])
- Record lws cluster workload fit ([5db5756])
- Record gateway api inference extension fit ([c115fd4])
- Refresh queue snapshot after bundle history ([ef07659])
- Refresh queue snapshot after transport locator history ([d4d6948])
- Refresh queue snapshot after bundle transport locator ([3eed571])
- Refresh queue snapshot after bundle transport locator ([5023bd6])
- Refresh queue snapshot after bundle transport locator reconciliation ([005d1a2])
- Refresh active queue after bundle transport locator history ([19b5f40])
- Refresh queue snapshot after bundle transport locator history ([42dc08d])
- Refresh queue snapshot after receipt transport locator ([e4338cf])
- Refine commit message heuristics ([189cac3])
- Repair active queue after receipt reconciliation ([9f2826a])
- Repair rollback queue order ([2c67074])
- Record dependency refresh maintenance heuristic ([f756482])
- Tighten commit subject heuristic ([3194669])
- Tighten workflow-surface heuristics ([3e35510])
- Clarify distributed reference roles ([1941647])
- Record operator surface boundary ([e438735])
- Record operator command model ([2683e90])
- Add public cli admission rule ([803d51b])
- Move operator matrix into deployment section ([d34a25e])
- Tighten front page ([0953edc])
- Add github metadata candidates ([6b775f1])
- Collapse github metadata workstream ([7d1746e])
- Restore checkpoint group anchor ([4676420])
- Group deployment verification surfaces ([ba78f03])
- Add queue status reporting rule ([f85091a])
- Clarify single-objective commit heuristic ([74ee370])
- Capture end-to-end workflow-surface heuristic ([09a0bb5])
- Tighten grouped operator-surface heuristic ([92d7a4f])
- Keep workflows grouped and move details to reference ([31c3191])
- Advance horizon after github repo settings update ([7545042])
- Reprioritize deployment verification grammar ([758d04a])
- Repair stale active todo horizon ([c3b94ee])
- Remove brittle scheduler entrypoint count ([6c0950a])
- Narrow drift docs todo scope ([3595c8b])
- Group workflow surface and move details to reference ([bde6cc6])
- Tighten queue objective reporting ([fa40090])
- Group workflow surface and move details to reference ([23caa0d])
- Group workflow surface and move details to reference ([a58cf13])
- Move blocked burn migration below runnable work ([e1c7f4c])
- Group deployment-stack workflow surface and move details to reference ([2b72984])
- Group deployment-utility workflow surface and move details to reference ([ab055bc])
- Clarify low-friction jj control points ([2317a52])
- Narrow remaining docs-family todo scopes ([0393cac])
- Add grouped deployment matrix surface guard ([7051100])
- Forbid cosmetic queue-horizon follow-ons ([c14760c])
- Calibrate justfile midpoint-progress wording ([5f066b3])
- Narrow source and lineage workflow wording ([0227edb])
- Prioritize public CLI admission gate ([ce83478])
- Tighten deployment-verification workflow wording ([24b6d55])
- Tighten scheduler-heartbeat workflow wording ([c07c013])
- Park blocked burn bpk migration ([346d83f])
- Tighten grouped workflow wording ([9d057e4])
- Tighten grouped workflow wording ([bfa29b7])
- Tighten grouped workflow wording ([332ef34])
- Tighten grouped workflow wording ([a272f78])
- Tighten grouped workflow wording ([503125b])
- Tighten grouped matrix wording ([7c6d774])
- Tighten grouped utility wording ([dc2c85b])
- Tighten grouped workflow wording ([0201ff8])
- Tighten grouped stack wording ([b51485e])
- Tighten grouped workflow wording ([a2ded8a])
- Align grouped operator-flow wording ([7988a65])
- Tighten grouped reference wording ([0f65069])
- Tighten grouped scheduler wording ([bd5cf4e])
- Tighten grouped source and lineage wording ([c58c147])
- Tighten grouped reference wording ([da8dcf7])
- Align deployment-stack shared validation wording ([db892d8])
- Drop stale cleanup split gate ([9a31b3a])
- Drop stale profiling section split gate ([2b55941])
- Drop stale drift section split gate ([ecdff5a])
- Add shared-gate stale-top precheck ([c5f7735])
- Drop stale Burn inventory split gate ([e8fab39])
- Drop stale drift extended split gate ([0a21484])
- Regroup top validator batch ([bff0ee1])
- Refine lint cleanup heuristic ([6050723])
- Merge runtime backend fit decisions ([9014e7b])
- Merge distributed runtime fit decisions ([e283bfd])
- Merge artifact serialization fit decisions ([2ab5f99])
- Merge cluster scheduling fit decisions ([f8968ee])
- Merge local execution fit decisions ([d9c24d0])

### Fixed

- Align artifact resolution and runtime fallback ([fbfa228])
- Enforce rollout budget schema gate ([e2c0b4c])
- Enforce strict pretraining metadata shape ([f7b370f])
- Align single infer success envelope ([1040623])
- Tolerate queue-refresh commit in snapshot check ([ddd9360])
- Correct handoff transport locator history context ([03127dd])
- Add explicit top-scope repair path ([e5f4a44])
- Harden queue preflight against stale state ([fa920fa])
- Add queue resume and completion boundary guard ([865714b])
- Add explicit stale-snapshot repair mode to preflight ([a003f63])
- Carry changelog baseline through queue resume ([3c6b0c1])
- Harden completion boundary for shared docs scopes ([79a7c2a])
- Block execution on blocked top active TODO ([6b00de2])
- Promote runnable backlog items ([bacc087])
- Extract execute preflight helper ([255f741])
- Carry repaired metadata into execute preflight ([5e02bcf])
- Allow in-flight top-scope repair ([44268ce])
- Harden stale repo lock recovery ([b86b6e0])
- Use dependency chaining for queue recipes ([4ec1d55])
- Harden test verification slices ([e62bb20])
- Clarify missing cli flag errors ([87a03cc])
- Refresh grouped workflow test targets ([8a7d2ba])
- Resolve clippy warnings ([8fa3d61])

### Other

- Revert "feat(devenv): enable Rust and mold in devenv.nix" ([271f315])
- Revert "chore(devenv): add devenv & direnv configs, update .gitignore, add initial model.rs" ([3d1724d])
- Apply canonical formatter ([fd46fc0])

### Tests

- Add golden fixtures for contract stability ([7a104bf])
- Add infer response golden fixtures ([7238ba4])
- Add HTTP graceful shutdown integration gate ([ef7d0b0])
- Add telemetry envelope fixture gate ([885523f])
- Add HTTP error contract fixtures ([106c08f])
- Add fixture dataset hash gate ([9529c9a])
- Add build graph guard gate ([04b7004])
- Add preprocess numerics guard ([0a84cd9])
- Add RL rollout schema fixture gate ([0eff34b])
- Add kernel logits shape guard ([fcd1bc9])
- Add distributed artifact copy smoke gate ([6f3bb1a])
- Add schema fixture gates for metadata stubs ([2da5431])
- Align schema fixture validation updates ([3dfec67])
- Add monitoring artifact schema gate ([9691537])
- Add infer_done event fixture gate ([30201db])
- Add summary success fixture gate ([b0b12a5])
- Add backend profile gate ([6f7b290])
- Restore active horizon gate ([b084fab])
- Add target profile example fixture ([a916ce8])
- Pin train_start event fixture ([eefaeb3])
- Pin artifact_exported event fixture ([a354b0b])
- Lock architecture and docs ownership roles ([56c9109])
- Gate optimized model capability surface ([c4446a4])
- Define optimized model packaging contract ([c9f30dc])
- Gate tokio runtime fit ([cad7ef4])
- Gate burn stable release availability ([eafeb6a])
- Gate rollback helper adoption ([8370836])
- Reduce rollback doc-policing ([d85bfe9])
- Trim verification rollback doc-policing ([2aeda96])
- Trim verification rollback history doc-policing ([08e6df7])
- Trim verification locator doc-policing ([bf41a4f])
- Trim verification bundle locator doc-policing ([1e579e0])
- Trim verification transport doc-policing ([5a04821])
- Add doc-test admission guard ([ce76111])
- Refresh burn inventory example ([557d69e])
- Burn down stale admission snapshots ([505f2b6])
- Add routing orchestration surface gate ([0041774])
- Add public CLI admission gate ([727fe00])
- Add grouped operator grammar gate ([a81ecea])
- Add justfile and reference doc gates ([9251585])
- Strengthen grammar, thinness, and reference gates ([35c9190])
- Align grouped matrix validation wording ([88ed206])
- Align rollout shared validation wording ([1052c29])
- Align scheduler shared validation wording ([7458414])
- Align deployment-verification shared validation wording ([18966f5])
- Align grouped reference heading gates ([35b02f7])
- Align source and lineage split headings ([376bdce])
- Align rollout split heading ([295188a])
- Align scheduler split heading ([e4b39c5])
- Align deployment-stack split heading ([cf5bfb9])
- Align deployment-verification split flows ([70aa9c9])
- Align cleanup split family ([72c9279])
- Align profiling split family ([b60cf0d])
- Align drift split family ([776244d])
- Align source section split heading ([bbb287e])
- Align deployment section split heading ([5c83b8f])
- Align deployment-utility split artifacts ([54a3da9])
- Align kube-lease split cluster ([fe78d77])
- Align rollout split artifacts ([612ec19])
- Align scheduler artifact split cluster ([e9a1775])
- Align deployment-stack launch artifacts ([6f96cf2])
- Align deployment-verification receipt and handoff artifacts ([85d86de])
- Align deployment-stack locator artifacts ([0cb0918])
- Align deployment-stack check artifact ([b25218f])
- Align training-side artifact and event lines ([3b0b4bc])
- Batch front contract split assertions ([43a9d4b])
- Batch front contract split assertions ([c877766])
- Batch shared workflow-link split gates ([87f756c])
- Batch remaining split validator sibling gates ([1a5f52e])
- Batch reference validation and reprioritize justfile debt ([96d2498])
- Cover queue snapshot cli parser ([b30e2ac])
- Reduce fit test fanout ([a7a0baf])
- Reduce contract anchor test fanout ([1fdee5c])
- Reduce distributed runtime fit fanout ([97dd62f])
- Reduce format fit test fanout ([b8b9db9])
- Reduce workload boundary fit fanout ([4db3092])
- Reduce preprocess test fanout ([e5acec4])
- Reduce helper rollout test fanout ([f0309e2])
- Reduce deployment verification surface fanout ([f4f6828])
- Reduce docs surface test fanout ([0467bd0])
- Reduce workflow family surface fanout ([69ca513])
- Reduce cli surface test fanout ([e31fe2c])
- Reduce runtime deployment test fanout ([fbf3db0])
- Reduce event contract test fanout ([b6d60fc])
- Reduce rollout orchestration fanout ([0068d34])
- Reduce repo structure test fanout ([f410918])
- Reduce deployment surface fanout ([be2ee09])
- Reduce markdown assertion fragility ([c74ec5c])
- Add package-scoped verification entrypoints ([8c60b1f])

[Trunk]: https://github.com/zxfsee/afterburner/commits/HEAD
[118aa3b]: https://github.com/zxfsee/afterburner/commit/118aa3bd3a2e294be709228903dcdfdfa8e9e6ed
[249e039]: https://github.com/zxfsee/afterburner/commit/249e0391e0b8dd5ec107e2bdebaf62bff319fc63
[94e3119]: https://github.com/zxfsee/afterburner/commit/94e311907737a621bc3a55af4f6a2f1302bc0f27
[271f315]: https://github.com/zxfsee/afterburner/commit/271f315462dcf6b7e5a338daa98558e57c12a1e2
[3d1724d]: https://github.com/zxfsee/afterburner/commit/3d1724ddda537a6329ee1a8ae32eae327db0aa0b
[c4c241b]: https://github.com/zxfsee/afterburner/commit/c4c241be90ec76ebc559cedaec4918566c2c8e7e
[4e2536e]: https://github.com/zxfsee/afterburner/commit/4e2536e7c25b25ef47bef18d406e89aac9804292
[8a19388]: https://github.com/zxfsee/afterburner/commit/8a193886fadf822d02395b24ed8eeec71a7e9869
[283d0fa]: https://github.com/zxfsee/afterburner/commit/283d0fa3861f3b0bbdf4edccbe04a9bf42a2b7fc
[2535151]: https://github.com/zxfsee/afterburner/commit/2535151770092d1985b39d3e9ef6206209aa3fb4
[6f45e66]: https://github.com/zxfsee/afterburner/commit/6f45e6648c099a261903384b37cbeece061d695e
[ac473da]: https://github.com/zxfsee/afterburner/commit/ac473dac8c9124d0962e68579acb508c1e80934a
[bc679fe]: https://github.com/zxfsee/afterburner/commit/bc679fef261e3e2934732bfeb0aec445ece90e42
[1bcccf7]: https://github.com/zxfsee/afterburner/commit/1bcccf7a5aace7c7d446feced33817afd999bf27
[3d3181a]: https://github.com/zxfsee/afterburner/commit/3d3181a9610343f6754bcd186247b49a134b28ad
[d21a891]: https://github.com/zxfsee/afterburner/commit/d21a8910e5ca657c354ab8e560158d7f6daa39de
[97ef620]: https://github.com/zxfsee/afterburner/commit/97ef620ea137dd4c923249bb9dcc10ff05a9351a
[cc2ad89]: https://github.com/zxfsee/afterburner/commit/cc2ad8968448e202229cc0f2f9f80f161c8c0563
[2c824be]: https://github.com/zxfsee/afterburner/commit/2c824be9cab0fb428064598a28fba5249de4e2b2
[41b6336]: https://github.com/zxfsee/afterburner/commit/41b6336494b10542c4b47b51f2a24985f7549d2b
[8aeec2f]: https://github.com/zxfsee/afterburner/commit/8aeec2f69565a5dd7048b5dcd13f7b7c7b975ca9
[e36bcda]: https://github.com/zxfsee/afterburner/commit/e36bcdab992098b9c361e20cfc17b67bcd22d6de
[173ebc1]: https://github.com/zxfsee/afterburner/commit/173ebc163dd7435b96abac229e8fe5422e4118d8
[c14d65b]: https://github.com/zxfsee/afterburner/commit/c14d65bd035f0ee0f8fa191f15e6a2c8e32a1919
[cadcadb]: https://github.com/zxfsee/afterburner/commit/cadcadbeb680679c353f7ce4f0ce13b5ad123e5f
[0c9d78f]: https://github.com/zxfsee/afterburner/commit/0c9d78fb90abef23286d96513b8b05120d4b7bf3
[f1248e5]: https://github.com/zxfsee/afterburner/commit/f1248e54220a7d0758ad17893efb2a6ff516c9e7
[6050ad6]: https://github.com/zxfsee/afterburner/commit/6050ad60829fea10073f75236aca50dd6efa71ec
[3445782]: https://github.com/zxfsee/afterburner/commit/34457825780d2190b7c8bf8091000406806c0601
[13ffa8e]: https://github.com/zxfsee/afterburner/commit/13ffa8e48b770688de7a939d269739cb456fc73d
[ea3c8bf]: https://github.com/zxfsee/afterburner/commit/ea3c8bf18dddbf5481df8143b6e005506b7fb976
[92cf2a9]: https://github.com/zxfsee/afterburner/commit/92cf2a9a5c5496d79021b3b6922e97fdc7691fc1
[9d6119d]: https://github.com/zxfsee/afterburner/commit/9d6119d51fb770790213abfdd4ec0050651e3d69
[8b74d56]: https://github.com/zxfsee/afterburner/commit/8b74d569c7a7bc25f19d99b9c6460a8eba717eeb
[55e9131]: https://github.com/zxfsee/afterburner/commit/55e9131627b870f786791192aa03b90b05d20539
[086147c]: https://github.com/zxfsee/afterburner/commit/086147ceb63c0ea18b0af05601f753525971fc2f
[8007d08]: https://github.com/zxfsee/afterburner/commit/8007d08f11169e2ac99cd6396834b74706a25843
[3858b0f]: https://github.com/zxfsee/afterburner/commit/3858b0f2352fdcf672089c76ff2b6a0ffa65ab2d
[7f3a4ef]: https://github.com/zxfsee/afterburner/commit/7f3a4efa8ad4ed13a13e50a2eeeac75940621743
[ae91a99]: https://github.com/zxfsee/afterburner/commit/ae91a999a96e9647c81fe7a7436e7a0e2c98be87
[5ab0036]: https://github.com/zxfsee/afterburner/commit/5ab0036430c66b3c753ceabd822c2bfcd8f2d826
[7a104bf]: https://github.com/zxfsee/afterburner/commit/7a104bf1353e1dd38e03da11a4bc94cdb7f6434c
[b981d60]: https://github.com/zxfsee/afterburner/commit/b981d6096774c712ea3fd0caf027d90e6209fed3
[9f107fd]: https://github.com/zxfsee/afterburner/commit/9f107fdf546cd0d2d7743c4d0a6b5b5ed79e0c04
[be3bdce]: https://github.com/zxfsee/afterburner/commit/be3bdce5335fe2a70f7dd997efc58aa97930fd3d
[ce6b8a7]: https://github.com/zxfsee/afterburner/commit/ce6b8a703ed26772e8410af256e001585c1afdab
[4506a9a]: https://github.com/zxfsee/afterburner/commit/4506a9a9cb8db4c10ef807600384c69daa7fe9d0
[2916592]: https://github.com/zxfsee/afterburner/commit/2916592a1a3c66c8b2cfc828b5b7b75c49d0a587
[dcf4b55]: https://github.com/zxfsee/afterburner/commit/dcf4b55c198ad25c932e9d0cfd80a27d13657ce1
[ffdc207]: https://github.com/zxfsee/afterburner/commit/ffdc20706e5ceb284c024064ac081c2d7e42e713
[aa42936]: https://github.com/zxfsee/afterburner/commit/aa429365037b81f0d4667d7df9a9d7087ab0a970
[b635ab7]: https://github.com/zxfsee/afterburner/commit/b635ab7a9802d2824d2780cebb6b5b1f456cafee
[57f373e]: https://github.com/zxfsee/afterburner/commit/57f373e7bd26179b0042a35f16a332dbe51142de
[438b0e4]: https://github.com/zxfsee/afterburner/commit/438b0e401dba945dae236dcccea50385864f42ce
[489ced4]: https://github.com/zxfsee/afterburner/commit/489ced402845ee76ef11d5a081d9299d0136f6ea
[29aeb4f]: https://github.com/zxfsee/afterburner/commit/29aeb4f966d70400183769f3bc1e57f9f001af04
[0ae2b91]: https://github.com/zxfsee/afterburner/commit/0ae2b91537ec22772c6f4b2c917d71741ca15dec
[ff6d689]: https://github.com/zxfsee/afterburner/commit/ff6d6893b7187b49241d664995a634dcf5d39691
[afbf484]: https://github.com/zxfsee/afterburner/commit/afbf4844e2db48029bc4b5e8f2ac08d7f1f77cee
[1e799f9]: https://github.com/zxfsee/afterburner/commit/1e799f9ba3853e97cc7ea537257c8b3e6a5da879
[a7c1975]: https://github.com/zxfsee/afterburner/commit/a7c1975f788f04b489ffc9f29cc134c7b384271d
[621a88b]: https://github.com/zxfsee/afterburner/commit/621a88bb532258ffa4a87f72b38d9582fdc18421
[fbfa228]: https://github.com/zxfsee/afterburner/commit/fbfa228fb55bbfd81d24fb0eeb7ad4dc26b24150
[dc2fef8]: https://github.com/zxfsee/afterburner/commit/dc2fef8e53fc047de1b0ed020156ac0839afd248
[1d3ff39]: https://github.com/zxfsee/afterburner/commit/1d3ff39a3d7c03fcc98222d95797fd8ebab76bdb
[8a1ad14]: https://github.com/zxfsee/afterburner/commit/8a1ad14ee3bca7ee1db17bbc73e05b8a231fa3f7
[33b2324]: https://github.com/zxfsee/afterburner/commit/33b23248a678a73434dd7c9e75fcf6a8da604367
[13179ee]: https://github.com/zxfsee/afterburner/commit/13179eefb081c1341c22ad7f82573d8053d25b07
[dedb601]: https://github.com/zxfsee/afterburner/commit/dedb601ee5ca47f76e8e9acec54ef5b2b543eafc
[719ca73]: https://github.com/zxfsee/afterburner/commit/719ca73d55c734fee2692da9133021a4cea08496
[5bb4362]: https://github.com/zxfsee/afterburner/commit/5bb43629cd9c388aa9d378130e69fddd1f149a23
[6cad4fe]: https://github.com/zxfsee/afterburner/commit/6cad4feeb905714b160b9cab1db94cc8c9ff04d7
[641e796]: https://github.com/zxfsee/afterburner/commit/641e796dd2fa3667bb9fda4115fe6ce74abb642e
[7238ba4]: https://github.com/zxfsee/afterburner/commit/7238ba41d79b9a41bd1a91956e7b20d000be0a8c
[c0e779b]: https://github.com/zxfsee/afterburner/commit/c0e779bfcc7c8f83f9e18d37b5fb984eeb47c837
[ef7d0b0]: https://github.com/zxfsee/afterburner/commit/ef7d0b0177ce1be6e8456d6c7bb6e3aae233a330
[885523f]: https://github.com/zxfsee/afterburner/commit/885523f762533cf64e3f03317ebea4178e1f1eb6
[e70afe8]: https://github.com/zxfsee/afterburner/commit/e70afe847cadaf16b3574c15600505df53e0e8a0
[2361d4a]: https://github.com/zxfsee/afterburner/commit/2361d4a0f9d73ff805829a0485f342080323be3c
[106c08f]: https://github.com/zxfsee/afterburner/commit/106c08f77b900011774e471d6aeb99e3558238b4
[9529c9a]: https://github.com/zxfsee/afterburner/commit/9529c9afab2828de4b9bf1e0f26f7e9eed1e96ad
[04b7004]: https://github.com/zxfsee/afterburner/commit/04b700436f1da4e66d735dc14a4bfca006e7d79a
[0a84cd9]: https://github.com/zxfsee/afterburner/commit/0a84cd970070fea8db075e4e05ae0ee689b3d7e8
[0eff34b]: https://github.com/zxfsee/afterburner/commit/0eff34b2da6c35b4fe708f19fcb7ee6b65bed097
[fcd1bc9]: https://github.com/zxfsee/afterburner/commit/fcd1bc97339489befc4d568bd5c910c1ad5ada6f
[6f3bb1a]: https://github.com/zxfsee/afterburner/commit/6f3bb1a8748493d7cdb4b49585fc9c1a3af23f80
[0fe351c]: https://github.com/zxfsee/afterburner/commit/0fe351ca326595d4fdef30437bdc5f8faade5b8f
[cecd798]: https://github.com/zxfsee/afterburner/commit/cecd798e26432e519e5215d4d068737da524f741
[f060b0a]: https://github.com/zxfsee/afterburner/commit/f060b0ade388b94b2de5f9bc0d01a2aaa46cfdb4
[90ec1c4]: https://github.com/zxfsee/afterburner/commit/90ec1c42b073cbb33f0fe865baa9481ed57fdee6
[e092954]: https://github.com/zxfsee/afterburner/commit/e09295489879297f738e54f100acdea5ebbe8d23
[2da5431]: https://github.com/zxfsee/afterburner/commit/2da543111683141b51f9fd4565681d189b420bb4
[36a1d02]: https://github.com/zxfsee/afterburner/commit/36a1d02086b97e9720d59c4086cf8da69b262d71
[f8f847a]: https://github.com/zxfsee/afterburner/commit/f8f847a543a3fb84055a6f93599a119bccb5eae2
[3dfec67]: https://github.com/zxfsee/afterburner/commit/3dfec6706a507fa74b502c78c219b63824fb044b
[900bb60]: https://github.com/zxfsee/afterburner/commit/900bb601eaad8108edc6e5026bd5a6fd05b0b84d
[e2c0b4c]: https://github.com/zxfsee/afterburner/commit/e2c0b4c1df29a0ce72ada6c07af4ed03b920bc04
[f7b370f]: https://github.com/zxfsee/afterburner/commit/f7b370f6d0017b31f3c5144b0e9c37a3386ed039
[cc117d9]: https://github.com/zxfsee/afterburner/commit/cc117d926c4a47c878c83febb72b0e3eb4a2d8f1
[300dc94]: https://github.com/zxfsee/afterburner/commit/300dc949407455ba05a7c3fa2443f2e86a4db1d8
[30b6aab]: https://github.com/zxfsee/afterburner/commit/30b6aab36931e0c35893f6cd1c3e49f5f67411ce
[faba00b]: https://github.com/zxfsee/afterburner/commit/faba00bd2b19ab342427d7e0d7d6be2b50535f20
[f6b3365]: https://github.com/zxfsee/afterburner/commit/f6b3365b762b2bd5cf94c3e517f434f14a4b5db9
[4fb323d]: https://github.com/zxfsee/afterburner/commit/4fb323de80df3f2f4bf25e8213a737b6edf4e9d7
[6e763c9]: https://github.com/zxfsee/afterburner/commit/6e763c93251c2d49f8ba28b21485e2d41086d279
[9fc29da]: https://github.com/zxfsee/afterburner/commit/9fc29da9df910b48c1e877d1e6d98b49a6837ac5
[d087302]: https://github.com/zxfsee/afterburner/commit/d087302b2ae1f29bee5efaf00258ac71443816ca
[2fbdb3c]: https://github.com/zxfsee/afterburner/commit/2fbdb3c26ef0870f047fb99c754d9d0977ae405a
[1cbcdd7]: https://github.com/zxfsee/afterburner/commit/1cbcdd70f04e29d39172ad73e27a0dbbc5c6a024
[1157d1a]: https://github.com/zxfsee/afterburner/commit/1157d1aca6ab8330b957ee80a7f84600a258ade9
[22a3839]: https://github.com/zxfsee/afterburner/commit/22a3839e861d6235a6f9b949e28bd9276e14e28a
[bb98e59]: https://github.com/zxfsee/afterburner/commit/bb98e598e703e8eb9f59286a7d757c9bf5f8d3bd
[c2d8837]: https://github.com/zxfsee/afterburner/commit/c2d88375b5c49462fb78db535d405642cf0814cc
[04449f7]: https://github.com/zxfsee/afterburner/commit/04449f7d08c39b8b29879b34aef326ac0c76478d
[97e7620]: https://github.com/zxfsee/afterburner/commit/97e7620df8de7687d26a40004bafc9ca46aad0db
[25572b8]: https://github.com/zxfsee/afterburner/commit/25572b827cd087acace41ca21aa42b8ef675615e
[e3cf388]: https://github.com/zxfsee/afterburner/commit/e3cf388d747dd4784c11353fdae71c7c0f491374
[e283856]: https://github.com/zxfsee/afterburner/commit/e2838569b622d99ade2fee909dd4848ad138fabb
[9691537]: https://github.com/zxfsee/afterburner/commit/96915371230b5a965fb00d35d36d2f9f7e01d761
[5b80b82]: https://github.com/zxfsee/afterburner/commit/5b80b82b63549de55873273e4d69c23d5be875d4
[6495bbe]: https://github.com/zxfsee/afterburner/commit/6495bbea37490c16125842be4d606927c16e75ff
[30201db]: https://github.com/zxfsee/afterburner/commit/30201db3ecd755175f4d97618cc7378f5d528981
[b0b12a5]: https://github.com/zxfsee/afterburner/commit/b0b12a57ef92d69d6892d433bf82ba8372a95885
[1040623]: https://github.com/zxfsee/afterburner/commit/10406230ea17b4278f535cb41c8c0bf6157aa970
[6f7b290]: https://github.com/zxfsee/afterburner/commit/6f7b29028d52195a4c0057bdfcf3ebd270a6991c
[cb0efd1]: https://github.com/zxfsee/afterburner/commit/cb0efd1c84fc7bc0b4b13a5b005553864fbad08e
[09f053a]: https://github.com/zxfsee/afterburner/commit/09f053adb59dd310e65d29a6bf7f66f3d82ae2e5
[183bbd5]: https://github.com/zxfsee/afterburner/commit/183bbd58c632d25e0a47e8739acb4148b61ef490
[277391f]: https://github.com/zxfsee/afterburner/commit/277391fdbe8ed734f351e548d0de4d203dc1ebaf
[5f70ebf]: https://github.com/zxfsee/afterburner/commit/5f70ebf6fd0db4795677699d0b5a1664b008eda8
[b084fab]: https://github.com/zxfsee/afterburner/commit/b084fab8279a58ed4cdaa08380f1bccc23447b4f
[df702e0]: https://github.com/zxfsee/afterburner/commit/df702e0b7ad7a32e56eaae2e81bab36d8700a1d3
[53fa3ab]: https://github.com/zxfsee/afterburner/commit/53fa3abba1e6e1c554acb50a2fb646add852b807
[03f664a]: https://github.com/zxfsee/afterburner/commit/03f664a4e9d7e604510bf03cf52c9843549f0e43
[18bc718]: https://github.com/zxfsee/afterburner/commit/18bc7182b95fc59d382a2cad787fda9dacd8adb8
[046aab1]: https://github.com/zxfsee/afterburner/commit/046aab17075bc8daad9739ed451f9e55109c641c
[e3b73e1]: https://github.com/zxfsee/afterburner/commit/e3b73e179f60890c7f6fe55e2e8f384194ecf080
[2f55936]: https://github.com/zxfsee/afterburner/commit/2f55936f6c47286b1beed742e616f8983383d738
[a916ce8]: https://github.com/zxfsee/afterburner/commit/a916ce8c902a1e3435cb4fb98357e1cad8f01762
[d6fb65d]: https://github.com/zxfsee/afterburner/commit/d6fb65d4f8b525b9104e628d3fd1868012b649d6
[8e6c185]: https://github.com/zxfsee/afterburner/commit/8e6c18514e801d1f91b47591bc998267e164f356
[eefaeb3]: https://github.com/zxfsee/afterburner/commit/eefaeb3b12e27c289adde0661f366e5eb560d43d
[d697bea]: https://github.com/zxfsee/afterburner/commit/d697beaa626969b6c4dc6e50fe418477d35fd56a
[6c42dc3]: https://github.com/zxfsee/afterburner/commit/6c42dc3b9fcc2e88c6f6eb7854fc6fed60214972
[d696c68]: https://github.com/zxfsee/afterburner/commit/d696c6839119e503f71b8755f912eea69edc9933
[dfdb6e2]: https://github.com/zxfsee/afterburner/commit/dfdb6e2a97928c36164d85eeece6b130c9ec9f13
[a02470d]: https://github.com/zxfsee/afterburner/commit/a02470d81dc443f868f419ba039620f28dec447b
[a354b0b]: https://github.com/zxfsee/afterburner/commit/a354b0bfcc07f67e27f76c72458eddaa6da23e2a
[7996475]: https://github.com/zxfsee/afterburner/commit/799647512473b60c68a94eaebd4c7aac648686e9
[66269c9]: https://github.com/zxfsee/afterburner/commit/66269c9a0e43f22886ce486c8671ee9fe8ac10f9
[ddf1ee5]: https://github.com/zxfsee/afterburner/commit/ddf1ee5185cfcc6fb4372a35eb985afa48193369
[d0e687f]: https://github.com/zxfsee/afterburner/commit/d0e687f683e7fe17bf7ac68ccb580ffdacb160f3
[3619d37]: https://github.com/zxfsee/afterburner/commit/3619d37a6bcb171e2ef36be65d3f9a60b29e2996
[6f93b97]: https://github.com/zxfsee/afterburner/commit/6f93b97baf09b9f38fc569eab8e0d7394f9d6c66
[921e233]: https://github.com/zxfsee/afterburner/commit/921e233b480a133854359de172d21237f2767c20
[b30bcb0]: https://github.com/zxfsee/afterburner/commit/b30bcb09bc51339d9920c58b57efb7c27936c733
[01d23a9]: https://github.com/zxfsee/afterburner/commit/01d23a9b05040eb5d7893148566a61946826f573
[87adeaf]: https://github.com/zxfsee/afterburner/commit/87adeaf0c122310130a30c76b630478851e954af
[3ea5b67]: https://github.com/zxfsee/afterburner/commit/3ea5b67470213f9ede3ffa0139d03792117441b4
[96e45ae]: https://github.com/zxfsee/afterburner/commit/96e45ae530662556df5ac28e8f0e8ad4d73fb64a
[71e819e]: https://github.com/zxfsee/afterburner/commit/71e819e5c9aa6740ccddfe3fd554bf2af883db5f
[26f24f0]: https://github.com/zxfsee/afterburner/commit/26f24f03de59dfa303208bdd5a6e87b480888019
[c3b911e]: https://github.com/zxfsee/afterburner/commit/c3b911e9ae2938286ef295fec09acb3442856fa5
[d130225]: https://github.com/zxfsee/afterburner/commit/d130225f9b10e47e43a10668e525672af0ec31b4
[a828c2f]: https://github.com/zxfsee/afterburner/commit/a828c2fcbec3caeb35df828bfdc5460a372817d3
[d37aeed]: https://github.com/zxfsee/afterburner/commit/d37aeed8e565dc68e496067cf1db26a7932bc249
[9baf390]: https://github.com/zxfsee/afterburner/commit/9baf39062da2a5cf79c63024130c185f96f7c4b8
[91dec15]: https://github.com/zxfsee/afterburner/commit/91dec153e0eda10be71ecfa743e5e1967b2007f5
[6bbeb18]: https://github.com/zxfsee/afterburner/commit/6bbeb18722c3ca08a46d0b5d40353fa190304086
[4cfb1f9]: https://github.com/zxfsee/afterburner/commit/4cfb1f9eeca9079087d97cfb316130ce4fe7c60f
[7df36b3]: https://github.com/zxfsee/afterburner/commit/7df36b3dc7d7807b03a6a822e37f61fe3d721527
[604223b]: https://github.com/zxfsee/afterburner/commit/604223b51d05b450e3aaa0a660ce998cd50acbb1
[4be1f7a]: https://github.com/zxfsee/afterburner/commit/4be1f7a35a72efa7347b6ebb38088294644f78f2
[b21cbaf]: https://github.com/zxfsee/afterburner/commit/b21cbaf273e05d30300a3b85245bd4719aa8acea
[d11f075]: https://github.com/zxfsee/afterburner/commit/d11f0754301f7da902c04a9fea1e596b65c589bb
[b0c42a7]: https://github.com/zxfsee/afterburner/commit/b0c42a76bb0d2a015d2cb00ee33c8f0fa7ae8f53
[dbb098f]: https://github.com/zxfsee/afterburner/commit/dbb098f2c529418d458442a6dec0e52de9a2bc27
[7889db8]: https://github.com/zxfsee/afterburner/commit/7889db8b26c917ffeeab1616d446f7bf4a48cef3
[86b81ba]: https://github.com/zxfsee/afterburner/commit/86b81ba114422f0b94db67cdc3d1f4252fc943e3
[9366c4e]: https://github.com/zxfsee/afterburner/commit/9366c4e9e0200b7a3a4d997a50e141d928aa2640
[f44b6ce]: https://github.com/zxfsee/afterburner/commit/f44b6ce55eb03df67da1bed3b8d2d6212d51555b
[7ed4651]: https://github.com/zxfsee/afterburner/commit/7ed46516b1f1a345c95654f425df27714c6a316b
[8bde466]: https://github.com/zxfsee/afterburner/commit/8bde46649ba3f08d994e8b6bd374930d4b30d0d7
[47ebf56]: https://github.com/zxfsee/afterburner/commit/47ebf566beb8cfd8733b53c137f500692a41f13b
[923213f]: https://github.com/zxfsee/afterburner/commit/923213fd68b8766b0b4d3ba60b2f78772d01848f
[5d5f466]: https://github.com/zxfsee/afterburner/commit/5d5f466993d69d36be827d223d55cb051a09fa10
[ac3cbc5]: https://github.com/zxfsee/afterburner/commit/ac3cbc5283d39978885e5b50db0b6e53f4cfb305
[f3f7840]: https://github.com/zxfsee/afterburner/commit/f3f78406b25e929d899906dd81847ebb5133dd83
[3b8fb49]: https://github.com/zxfsee/afterburner/commit/3b8fb49ce23a6c6fefb509e306f122d618937ac2
[98f3063]: https://github.com/zxfsee/afterburner/commit/98f3063fa1f1c520345c8bc1cd441794e5842588
[064513d]: https://github.com/zxfsee/afterburner/commit/064513d7e89dbd2115d0cdaf1f8951c5fc4e885e
[984238a]: https://github.com/zxfsee/afterburner/commit/984238aaa6997b866baf76d8898984b11be1cd11
[eac0cbd]: https://github.com/zxfsee/afterburner/commit/eac0cbd6f067f2aea7420d32daf062b47f829191
[011b945]: https://github.com/zxfsee/afterburner/commit/011b945a9f25e4944967dbdeab82009e85bd6b21
[132107b]: https://github.com/zxfsee/afterburner/commit/132107bd236a9deafc61c9eebb0dd0fc2ef29f30
[a626d1d]: https://github.com/zxfsee/afterburner/commit/a626d1d023b2790de4dcc7a7348cf6a5c8861d70
[5d93e84]: https://github.com/zxfsee/afterburner/commit/5d93e84b3a6df28b857e9a5b1e5ed01d49dcdcf8
[a91fc03]: https://github.com/zxfsee/afterburner/commit/a91fc03ddd89a880864d3e99178acab5492939f9
[81d63a7]: https://github.com/zxfsee/afterburner/commit/81d63a7012b70ccd2834491f569380e83af070f8
[b625ad2]: https://github.com/zxfsee/afterburner/commit/b625ad2a8e059b1e7ffa3a2ca19425e316bc0224
[d96a0a7]: https://github.com/zxfsee/afterburner/commit/d96a0a74af12266e9a3d2f961c2d9b0a1410ee75
[7bf677b]: https://github.com/zxfsee/afterburner/commit/7bf677b6944699981490fc65608e1fc59d7ddbe7
[c4f1441]: https://github.com/zxfsee/afterburner/commit/c4f14417b908c652939478a478d79b8a57688bb8
[fb040e9]: https://github.com/zxfsee/afterburner/commit/fb040e9758745311f537482fc522381ba8a30b11
[e0652fe]: https://github.com/zxfsee/afterburner/commit/e0652fe4f847556adcc4859b54acbf4ff05c4b93
[372f9db]: https://github.com/zxfsee/afterburner/commit/372f9dbda6dc4250cb636f6e9e2c46336ce784d2
[76abac1]: https://github.com/zxfsee/afterburner/commit/76abac1903d4255ad83bc3e9f0cb7957f94b7f2f
[f000bc8]: https://github.com/zxfsee/afterburner/commit/f000bc85d5f625368120599dee418923906abbd2
[df8c1e6]: https://github.com/zxfsee/afterburner/commit/df8c1e60d72895a56451357250448ab27b7c97c9
[953848f]: https://github.com/zxfsee/afterburner/commit/953848f02e1c614cc97a87a852b803b3c291924f
[904cc7a]: https://github.com/zxfsee/afterburner/commit/904cc7ab3e611816ffb6ff3f8d99fe6ae11d0da6
[fd35014]: https://github.com/zxfsee/afterburner/commit/fd3501464cc96b561c827f1d3ccdd2766daa3818
[6c3aca6]: https://github.com/zxfsee/afterburner/commit/6c3aca629e66aebe35cc568e2c575f6cb84377da
[486da79]: https://github.com/zxfsee/afterburner/commit/486da79e81827eb165f295538977f86e5f391fc7
[5d8881c]: https://github.com/zxfsee/afterburner/commit/5d8881cd7247e3d1e6732fa857ce3f2e69a131a1
[e66dbe8]: https://github.com/zxfsee/afterburner/commit/e66dbe8e2ce745127371b68c21b75c59a1cb4096
[a4b7835]: https://github.com/zxfsee/afterburner/commit/a4b783597f576d10a3ee6c2dee5aba66d467cb13
[5000595]: https://github.com/zxfsee/afterburner/commit/5000595ad779a9026c56d10a02713ef240d8aea6
[f6a9bb5]: https://github.com/zxfsee/afterburner/commit/f6a9bb56d84f6827388d0a45508516a23a2a571b
[88b14e1]: https://github.com/zxfsee/afterburner/commit/88b14e1d51ed0ca03f7cf43c50aa2f7982237432
[4e567a5]: https://github.com/zxfsee/afterburner/commit/4e567a5b8edbaa0916eed05a4a6681941d2b5faf
[583a515]: https://github.com/zxfsee/afterburner/commit/583a51577684037990acd272a49a25e921d74297
[d708257]: https://github.com/zxfsee/afterburner/commit/d70825734d5e420e74ff4464fb7c758319d07892
[55bce19]: https://github.com/zxfsee/afterburner/commit/55bce199bda6f46b7082471c6ef2eae762eff658
[00e256b]: https://github.com/zxfsee/afterburner/commit/00e256b2c9454ff12befa4cc5407d3de7c1af090
[563476b]: https://github.com/zxfsee/afterburner/commit/563476b0757870436e0768f4c3c3cad34c0fbe13
[0c969b5]: https://github.com/zxfsee/afterburner/commit/0c969b5941a8fc276969933e368eae0a35e3d030
[007426e]: https://github.com/zxfsee/afterburner/commit/007426e76edbca41b91d486adf2cfeb27171cf81
[0020673]: https://github.com/zxfsee/afterburner/commit/002067333206df833364159bee3e21db0ceaa60a
[e4e9c1c]: https://github.com/zxfsee/afterburner/commit/e4e9c1c0eb8fea17e3e5d2f0906e98da6db85292
[b946e88]: https://github.com/zxfsee/afterburner/commit/b946e8888abc8bd3d7b117fabb25120e90016baa
[150abf2]: https://github.com/zxfsee/afterburner/commit/150abf2619fc51a71be57931f8f3c69e23d325ab
[f3aeb32]: https://github.com/zxfsee/afterburner/commit/f3aeb3276e88868365cd7755813af583c712b54e
[3e3c04a]: https://github.com/zxfsee/afterburner/commit/3e3c04a5f8c8f9992248f42036fa78ec7239d8ab
[9001f06]: https://github.com/zxfsee/afterburner/commit/9001f066ff357f881c7d189fe6dc5289f9b5cc12
[e7d722b]: https://github.com/zxfsee/afterburner/commit/e7d722b8adc5489ab285051abf199defe75f8fc1
[6e2e7b4]: https://github.com/zxfsee/afterburner/commit/6e2e7b4526cfd0107a69ece7cf6350ca2de67da7
[9eaf953]: https://github.com/zxfsee/afterburner/commit/9eaf953eef0e0ead61345789d18746476732e6a3
[ed9d7eb]: https://github.com/zxfsee/afterburner/commit/ed9d7eb574b891b65dbcd128ffe8991bbe015689
[7b0fb2f]: https://github.com/zxfsee/afterburner/commit/7b0fb2f38f60cb765bc1b807da46f8ba6cb9e0af
[edd955a]: https://github.com/zxfsee/afterburner/commit/edd955af7eb36b77025271a735e2adf858801a96
[0661586]: https://github.com/zxfsee/afterburner/commit/066158646ad1e67ea89aa81d8def48c6e7f11846
[9d787dc]: https://github.com/zxfsee/afterburner/commit/9d787dc63ffecc033156dcf0a0154af5667d0daa
[1f7a198]: https://github.com/zxfsee/afterburner/commit/1f7a1982f5b1790f6f398d265cba1bb533b1a23a
[bdfabea]: https://github.com/zxfsee/afterburner/commit/bdfabeaad4892d271cebea028bd43dc79595d836
[9f8e082]: https://github.com/zxfsee/afterburner/commit/9f8e0820f267d3a8e1402711d81ddf609183badf
[31cc68e]: https://github.com/zxfsee/afterburner/commit/31cc68e9805f2251630747bfcec95523c2e50a66
[93c5056]: https://github.com/zxfsee/afterburner/commit/93c50569f426ee55198848f18db7cfa8a6b39e59
[32d63a5]: https://github.com/zxfsee/afterburner/commit/32d63a5929b3699cd85c6856f2eda0d6864df70d
[11a4225]: https://github.com/zxfsee/afterburner/commit/11a42259710fab15d374aff4a2d3852eab03c8b7
[792f803]: https://github.com/zxfsee/afterburner/commit/792f8033a687760372dadd465e23822f543046d3
[91325b2]: https://github.com/zxfsee/afterburner/commit/91325b26be04b3bb39c2f4436fc51e1b5aee9929
[49cfe2f]: https://github.com/zxfsee/afterburner/commit/49cfe2fcef8b2be7fa061ff8f239579615eadaf4
[99144c7]: https://github.com/zxfsee/afterburner/commit/99144c782cf9fae1fff8b7f363849b21ae7e8cad
[a5defc4]: https://github.com/zxfsee/afterburner/commit/a5defc47556595580bc610af4997a4c8da90a7d3
[c4742ee]: https://github.com/zxfsee/afterburner/commit/c4742ee0ac99e305a76ad687bb4ea76c1bc141cf
[23bac29]: https://github.com/zxfsee/afterburner/commit/23bac2912cdf760ce3593a782f2ce46c3d3808cf
[1654689]: https://github.com/zxfsee/afterburner/commit/165468913f83ebeb9ea90092312b2ca8bf55d287
[f62c75c]: https://github.com/zxfsee/afterburner/commit/f62c75caf3ebd1d6ffad61e17e8c231960c7870f
[392e980]: https://github.com/zxfsee/afterburner/commit/392e980dfa9de2dc0ab672d915935d7fc6ebb0b9
[eafc870]: https://github.com/zxfsee/afterburner/commit/eafc87039534330fe0ace1f715994eba7273b8d3
[3d38394]: https://github.com/zxfsee/afterburner/commit/3d38394a4e15b205c4af3f7496fdb2622cf9c7f7
[1994c27]: https://github.com/zxfsee/afterburner/commit/1994c270491cb61755ba1c4c1c9063836be1f5d5
[8d7c2b9]: https://github.com/zxfsee/afterburner/commit/8d7c2b91135fc80d771ef1386c04103be39c8ba0
[d298d6d]: https://github.com/zxfsee/afterburner/commit/d298d6df5e306d296c669797e31165ef563879b2
[4b2be9e]: https://github.com/zxfsee/afterburner/commit/4b2be9e57855f37403e44b8417f3fa3ed8be1d2b
[14c151f]: https://github.com/zxfsee/afterburner/commit/14c151f1bef06a928e4da6745029f71dc4f02cf5
[15485b7]: https://github.com/zxfsee/afterburner/commit/15485b73b186df7168c65a3a3baab16c9e5a1c18
[c2ceccf]: https://github.com/zxfsee/afterburner/commit/c2ceccf37d05579ca42ebeabfd70d79f701747a2
[de024d0]: https://github.com/zxfsee/afterburner/commit/de024d0f0df48cfc3743067e3b8b679cba227a11
[1637413]: https://github.com/zxfsee/afterburner/commit/16374133b19f7876163661d5d267b06ddfeaf1b4
[4af8aac]: https://github.com/zxfsee/afterburner/commit/4af8aac8287be1c93f539d0c7b4b560493d0069f
[c289810]: https://github.com/zxfsee/afterburner/commit/c289810c74a231d2564912244e8c2dbbeaf39961
[5e9bc18]: https://github.com/zxfsee/afterburner/commit/5e9bc18426bbb8d2fd1064129f1d38a14738c1b3
[4932265]: https://github.com/zxfsee/afterburner/commit/4932265adea2a9b4df63f46d5995378d45e60d3a
[e0cfe47]: https://github.com/zxfsee/afterburner/commit/e0cfe4720924f5d2095e72d4268065d5ec1271a6
[1e507eb]: https://github.com/zxfsee/afterburner/commit/1e507eb8bde9a72374773f73b34a08fdce949b2c
[33640a8]: https://github.com/zxfsee/afterburner/commit/33640a89b78dacf73da656dbc6ff26b06858a36c
[129e859]: https://github.com/zxfsee/afterburner/commit/129e8594eedf1c11638f7585d1339621898a7a0b
[27002f3]: https://github.com/zxfsee/afterburner/commit/27002f34cfce060ab41cf67c55da73f48e771ae1
[31ce7d7]: https://github.com/zxfsee/afterburner/commit/31ce7d7f0a776cf47890cab7bf1c541d5d341259
[87abf6a]: https://github.com/zxfsee/afterburner/commit/87abf6ae5d25aa7704e8d81b23fad398c2e222c4
[656e2bb]: https://github.com/zxfsee/afterburner/commit/656e2bb3a869c78edc8d42624c2e7779eb315395
[de3db83]: https://github.com/zxfsee/afterburner/commit/de3db8329e449677c73a6d769442b35453a4a819
[1f06ca3]: https://github.com/zxfsee/afterburner/commit/1f06ca36f51d6c18a64999d7d00bd90f4598d79a
[a1124a2]: https://github.com/zxfsee/afterburner/commit/a1124a22cafec59f7963fbb9aafeb8e991c4f0aa
[71f73c7]: https://github.com/zxfsee/afterburner/commit/71f73c7082154a96f8c9f819a666b41e8a670f4f
[c932cb3]: https://github.com/zxfsee/afterburner/commit/c932cb3fa6c912feea7a1a2a953e782db167735d
[7c85ef0]: https://github.com/zxfsee/afterburner/commit/7c85ef0e780dd55d03e03d476a0daf63ff24b216
[485fbe1]: https://github.com/zxfsee/afterburner/commit/485fbe1748d464a18c7efd2fc2e0718cf3bad8ec
[56c9109]: https://github.com/zxfsee/afterburner/commit/56c9109a0c4fbaffde33ff99aea5a4fbaca395f7
[bfc546e]: https://github.com/zxfsee/afterburner/commit/bfc546e7eeab8dec3f0d7c3c4a288df4982cd0d0
[0e6a59a]: https://github.com/zxfsee/afterburner/commit/0e6a59a1a55e57397f125c62bc121f4d2852e6a4
[06fb9e4]: https://github.com/zxfsee/afterburner/commit/06fb9e40e2c1daf8640a6c7a8055c7e6f29facc6
[d8ce479]: https://github.com/zxfsee/afterburner/commit/d8ce479db4e9e71247f1b319c84ba561df39f837
[980f670]: https://github.com/zxfsee/afterburner/commit/980f670588afe494aca0e6abceba578fdc005078
[be1ae26]: https://github.com/zxfsee/afterburner/commit/be1ae269cb438c2c5395d813f0bdd82fcc9f9490
[344d77a]: https://github.com/zxfsee/afterburner/commit/344d77a408d758625da481b08834dbd36e06bc8c
[e693670]: https://github.com/zxfsee/afterburner/commit/e6936705c6cdb6dcbdcce690d1b450127481bf50
[c673585]: https://github.com/zxfsee/afterburner/commit/c673585026efe178465ffaa03cd74e19ba8f5306
[ee4732b]: https://github.com/zxfsee/afterburner/commit/ee4732bc2ecabeb076b1a09bacb7fb89fcd29538
[60b6fdc]: https://github.com/zxfsee/afterburner/commit/60b6fdc0ad904e74d890a9e9408724b8b819940d
[3b41fab]: https://github.com/zxfsee/afterburner/commit/3b41fab58f09c05414b6b154907823c5f571b556
[c361f52]: https://github.com/zxfsee/afterburner/commit/c361f52b9aa3cff2296fafe17621de713fc0e390
[1bd69b3]: https://github.com/zxfsee/afterburner/commit/1bd69b3a01dccd113be80b79848d2e2706913cbb
[863d786]: https://github.com/zxfsee/afterburner/commit/863d7861ada7e696c1c3d28f7b6b10b5985cd370
[12f5f55]: https://github.com/zxfsee/afterburner/commit/12f5f556ce33ad0577a6ec561e7254657cbd3151
[4080a84]: https://github.com/zxfsee/afterburner/commit/4080a84a8b150fc0196c45e60e7e4c000c0e542f
[2f15119]: https://github.com/zxfsee/afterburner/commit/2f15119a443c4792fb6178e1b1396e650430ebdd
[3fec302]: https://github.com/zxfsee/afterburner/commit/3fec302754f69b292026425623b1c42573e11706
[abea151]: https://github.com/zxfsee/afterburner/commit/abea151df3a57848f0bbad1cb658472405c83b5d
[5db0739]: https://github.com/zxfsee/afterburner/commit/5db0739fca708504bc058b5f25a16368f2f0a5a7
[4a29fa9]: https://github.com/zxfsee/afterburner/commit/4a29fa906ddda3471f0afff78990f33b6c05976f
[c37819b]: https://github.com/zxfsee/afterburner/commit/c37819b1ac53ebfbd27bdafc21856422ac1ac3e6
[8fee285]: https://github.com/zxfsee/afterburner/commit/8fee285de30597b400bd3adb7ecdcfbb6a161d3f
[9b3a4fa]: https://github.com/zxfsee/afterburner/commit/9b3a4fa34a72ba72f831bc972c0d600f150889f0
[695cfc0]: https://github.com/zxfsee/afterburner/commit/695cfc06ccbde952a1bf170d5563265ff3d55959
[e5dd680]: https://github.com/zxfsee/afterburner/commit/e5dd68084b0fd795afbdf44c298f7d76948abe36
[26bb73c]: https://github.com/zxfsee/afterburner/commit/26bb73cba538d13ff6d1609bae7072b9a22295c0
[8ad8913]: https://github.com/zxfsee/afterburner/commit/8ad8913cf6b92bede72afa3f5106486d6fb94c77
[59dc9a5]: https://github.com/zxfsee/afterburner/commit/59dc9a5bcc74332ac11356bebd8e5ddc0c943b76
[60b0aad]: https://github.com/zxfsee/afterburner/commit/60b0aad3848b996ee777b704e29e59dd963a0883
[65ad9f1]: https://github.com/zxfsee/afterburner/commit/65ad9f161fc1b93424f181f4da0c4c04cc11cb2f
[5fa5db1]: https://github.com/zxfsee/afterburner/commit/5fa5db1e196c30342edd06116c8b51b6079c2650
[4e148ff]: https://github.com/zxfsee/afterburner/commit/4e148ff655ca7cacf2d2c43297c9aef63b306429
[2343f36]: https://github.com/zxfsee/afterburner/commit/2343f36a1d2a9995ba1fccc4c34428bca0e20ded
[9850781]: https://github.com/zxfsee/afterburner/commit/98507819d5f0942b5f90ffe4a407e2a898616013
[ed6b43f]: https://github.com/zxfsee/afterburner/commit/ed6b43ff970a2a5bf8127ba126975843adad506d
[0d15881]: https://github.com/zxfsee/afterburner/commit/0d15881e338d94bb94ed745c0991d31b9e17f5c1
[d39df9d]: https://github.com/zxfsee/afterburner/commit/d39df9dc12a5a322cf8bef62368f53fd9ba4d529
[ddd9360]: https://github.com/zxfsee/afterburner/commit/ddd9360b6be8ad47a834005c35ff3997c08884fe
[619cbab]: https://github.com/zxfsee/afterburner/commit/619cbabf2906d66312113c816c6be36f23eea724
[fbeb94f]: https://github.com/zxfsee/afterburner/commit/fbeb94f77431fc15296801bc9898fe3628897e57
[bc4a279]: https://github.com/zxfsee/afterburner/commit/bc4a279a7e5e3a5fab731060fa52bf75f36d0c37
[50a60bb]: https://github.com/zxfsee/afterburner/commit/50a60bbc51051169114dc382d81d63e1048ad089
[6eac14b]: https://github.com/zxfsee/afterburner/commit/6eac14b40a13103824f51571378d309f54c6444a
[6cdcb37]: https://github.com/zxfsee/afterburner/commit/6cdcb377f2205b2c6cdb62aadb58e7bb6fb39605
[f9e57f7]: https://github.com/zxfsee/afterburner/commit/f9e57f737c35242a2feaea9d6c07fcabd062e686
[5db5756]: https://github.com/zxfsee/afterburner/commit/5db575622b7eb53b6472bbb6b07bd8131ad687ce
[c115fd4]: https://github.com/zxfsee/afterburner/commit/c115fd45c509fabb1c7b9d81e3377846bb391de0
[050cf2c]: https://github.com/zxfsee/afterburner/commit/050cf2c6f554dc2975f60ea3c4abd6456fce5d7e
[59f40cb]: https://github.com/zxfsee/afterburner/commit/59f40cb992e9a0269b759d4585a07ae73109b99d
[20f62ad]: https://github.com/zxfsee/afterburner/commit/20f62ad66884ec44821b044aebc4a8fd0ef534ad
[c04fd1a]: https://github.com/zxfsee/afterburner/commit/c04fd1ab8cc1c6a21a50d8afc3cd26804e6bf726
[ef07659]: https://github.com/zxfsee/afterburner/commit/ef076591c6834f4f672d83216652637be7337a8e
[2401197]: https://github.com/zxfsee/afterburner/commit/2401197ab08130ae08a01d722ad50b19167fa973
[c04eb70]: https://github.com/zxfsee/afterburner/commit/c04eb705c8cfbcc5ca0eff7b4ed4941ca7b90d4b
[a5b4077]: https://github.com/zxfsee/afterburner/commit/a5b4077dc9e15a9e319433b8ca3891372e743a1f
[f928ace]: https://github.com/zxfsee/afterburner/commit/f928ace788ac7907ac5c692e49e592e4c790ead7
[03127dd]: https://github.com/zxfsee/afterburner/commit/03127dd4ba4cb4a552ddedc5326f231d7a1a67e0
[d4d6948]: https://github.com/zxfsee/afterburner/commit/d4d694818915debdd71485a190668142fb104f2f
[40e195b]: https://github.com/zxfsee/afterburner/commit/40e195be1b0073d3e914ee25c3decafa163f33c5
[3eed571]: https://github.com/zxfsee/afterburner/commit/3eed571ba98d5b41f147a21afb53ab801c2b3742
[5023bd6]: https://github.com/zxfsee/afterburner/commit/5023bd6b64de1d066ef8acac365c53f9cc5df44a
[9630cf3]: https://github.com/zxfsee/afterburner/commit/9630cf32a7ab60b50a132e24b35ec1cf91848bd5
[005d1a2]: https://github.com/zxfsee/afterburner/commit/005d1a27246700d2ad556304b4eb5efca75f1a8c
[d314992]: https://github.com/zxfsee/afterburner/commit/d31499240df101d43278f47a3867e0b38609d4d6
[19b5f40]: https://github.com/zxfsee/afterburner/commit/19b5f40a5f09a2ff30f8339164e8db87004a02f0
[42dc08d]: https://github.com/zxfsee/afterburner/commit/42dc08d60251098e4f36010465512936bc6b0fb6
[3697ef0]: https://github.com/zxfsee/afterburner/commit/3697ef0c2bc2c8932f95d0f279456a8671734926
[e4338cf]: https://github.com/zxfsee/afterburner/commit/e4338cf08609abfdc6ef450280a7953cb3a4f3d9
[7171d6e]: https://github.com/zxfsee/afterburner/commit/7171d6ef597ee89d73924cf6190b44b3b6ab7c93
[189cac3]: https://github.com/zxfsee/afterburner/commit/189cac3a69861f4cd2422941a3de1f97220f1b41
[9f2826a]: https://github.com/zxfsee/afterburner/commit/9f2826a5b464cff1a6e021a40d728ef9da9404b5
[9d52b39]: https://github.com/zxfsee/afterburner/commit/9d52b39609d2a63e02765aa8037f16dae76c0386
[98f8645]: https://github.com/zxfsee/afterburner/commit/98f8645af7f7ebae5dae1ae4801e260bfe41693d
[3a2c802]: https://github.com/zxfsee/afterburner/commit/3a2c802d05c120599223c35b77bc499995a8dae2
[bf3ef7a]: https://github.com/zxfsee/afterburner/commit/bf3ef7a6f3713b4fe4294e8ac74b33fa118af20c
[bfb22c4]: https://github.com/zxfsee/afterburner/commit/bfb22c4059a753c8a5d899ab0ed373f8d71f8469
[23e165c]: https://github.com/zxfsee/afterburner/commit/23e165cf0dc2e8fb72bdd72151e36a4fef190309
[c7de99d]: https://github.com/zxfsee/afterburner/commit/c7de99d19cec5b41e09864375ed39b8aa907b922
[f2cba27]: https://github.com/zxfsee/afterburner/commit/f2cba27648484aa9fd7f8bb9d6bf1ef7ed75c551
[d6690bb]: https://github.com/zxfsee/afterburner/commit/d6690bb4587cd4c7d93b6e9bf45060e563096da1
[4eb3d60]: https://github.com/zxfsee/afterburner/commit/4eb3d60d75761e250c5c113f847ec463e2869f08
[29a1191]: https://github.com/zxfsee/afterburner/commit/29a1191630ca52d393383f6ff5e3f57ed1b9e3d1
[2747d3f]: https://github.com/zxfsee/afterburner/commit/2747d3f6ee51666e6abd2efd61b0b7c15c43d71f
[ceb3be4]: https://github.com/zxfsee/afterburner/commit/ceb3be4d785ab79865540aaf4980de225615c74f
[2c0aec5]: https://github.com/zxfsee/afterburner/commit/2c0aec51853e34669d75ce6e825933ed0d96b46b
[68ea9d9]: https://github.com/zxfsee/afterburner/commit/68ea9d93ac6a5133efb13f8534a214ca7f95693f
[1a2d92e]: https://github.com/zxfsee/afterburner/commit/1a2d92e346f43c51ab38ab326911aa505f54366b
[314da90]: https://github.com/zxfsee/afterburner/commit/314da90623414e9518cdc4986aa7d9d2fd73e9a8
[2c67074]: https://github.com/zxfsee/afterburner/commit/2c6707471c114826ab25a9f8a483f46f397a7f01
[fca8951]: https://github.com/zxfsee/afterburner/commit/fca8951cfb69176346e0eb5927434a41612a9f83
[d1f1d81]: https://github.com/zxfsee/afterburner/commit/d1f1d81f452c3288a051e6fc298ccd9312b02d7c
[89218b5]: https://github.com/zxfsee/afterburner/commit/89218b5d5f4256d494e7d40dfaee91571fdd87e8
[569854e]: https://github.com/zxfsee/afterburner/commit/569854edfa8a5797e9d696fb300994d38c4e9ef2
[1e98527]: https://github.com/zxfsee/afterburner/commit/1e985276c8eb1f367f203788462a6af663053349
[f7b9bd7]: https://github.com/zxfsee/afterburner/commit/f7b9bd75671e52a59eebbcd1a7032cca178287b3
[83ec760]: https://github.com/zxfsee/afterburner/commit/83ec7609c6e1f5057388122d0209cc2870561cf9
[f756482]: https://github.com/zxfsee/afterburner/commit/f756482552519b8556ac0e03c75535f50f817272
[3194669]: https://github.com/zxfsee/afterburner/commit/3194669be06e05e7ddafe09fc5e11ad4e2477fd3
[14cbc70]: https://github.com/zxfsee/afterburner/commit/14cbc70e1da691a456f21f3b3a1b7fce0a4b3e55
[9b7d001]: https://github.com/zxfsee/afterburner/commit/9b7d001f93f97603a0c0c63506dade6055f30b54
[c31c94b]: https://github.com/zxfsee/afterburner/commit/c31c94b4e6004b04680f60360c6d6b75ccab326f
[8450451]: https://github.com/zxfsee/afterburner/commit/845045183b22c0e9d0c6bc4ded043550a51f237b
[c4446a4]: https://github.com/zxfsee/afterburner/commit/c4446a4ecbbb49c2c596f5daad6a4e529c6a989d
[408ee10]: https://github.com/zxfsee/afterburner/commit/408ee10bd8f441bbe91ca7d3d10b8efd56ba67d8
[c9f30dc]: https://github.com/zxfsee/afterburner/commit/c9f30dc86b6e67989ecb23d6f7df875f236b867a
[40ed6f7]: https://github.com/zxfsee/afterburner/commit/40ed6f7c20195e3d1f8eff720f6d46c97f364e2a
[3e35510]: https://github.com/zxfsee/afterburner/commit/3e35510ead2966d3ea18840e7569f51622a91fc2
[df645fa]: https://github.com/zxfsee/afterburner/commit/df645faa785356988c732e3854ad7c49a5a230f7
[8f3b5b9]: https://github.com/zxfsee/afterburner/commit/8f3b5b9dddd73d595046103da4b7daf813e8c6ff
[6ef2373]: https://github.com/zxfsee/afterburner/commit/6ef237382893024efea35ab594896d9bf71ef6d1
[d7aa627]: https://github.com/zxfsee/afterburner/commit/d7aa627c02adaa14194b5b8fb96dc3dcdf521f9d
[05de1f7]: https://github.com/zxfsee/afterburner/commit/05de1f74d7b671cfd7aaace9ea1feac8475d02a0
[1941647]: https://github.com/zxfsee/afterburner/commit/1941647a5e28f92ef6af2b1705ba9248f4acd4dc
[2feefac]: https://github.com/zxfsee/afterburner/commit/2feefac8da05d268aaba020e16c7b2a2c270c945
[7ae9c61]: https://github.com/zxfsee/afterburner/commit/7ae9c6101abcead9861eda648e7a362659044e09
[cad7ef4]: https://github.com/zxfsee/afterburner/commit/cad7ef43aadd678bb38644b36756b34c2105f3a4
[b36e66f]: https://github.com/zxfsee/afterburner/commit/b36e66fd079f9c5f5db0b666c0e190d45c7a1626
[2191c50]: https://github.com/zxfsee/afterburner/commit/2191c50a8469f55ca96552d52e42b007a97c46a2
[42cb8d7]: https://github.com/zxfsee/afterburner/commit/42cb8d7a4184497b22ece2c1f96d2c431d572e12
[502a747]: https://github.com/zxfsee/afterburner/commit/502a747dddf0eea702084789d1450b319e31dcfb
[7cb56f6]: https://github.com/zxfsee/afterburner/commit/7cb56f6316e3d74fd499b43313896037f82c578b
[e98465b]: https://github.com/zxfsee/afterburner/commit/e98465bd230284ee338b3e4096e77e5980ea3c74
[a7b901e]: https://github.com/zxfsee/afterburner/commit/a7b901eaca1e700ece16cc11ca6f3e9d9b6610eb
[b1fe569]: https://github.com/zxfsee/afterburner/commit/b1fe5696a5e056d0229068e99e2eaf048c264497
[351ec70]: https://github.com/zxfsee/afterburner/commit/351ec7059fa07da883b1c3415ab00dc6c9b25fde
[1a3ca9c]: https://github.com/zxfsee/afterburner/commit/1a3ca9cddd3ab8dabefccf82d5d2ca3e31b40c5e
[376e8bf]: https://github.com/zxfsee/afterburner/commit/376e8bf0be0d71314933191e0874bbbe72a6503a
[d954a72]: https://github.com/zxfsee/afterburner/commit/d954a720a8997de1aeb69ffc00bc199a7073c221
[3a36714]: https://github.com/zxfsee/afterburner/commit/3a367142f302a8ba7a1888e9fc41385da81468c0
[e1b3201]: https://github.com/zxfsee/afterburner/commit/e1b32010417608da07e58b4cc47d4a94a0a8f372
[b7c84ad]: https://github.com/zxfsee/afterburner/commit/b7c84ad58e10d4de7806c0a94ef910ec988e6e13
[9bf1597]: https://github.com/zxfsee/afterburner/commit/9bf15976eae4ab9c540ea2418bd07e463f1a72f0
[0dd6ed8]: https://github.com/zxfsee/afterburner/commit/0dd6ed82ed50f59a58847730ab58608296f352b9
[66cd1bc]: https://github.com/zxfsee/afterburner/commit/66cd1bc40e470be4007f486685ebdb9fe7dcd59b
[30c0657]: https://github.com/zxfsee/afterburner/commit/30c06576bb0d0d51f7e15b1a6ac221c3e58dcaf1
[6f21d24]: https://github.com/zxfsee/afterburner/commit/6f21d249741cb876a558a06824b5f203f33f332a
[8745849]: https://github.com/zxfsee/afterburner/commit/8745849dd23af263be78a0bfa95e664a6daac3da
[1c05b5e]: https://github.com/zxfsee/afterburner/commit/1c05b5eecaf7b24c119c7529e00a0d6431fe4c5c
[60f100c]: https://github.com/zxfsee/afterburner/commit/60f100c3aba9fd026dcc58a6adc31c76047f6c53
[f06289e]: https://github.com/zxfsee/afterburner/commit/f06289ee2d5234978adf75d8f13f866e271905e7
[a80eaba]: https://github.com/zxfsee/afterburner/commit/a80eaba967e33ec517e67685ceab1412e064e4d7
[472db72]: https://github.com/zxfsee/afterburner/commit/472db720f1c8ba8dda5e8e8a2594cc81244c189e
[2c75e73]: https://github.com/zxfsee/afterburner/commit/2c75e73c82768627ad298bf46ae1fc5836a6fa9c
[19f7a23]: https://github.com/zxfsee/afterburner/commit/19f7a2315553e49763bc021c8128c600a2a29935
[904d2a3]: https://github.com/zxfsee/afterburner/commit/904d2a333ca085c16f2c70f3cde8ee3e5253dc56
[e438735]: https://github.com/zxfsee/afterburner/commit/e438735be07362808840212b016ca1e5b666bebb
[2683e90]: https://github.com/zxfsee/afterburner/commit/2683e900307433f945adad427624a916974102d3
[803d51b]: https://github.com/zxfsee/afterburner/commit/803d51b7250e9dc3115963d8c44f78e10544457b
[d34a25e]: https://github.com/zxfsee/afterburner/commit/d34a25e82ce5a368be87c35abc27b1fed6cddbe3
[3bf2461]: https://github.com/zxfsee/afterburner/commit/3bf24611d04ded57e7a7fe29f1740f5d58c7718d
[604d90c]: https://github.com/zxfsee/afterburner/commit/604d90cc981cd88f22c78052ac81b3b1d8ccf6bc
[f9cef32]: https://github.com/zxfsee/afterburner/commit/f9cef32934afa2dd3c3a70fdcff24ec771829583
[152b064]: https://github.com/zxfsee/afterburner/commit/152b064365e67d4c5c8e5fc93ea3d99eff9524fe
[541f78f]: https://github.com/zxfsee/afterburner/commit/541f78f36c16776de2cd9c932978f29958d459d9
[84f9f99]: https://github.com/zxfsee/afterburner/commit/84f9f992908c421b6796e76e7fa02c1d292bcf0b
[bcc47f6]: https://github.com/zxfsee/afterburner/commit/bcc47f606089f8feb9f935445ecb6497972059af
[4727bef]: https://github.com/zxfsee/afterburner/commit/4727bef3d7b7e6d547eae14fb5f9b89510838b57
[b81622c]: https://github.com/zxfsee/afterburner/commit/b81622cc82686b42664bd9bd48a3d8a717dabbd7
[82b66cd]: https://github.com/zxfsee/afterburner/commit/82b66cd8cd5ea0893ee9bb264509068059a0f63f
[dfe389d]: https://github.com/zxfsee/afterburner/commit/dfe389d5a1508a5c58864f69f39ae208d6e01693
[0eab0a8]: https://github.com/zxfsee/afterburner/commit/0eab0a875733f01385932b93ae5fe0dbb7048eb4
[34a4f92]: https://github.com/zxfsee/afterburner/commit/34a4f92eada58c6255271707cd00eff51cb05e58
[4cbe5af]: https://github.com/zxfsee/afterburner/commit/4cbe5afbb0188474d70fc3395ac5323e2017bcd7
[c0457bf]: https://github.com/zxfsee/afterburner/commit/c0457bfa5c42de02593f4f9d1d7377a058897392
[f266fde]: https://github.com/zxfsee/afterburner/commit/f266fde66e37e497171fc9a444e6b707f028a10b
[4cf08bb]: https://github.com/zxfsee/afterburner/commit/4cf08bb74203c6bd4a32a24ef0a0e84070268eb4
[3ce5773]: https://github.com/zxfsee/afterburner/commit/3ce577305328491acd55990cbc0f68902394a0d7
[0d85bb3]: https://github.com/zxfsee/afterburner/commit/0d85bb3fd260348c2c0be36e141d00e047fbe06a
[49a07ce]: https://github.com/zxfsee/afterburner/commit/49a07cef8898fce825390389cbb62f051c0e9a72
[018bd03]: https://github.com/zxfsee/afterburner/commit/018bd0342fda941f58ac82980b71fd6dafeb5cea
[bab847b]: https://github.com/zxfsee/afterburner/commit/bab847bc44fa931d0eab341e9adf3e663bf261c7
[c4a8bab]: https://github.com/zxfsee/afterburner/commit/c4a8bab72184ae4ec2293ac921c631d12a0647cf
[18bc297]: https://github.com/zxfsee/afterburner/commit/18bc2976436cbc911c71b83690ad2a304c0211b1
[166dce2]: https://github.com/zxfsee/afterburner/commit/166dce27a157abac0fab3400e174cd0daff0db5f
[3cfb2c1]: https://github.com/zxfsee/afterburner/commit/3cfb2c1175d5bbb7aea9bda61405b75f1e191825
[58b003a]: https://github.com/zxfsee/afterburner/commit/58b003a6a64fcf711a6cd653f91ba0debea3f5ba
[cde8cd9]: https://github.com/zxfsee/afterburner/commit/cde8cd97c6ee349d56339bda74add2ebc422cf09
[24be72e]: https://github.com/zxfsee/afterburner/commit/24be72e2cadf278e089db8266b00d936c6812f6a
[7461b93]: https://github.com/zxfsee/afterburner/commit/7461b93a5919284cdc4d14a3a25bdb11f30eb55f
[80d02e9]: https://github.com/zxfsee/afterburner/commit/80d02e9f90cf2b2947e5db7ab75c9112d1184a17
[9bbf1a3]: https://github.com/zxfsee/afterburner/commit/9bbf1a334ac6d03384e18c9af5815aee87d48e57
[aa4c6e7]: https://github.com/zxfsee/afterburner/commit/aa4c6e7b16d83aa80f1a66397962477acfa0a99b
[cda7a1c]: https://github.com/zxfsee/afterburner/commit/cda7a1cbdb6bd223fc9031975a857bf27b148ed1
[6b0da75]: https://github.com/zxfsee/afterburner/commit/6b0da7533da9bd4f1ffb3a7d74fc9bad28afdb50
[e5f4a44]: https://github.com/zxfsee/afterburner/commit/e5f4a44b60debbd770853dad127b34f049263989
[555dd06]: https://github.com/zxfsee/afterburner/commit/555dd061a28ad3a0576fbd42e47ae383ace1915e
[31c5790]: https://github.com/zxfsee/afterburner/commit/31c5790aeae617bef7d859d88ac6fe513e13d3d1
[356ef50]: https://github.com/zxfsee/afterburner/commit/356ef5048657439d05b3acbe0872f6481696d980
[d92078e]: https://github.com/zxfsee/afterburner/commit/d92078ee20e87e4ccfd533661e2cf4ba98f1d3fe
[f85edf5]: https://github.com/zxfsee/afterburner/commit/f85edf5efd8dd657035e43cdb9d866e0f2ca67d3
[5b81d02]: https://github.com/zxfsee/afterburner/commit/5b81d027fe553c8c2a205e7cb73a7e96221463a0
[eafeb6a]: https://github.com/zxfsee/afterburner/commit/eafeb6a62ba3b83d7935e73fb0c90d90f5571020
[c1b43b7]: https://github.com/zxfsee/afterburner/commit/c1b43b76f3515bdc9cf984d2015aed27315edf39
[2ba8ec7]: https://github.com/zxfsee/afterburner/commit/2ba8ec767263491503619f879b5d8eb99ad86982
[c18a0d8]: https://github.com/zxfsee/afterburner/commit/c18a0d8714611f08bf4ddb265c2df8be189f8637
[3dcf04e]: https://github.com/zxfsee/afterburner/commit/3dcf04eb384a355bd454f586f518353b5078bbdb
[0565323]: https://github.com/zxfsee/afterburner/commit/0565323a8eb97f5a2edddcd4bfa3d265c0d6f1f5
[ff3dd01]: https://github.com/zxfsee/afterburner/commit/ff3dd0165788a9786c9dbc6b17546fb5319d3b8e
[4a2e1c3]: https://github.com/zxfsee/afterburner/commit/4a2e1c33516fea28ecba17b976e82b5671c62054
[8370836]: https://github.com/zxfsee/afterburner/commit/83708360b10a2ad8127c8728cbeaea7c3a7b877f
[2a95e0a]: https://github.com/zxfsee/afterburner/commit/2a95e0a809d9968f1e02f87f9668df00c6819dc5
[0953edc]: https://github.com/zxfsee/afterburner/commit/0953edcaa92a8c1700c3e256a8f3f6caeba069cd
[9491b99]: https://github.com/zxfsee/afterburner/commit/9491b997a51dd0e97dc1f80063e7545d7c116069
[e580407]: https://github.com/zxfsee/afterburner/commit/e580407b674f1b750ae39ec855b928fd694b6e11
[6b775f1]: https://github.com/zxfsee/afterburner/commit/6b775f15c15fc2b53695519ef65823c54624e830
[c559afe]: https://github.com/zxfsee/afterburner/commit/c559afe607ae18e34ac852f95334a7fe837b10eb
[7d1746e]: https://github.com/zxfsee/afterburner/commit/7d1746ef627c3c29010627397dec3a7aea792ab0
[d85bfe9]: https://github.com/zxfsee/afterburner/commit/d85bfe9459e80b9458305449a2b454aa5fa0ba0f
[2aeda96]: https://github.com/zxfsee/afterburner/commit/2aeda9632456a45f3638ccf09d34879b823cea81
[08e6df7]: https://github.com/zxfsee/afterburner/commit/08e6df75fafa179c2282a2a38ee511c48b8cae5c
[bf41a4f]: https://github.com/zxfsee/afterburner/commit/bf41a4facd43d34502596bf373b3071373a211f8
[1e579e0]: https://github.com/zxfsee/afterburner/commit/1e579e0644931ad8993cd721d9eb780c650f5c77
[5a04821]: https://github.com/zxfsee/afterburner/commit/5a048211270545527360d53a2793a23e37e452da
[ce76111]: https://github.com/zxfsee/afterburner/commit/ce761112632dea236a732a483f52dcf1da0ccf73
[f069e49]: https://github.com/zxfsee/afterburner/commit/f069e49ec49c12b6e576823279f288893e879f4b
[99a14e9]: https://github.com/zxfsee/afterburner/commit/99a14e921f4d6f74b3ed1da799ae3e8748febecd
[aff116e]: https://github.com/zxfsee/afterburner/commit/aff116e1ab34aee106bc7f9ab77fe1d82b94a5d5
[529561e]: https://github.com/zxfsee/afterburner/commit/529561eac3b234bf3cb4c6452f9f1b8d25bd5230
[166a3ee]: https://github.com/zxfsee/afterburner/commit/166a3eeb77d006eddc1f2b4e0946f034522dd88d
[e57b234]: https://github.com/zxfsee/afterburner/commit/e57b234f2ff567d6c7164a8193df8d8328b0cc3c
[557d69e]: https://github.com/zxfsee/afterburner/commit/557d69ee967aa819d3c3d02deca146f9face7447
[c96ed02]: https://github.com/zxfsee/afterburner/commit/c96ed023634991b571e296557350923fd026510d
[4676420]: https://github.com/zxfsee/afterburner/commit/46764201b0284ee5102187d8d14f3a0f0d55a39f
[7e4d9f3]: https://github.com/zxfsee/afterburner/commit/7e4d9f31fc6b7204f1cbef9d2ff56d13361fa7a3
[49025c5]: https://github.com/zxfsee/afterburner/commit/49025c520c0d183b7042891dd0dfc6f76557555a
[47006c7]: https://github.com/zxfsee/afterburner/commit/47006c768085e81fa636b6349446ca6b3f0c177c
[15bd378]: https://github.com/zxfsee/afterburner/commit/15bd3788b7326b861e56e62c8fde034cbe047fc6
[73e0288]: https://github.com/zxfsee/afterburner/commit/73e0288f223f87736fe96f845cfe95e7eb44a671
[d4952c4]: https://github.com/zxfsee/afterburner/commit/d4952c49fe308cc9e6f26db14c373dea9948dd8a
[6f915fc]: https://github.com/zxfsee/afterburner/commit/6f915fcea7b0f2130569725bcfb05c6259149298
[ba78f03]: https://github.com/zxfsee/afterburner/commit/ba78f03efd98cc66c554de34e07da78d30596c91
[6796657]: https://github.com/zxfsee/afterburner/commit/6796657f91764cab96249e77aafef7924df36776
[a1a8813]: https://github.com/zxfsee/afterburner/commit/a1a8813b17dcc1184f23bd0bc95a67010fcc5416
[086cfe6]: https://github.com/zxfsee/afterburner/commit/086cfe6a41da9e10f2f4044b757678e7114b445b
[505f2b6]: https://github.com/zxfsee/afterburner/commit/505f2b6e51d9802f30e72216c9be469f6d8f6c46
[9b7309d]: https://github.com/zxfsee/afterburner/commit/9b7309d2bd5ec484fc4d494ee6d388786d85ad75
[54d2db1]: https://github.com/zxfsee/afterburner/commit/54d2db14c76ed5f3553419b4455273c83cb7b9db
[3d0fd7b]: https://github.com/zxfsee/afterburner/commit/3d0fd7b50cf3053ed3ca595b359143be01f4eb61
[24a56d2]: https://github.com/zxfsee/afterburner/commit/24a56d29709b368ae6ee2c1e96974ffdd45af8c8
[775e493]: https://github.com/zxfsee/afterburner/commit/775e493af7362ec13d81d5e7094a4a149e5d5617
[bc6bae6]: https://github.com/zxfsee/afterburner/commit/bc6bae6c9f63c712f89f52f9938acc0af74abe36
[bfb331e]: https://github.com/zxfsee/afterburner/commit/bfb331efa2e64b6050f382e198b870e06986bc40
[71d1337]: https://github.com/zxfsee/afterburner/commit/71d133729d464b98b610bc678a6fe1ded86799bc
[be1e298]: https://github.com/zxfsee/afterburner/commit/be1e29887076810e5b18f8b3bd7bfe1a82faad66
[f708791]: https://github.com/zxfsee/afterburner/commit/f708791f5b88e147d6c2f5299ad81c91b39bbfeb
[ffba2cd]: https://github.com/zxfsee/afterburner/commit/ffba2cd348d078ca6c6ae60aa032e14dbb22197c
[fa920fa]: https://github.com/zxfsee/afterburner/commit/fa920faed835d76fe9eaddda803558872a78dadf
[0ffc865]: https://github.com/zxfsee/afterburner/commit/0ffc86536b7645966c202877b875777088c66644
[be2d50b]: https://github.com/zxfsee/afterburner/commit/be2d50b8c81a65ccf5b391682f51fe6b503c8fdf
[ee3289f]: https://github.com/zxfsee/afterburner/commit/ee3289fa9c34010fcd2f105dc32d5dc439061cd2
[aba64b4]: https://github.com/zxfsee/afterburner/commit/aba64b4e982f2c67c4450e2d347507a2bd2124a6
[9f6c607]: https://github.com/zxfsee/afterburner/commit/9f6c6071049a675181d5d091136cd24a528ca1c1
[8e73da5]: https://github.com/zxfsee/afterburner/commit/8e73da5d2be7459a77be5e29ed261af7dc08413f
[6fc6863]: https://github.com/zxfsee/afterburner/commit/6fc6863fe2d9d267f3c90f4928989385cf2d01d5
[04d350e]: https://github.com/zxfsee/afterburner/commit/04d350e49d6645db68811dd72b2b0c572c1639e3
[94b547b]: https://github.com/zxfsee/afterburner/commit/94b547bf0dc8c0a0d995dded3a9218a3bcb61777
[eb36f20]: https://github.com/zxfsee/afterburner/commit/eb36f20c7924f9feeaadf37e83053fab7e1db4b7
[9142545]: https://github.com/zxfsee/afterburner/commit/9142545842272cf51078bd135dae8d7d9d201537
[abca69b]: https://github.com/zxfsee/afterburner/commit/abca69b3b7f0edd296a6ba2116b4b5434a6344a6
[2427295]: https://github.com/zxfsee/afterburner/commit/24272955be4433f9e78734804e2f112d40eb128b
[08a6253]: https://github.com/zxfsee/afterburner/commit/08a625329b07c63a3a9241740eb3a9088d159937
[3e560de]: https://github.com/zxfsee/afterburner/commit/3e560de231f3f3ddb6d901fd9cfcbfadf2b4a423
[f2302e4]: https://github.com/zxfsee/afterburner/commit/f2302e49fc7e1c81273e9f46ea4806b26f6c8640
[fd46fc0]: https://github.com/zxfsee/afterburner/commit/fd46fc0304cf51a12e732c858920385ddb7d2000
[e84b3f6]: https://github.com/zxfsee/afterburner/commit/e84b3f68a3fdfe4afeab8ea630e3d5b8b4963d3f
[f2f12f0]: https://github.com/zxfsee/afterburner/commit/f2f12f081aa2fef03317950bcb8e733d1ce78ce1
[af0da7f]: https://github.com/zxfsee/afterburner/commit/af0da7fc312434fd932595636a729ef36751f1f3
[aec7d16]: https://github.com/zxfsee/afterburner/commit/aec7d16b0213b5ccf43fbf583db74f594eb76a4f
[70d30b5]: https://github.com/zxfsee/afterburner/commit/70d30b5f2afca0d09f56e4e67f4f398d7ca47376
[4b29939]: https://github.com/zxfsee/afterburner/commit/4b29939c8a93d2f2519afa91c118880cbca13aa6
[865714b]: https://github.com/zxfsee/afterburner/commit/865714b9b70906d9673286e0e7adb101308e9200
[f85091a]: https://github.com/zxfsee/afterburner/commit/f85091a92b8042b7f1cb8de336d40c4b53abc6f5
[a003f63]: https://github.com/zxfsee/afterburner/commit/a003f6335b6c18f6613612c1ae7763a5438e3a41
[74ee370]: https://github.com/zxfsee/afterburner/commit/74ee3704d37efdcba54303c98e4ad3fb6981bb68
[09a0bb5]: https://github.com/zxfsee/afterburner/commit/09a0bb5eea95d7ab87e694714779196faed84a3d
[3c6b0c1]: https://github.com/zxfsee/afterburner/commit/3c6b0c1818d98067f2a9c310c9d1da2cb194fdf7
[0d135f5]: https://github.com/zxfsee/afterburner/commit/0d135f5e56dd679475fa60112444bd2256ba3e2c
[8274014]: https://github.com/zxfsee/afterburner/commit/82740141cac8b778685153d3aa0a5335735a9537
[0d7b0b9]: https://github.com/zxfsee/afterburner/commit/0d7b0b9e00b515650f479e5438eb97cca6bdf1af
[92d7a4f]: https://github.com/zxfsee/afterburner/commit/92d7a4fdaeb40c598fe4c794eeae47fb6416372b
[0041774]: https://github.com/zxfsee/afterburner/commit/0041774a447ca7d57f1fab13c5c75b75b35196bd
[31c3191]: https://github.com/zxfsee/afterburner/commit/31c3191fd9beaea68a22f94888c54989243ba30d
[7545042]: https://github.com/zxfsee/afterburner/commit/75450421c47a205a6bedf4bc0f82b5ebc5ddc50a
[1405642]: https://github.com/zxfsee/afterburner/commit/1405642c24dbbf0d6994e789c841d3484b4294f5
[758d04a]: https://github.com/zxfsee/afterburner/commit/758d04a9a503c241532822d22c4e3561b3a442ec
[c2b1fe0]: https://github.com/zxfsee/afterburner/commit/c2b1fe0f3d111bf8d5c7ac615ced042febe9737b
[28997b0]: https://github.com/zxfsee/afterburner/commit/28997b023e3e5f493e94ad6b1f0ef11493def212
[c3b94ee]: https://github.com/zxfsee/afterburner/commit/c3b94eebe52c4f91829f3f14a38fb57c6b30532d
[cc694ec]: https://github.com/zxfsee/afterburner/commit/cc694ec5ff1478e253ba90c1e8d7a81185ae61c5
[d5357e6]: https://github.com/zxfsee/afterburner/commit/d5357e6ec3d4deda49c0420a7c13009cd9a28b54
[6c0950a]: https://github.com/zxfsee/afterburner/commit/6c0950a162c7a3a76adbf15b76411fcfd40f54e9
[3595c8b]: https://github.com/zxfsee/afterburner/commit/3595c8ba7e269cb3ec1161baeb8b3f9adb524489
[bde6cc6]: https://github.com/zxfsee/afterburner/commit/bde6cc6c9c32b39c43fe80f9afcba1d82ef2b2c0
[fa40090]: https://github.com/zxfsee/afterburner/commit/fa40090f0fc440d5a54b85c87e2e78897b7d6eb1
[79a7c2a]: https://github.com/zxfsee/afterburner/commit/79a7c2ae33be36a96c2d74c00f72fd2244bb0089
[23caa0d]: https://github.com/zxfsee/afterburner/commit/23caa0dfb49c7f7fcfb687e9d7b4cee9122f4a79
[a58cf13]: https://github.com/zxfsee/afterburner/commit/a58cf13dcb337529cb2458a5ee02090db58dd667
[e1c7f4c]: https://github.com/zxfsee/afterburner/commit/e1c7f4ce81ee67d3d0dcd8e336e51b83e409f420
[2b72984]: https://github.com/zxfsee/afterburner/commit/2b729841e54656b60e2d08db9a1e7a3c23278b25
[ab055bc]: https://github.com/zxfsee/afterburner/commit/ab055bc65efb116a591c470dadf61623fa47e418
[2317a52]: https://github.com/zxfsee/afterburner/commit/2317a52fceb548bff33e15857790d10d49fc6271
[0393cac]: https://github.com/zxfsee/afterburner/commit/0393cacc17a8c101c7a6446176b9af373948253e
[7051100]: https://github.com/zxfsee/afterburner/commit/7051100dfdb2a9122dc99ea09c8437c60e821338
[c14760c]: https://github.com/zxfsee/afterburner/commit/c14760c1c6e12b44f8220af4e72c2554d2d3ecac
[5f066b3]: https://github.com/zxfsee/afterburner/commit/5f066b32b4eb353ac6d0133012f39489ff1aaf6b
[0227edb]: https://github.com/zxfsee/afterburner/commit/0227edbf78e46b693809069ced4ad510ec2df518
[ce83478]: https://github.com/zxfsee/afterburner/commit/ce83478899fc38d7810f22c41f142a68a903031b
[727fe00]: https://github.com/zxfsee/afterburner/commit/727fe00de35ac958e4c00c8cf244244f111875d1
[a81ecea]: https://github.com/zxfsee/afterburner/commit/a81ecea0fd59a9a4f02aba885d8d573a9460ce86
[9251585]: https://github.com/zxfsee/afterburner/commit/9251585fffe42b76f2c0eee1b260157c6b6c3059
[35c9190]: https://github.com/zxfsee/afterburner/commit/35c9190abf6f2f48be92109cfb34f34c764cb413
[6b00de2]: https://github.com/zxfsee/afterburner/commit/6b00de21236f8c400d4daa2318350508ba5684f7
[24b6d55]: https://github.com/zxfsee/afterburner/commit/24b6d55a5e5f801f1b1929c6fe76e13be4727e5c
[c07c013]: https://github.com/zxfsee/afterburner/commit/c07c0134d8f639a0f642dc7657124c822f9d7d2b
[346d83f]: https://github.com/zxfsee/afterburner/commit/346d83f74e2676d90cff3bf1f391451e45f189c2
[9d057e4]: https://github.com/zxfsee/afterburner/commit/9d057e44910d30e2b5c1449915472bf15d8e14da
[bfa29b7]: https://github.com/zxfsee/afterburner/commit/bfa29b72e1679ac001e6ae3f6825e51eaa3dd667
[332ef34]: https://github.com/zxfsee/afterburner/commit/332ef34f230bab29b4d6d378cbcdc0d7397c8f61
[a272f78]: https://github.com/zxfsee/afterburner/commit/a272f78a4968b6b759defab3a3f5673b075f02a3
[bacc087]: https://github.com/zxfsee/afterburner/commit/bacc087efbb65354d88d2d2861fcedf08dd61cc5
[255f741]: https://github.com/zxfsee/afterburner/commit/255f74117acc7fefda18ee095eb8a1eaaf2e326a
[5e02bcf]: https://github.com/zxfsee/afterburner/commit/5e02bcf9bc0bd2b433972856988495cc18a27fb6
[44268ce]: https://github.com/zxfsee/afterburner/commit/44268ce1adb8531ca41a9c909659ec0308f620a9
[503125b]: https://github.com/zxfsee/afterburner/commit/503125bcca4c1b63e9911eab286e7728c3dedc1f
[7c6d774]: https://github.com/zxfsee/afterburner/commit/7c6d7741e64c802eb2ac535591f43ad324926602
[dc2c85b]: https://github.com/zxfsee/afterburner/commit/dc2c85b7686110018158f002cd6dffa708f4a885
[0201ff8]: https://github.com/zxfsee/afterburner/commit/0201ff8d0987f550bdde312d013c8231a8c3b417
[b51485e]: https://github.com/zxfsee/afterburner/commit/b51485e96502552130bc7f8d424007cfc2a0ecde
[a2ded8a]: https://github.com/zxfsee/afterburner/commit/a2ded8a9200b3f102ecf7eb9339fbbf5088969e0
[7988a65]: https://github.com/zxfsee/afterburner/commit/7988a650cdc1cdbf27e8cc105103f6f1ea623b63
[0f65069]: https://github.com/zxfsee/afterburner/commit/0f65069294874c9ddb107ca6755aedf3e4c66722
[bd5cf4e]: https://github.com/zxfsee/afterburner/commit/bd5cf4e55a52a35bfc623add3086a28b51ee9c9f
[88ed206]: https://github.com/zxfsee/afterburner/commit/88ed206e2841c5b8be0909d16c9b51e70269f337
[c58c147]: https://github.com/zxfsee/afterburner/commit/c58c14738f31b2804bb376a9b36c4c99f33e07ab
[da8dcf7]: https://github.com/zxfsee/afterburner/commit/da8dcf72baff9e0cf8f517f00c1ec0e49d983305
[1052c29]: https://github.com/zxfsee/afterburner/commit/1052c29e6184abfb129c7fb765449e81d4b963f6
[7458414]: https://github.com/zxfsee/afterburner/commit/7458414e7f827cd3683e366f2e98c3c1ae376fe4
[db892d8]: https://github.com/zxfsee/afterburner/commit/db892d86b8766df3aaffa94c92cd8da7bc6edc0b
[18966f5]: https://github.com/zxfsee/afterburner/commit/18966f5d701654c87cc6a71b8115c44ecff41d0e
[35b02f7]: https://github.com/zxfsee/afterburner/commit/35b02f7121e0f53188df257632d3dd44a0d55bdd
[376bdce]: https://github.com/zxfsee/afterburner/commit/376bdce882293716f207b9d10c3c7518ddd68f51
[295188a]: https://github.com/zxfsee/afterburner/commit/295188ace35ba0711b11b04ae3b0333623f1e934
[e4b39c5]: https://github.com/zxfsee/afterburner/commit/e4b39c5f74aed413701f40adc0f2670568f1421a
[cf5bfb9]: https://github.com/zxfsee/afterburner/commit/cf5bfb9ce2aaee52789f7751a890291eba6053fb
[70aa9c9]: https://github.com/zxfsee/afterburner/commit/70aa9c92226cd207c4c43678772f67d91b7e4bd3
[72c9279]: https://github.com/zxfsee/afterburner/commit/72c92796929719fe5e0cfa5057c3134713243c35
[b60cf0d]: https://github.com/zxfsee/afterburner/commit/b60cf0de8f5b28138462ba19f516407158bf84f4
[776244d]: https://github.com/zxfsee/afterburner/commit/776244d2d4c80f63543085359e67f3b822311840
[bbb287e]: https://github.com/zxfsee/afterburner/commit/bbb287e88b4ee95c8830e039124e7d13eb85de12
[9a31b3a]: https://github.com/zxfsee/afterburner/commit/9a31b3ad979e07999b2739b5babd8fa2042a60d4
[5c83b8f]: https://github.com/zxfsee/afterburner/commit/5c83b8f6f97304b661d8cc395a1bb9130e40330c
[2b55941]: https://github.com/zxfsee/afterburner/commit/2b559414e9d9a1b7c8c1c9de360cd40ad4691c93
[ecdff5a]: https://github.com/zxfsee/afterburner/commit/ecdff5a3bd2251d7e4bfcb8fcba34860a29fb4c3
[c5f7735]: https://github.com/zxfsee/afterburner/commit/c5f77356d87d12c5b94406d30cd2e90844b64ddb
[54a3da9]: https://github.com/zxfsee/afterburner/commit/54a3da95720910210cce4a4781af61b5c567cae2
[e8fab39]: https://github.com/zxfsee/afterburner/commit/e8fab39d86407438978fe38f4e365446f7e627d1
[fe78d77]: https://github.com/zxfsee/afterburner/commit/fe78d77bca9828bbba1d5e6bee465a58b6d7a4c6
[612ec19]: https://github.com/zxfsee/afterburner/commit/612ec19040e59ad8edced4805442da8c205044bb
[0a21484]: https://github.com/zxfsee/afterburner/commit/0a2148485d907c187c4c9748a1568263ef05f4f8
[e9a1775]: https://github.com/zxfsee/afterburner/commit/e9a17759fd29d7d22dd6c9e6ac10185ec11ce6bf
[6f96cf2]: https://github.com/zxfsee/afterburner/commit/6f96cf2797901d916e6a5736d0d3909c21f98f2f
[85d86de]: https://github.com/zxfsee/afterburner/commit/85d86de81992fb410528fae0371bcd0e208820e0
[0cb0918]: https://github.com/zxfsee/afterburner/commit/0cb0918530ed829d064ac113acb3af7e14bfbd9e
[b25218f]: https://github.com/zxfsee/afterburner/commit/b25218f6daf74dfed26ff56051c240dfd7e21cc2
[3b0b4bc]: https://github.com/zxfsee/afterburner/commit/3b0b4bc7c5caed16f7928b529af8416bf8cdef9c
[43a9d4b]: https://github.com/zxfsee/afterburner/commit/43a9d4b9ddffe4b5099a3507eadb46d3372d4a2b
[c877766]: https://github.com/zxfsee/afterburner/commit/c8777660825e3549ad71757c049dbe3045b98a99
[87f756c]: https://github.com/zxfsee/afterburner/commit/87f756c9ef0448e2ea3bfdc134224ee28dbc38dd
[1a5f52e]: https://github.com/zxfsee/afterburner/commit/1a5f52e69c4f318560a558055f3c785c886da195
[bff0ee1]: https://github.com/zxfsee/afterburner/commit/bff0ee16884fc6fa9de6253523ffd8f59c261b5a
[96d2498]: https://github.com/zxfsee/afterburner/commit/96d24989a3f56bd17bbdd251e2f4b4b13ca68cea
[a5152ee]: https://github.com/zxfsee/afterburner/commit/a5152eeeb92572699bc6e3b2607c511782dbd24f
[ae6ab47]: https://github.com/zxfsee/afterburner/commit/ae6ab4723f708e0fcb591c3dd517fe3ef9c74e44
[2a0ecb9]: https://github.com/zxfsee/afterburner/commit/2a0ecb9c9b079a20b14e5556e6c6ca7a65cd753f
[fa719e1]: https://github.com/zxfsee/afterburner/commit/fa719e17eb80dc84cf683aebcc2a1ea276a438b6
[cae2d95]: https://github.com/zxfsee/afterburner/commit/cae2d957ab382e407f2a8e065cb6c356dfdf9b0d
[d557728]: https://github.com/zxfsee/afterburner/commit/d557728da7cf4537f356faabbc7cc6ffb6b44cf9
[ddcd489]: https://github.com/zxfsee/afterburner/commit/ddcd48981a25e93f1d60635695cab5c8ff10e751
[ba6bdd9]: https://github.com/zxfsee/afterburner/commit/ba6bdd9bbd07308ae7bdb16cecc2b316a4e9c491
[0179925]: https://github.com/zxfsee/afterburner/commit/01799257b3534ef68f0f96aabc2e71c13b04bc08
[0e9125e]: https://github.com/zxfsee/afterburner/commit/0e9125e8f6b73a4d44fbcf932bf0b70f380b2785
[1970c2d]: https://github.com/zxfsee/afterburner/commit/1970c2d92dfd305a944decccbc62e208b17304e4
[d183107]: https://github.com/zxfsee/afterburner/commit/d1831072ef41c950715717dd5504bef055ab2799
[1271e03]: https://github.com/zxfsee/afterburner/commit/1271e037720193d2e65e2b3d170d5f893dbb5f6a
[e05bbd8]: https://github.com/zxfsee/afterburner/commit/e05bbd820cfedb514987cff2f1ccc39300b75070
[7169e39]: https://github.com/zxfsee/afterburner/commit/7169e39ead2a669a3e683052f64e4ed88a3a770b
[23f0589]: https://github.com/zxfsee/afterburner/commit/23f05896e032152a7b720feeebcb79863e67961a
[fe49697]: https://github.com/zxfsee/afterburner/commit/fe49697323093726cee82e9f0f66664d0c528699
[0e1d423]: https://github.com/zxfsee/afterburner/commit/0e1d42365e77301ff7ae0c7049d608545bd44a48
[8715201]: https://github.com/zxfsee/afterburner/commit/8715201bdaec3b93eb57d0547b7ec106b57c6b3f
[93595e7]: https://github.com/zxfsee/afterburner/commit/93595e79d659c623af54613f232f6b8457968881
[2dffe09]: https://github.com/zxfsee/afterburner/commit/2dffe09daa4879e2328c034f8c0d38c718c38fa4
[33c30b0]: https://github.com/zxfsee/afterburner/commit/33c30b0270bcebda52abe2d6dfa45bf4af285291
[7ba9f01]: https://github.com/zxfsee/afterburner/commit/7ba9f013ac2c36d514e0f45bb005523ef9c2674f
[c01b56e]: https://github.com/zxfsee/afterburner/commit/c01b56ed3d234c88ea047443d20b8414fae0795f
[c990787]: https://github.com/zxfsee/afterburner/commit/c99078779f73d4c92adbd2384f3d5284910b52be
[012d4f2]: https://github.com/zxfsee/afterburner/commit/012d4f2d2c86acc01994f6da5f41eb66b4848c6f
[5538479]: https://github.com/zxfsee/afterburner/commit/5538479fbd2e24d8185922298c66de7bd174664c
[b86b6e0]: https://github.com/zxfsee/afterburner/commit/b86b6e0505ca0fbf572ae4240ea98f9697fdacfd
[4ec1d55]: https://github.com/zxfsee/afterburner/commit/4ec1d551bbb3206b4c42a11f009f57558f7d31a8
[4abdebd]: https://github.com/zxfsee/afterburner/commit/4abdebd32e33f9cdd116919041bdc8cc3d998034
[e62bb20]: https://github.com/zxfsee/afterburner/commit/e62bb204f6d2d9fddd146c1bdc378638ea26d093
[cf7413a]: https://github.com/zxfsee/afterburner/commit/cf7413ae6d21561ce08d02f8bec12eb76b8c110e
[c9b7e43]: https://github.com/zxfsee/afterburner/commit/c9b7e43ee3cc917e63fa53a5a87e0ced8a5388ea
[9ad2b97]: https://github.com/zxfsee/afterburner/commit/9ad2b973155ea3956caf6ae4f08cca85049646a8
[3a557ec]: https://github.com/zxfsee/afterburner/commit/3a557ecf73796d6e73a897c3174eb5a6b5f185a9
[eaf0c7a]: https://github.com/zxfsee/afterburner/commit/eaf0c7ad48592e9fa1a8d35bcc7d1f5c3b7836c5
[42d0d0b]: https://github.com/zxfsee/afterburner/commit/42d0d0b8bb8f6ded3b9c92d1e70c5998db5ce7aa
[89ed136]: https://github.com/zxfsee/afterburner/commit/89ed13610da071a5d7434c322102cd5b2f0052e0
[b30e2ac]: https://github.com/zxfsee/afterburner/commit/b30e2ac3c930ccc429b95cbdd6cfc3e508a9975a
[87a03cc]: https://github.com/zxfsee/afterburner/commit/87a03cc02c8cfc8683f24c632dce0657d1d26327
[739a172]: https://github.com/zxfsee/afterburner/commit/739a17284a7696e8dd79eee6aa1e8a2921a36864
[a7a0baf]: https://github.com/zxfsee/afterburner/commit/a7a0baf776042caa2b04bf902e8d85569e9d9089
[a7aa812]: https://github.com/zxfsee/afterburner/commit/a7aa8125e19d3f56fbe1665d5825d81345831998
[1fdee5c]: https://github.com/zxfsee/afterburner/commit/1fdee5c9a197a62a9fc12cbad5b27d11af125a81
[97dd62f]: https://github.com/zxfsee/afterburner/commit/97dd62f4195d51782feb31da22e3a3611f5b8380
[b8b9db9]: https://github.com/zxfsee/afterburner/commit/b8b9db91e6980ec59ea82f145d9390d0d5e1b54b
[4db3092]: https://github.com/zxfsee/afterburner/commit/4db30920174f608799341fd03bf3e524f8f5776e
[e5acec4]: https://github.com/zxfsee/afterburner/commit/e5acec4dcac5ccfe595809a79bd3f1e15e493d88
[f0309e2]: https://github.com/zxfsee/afterburner/commit/f0309e255e9441ef1ccd9c868c6628530522ad38
[f4f6828]: https://github.com/zxfsee/afterburner/commit/f4f682846bb3a19ac2096eddb24a9c723530b745
[0467bd0]: https://github.com/zxfsee/afterburner/commit/0467bd0c392ec29ec73815fb957d88f7c6a01aa8
[69ca513]: https://github.com/zxfsee/afterburner/commit/69ca513bd1bb0bb27f18a2cd0b4b4e04fc49bfc5
[e31fe2c]: https://github.com/zxfsee/afterburner/commit/e31fe2c30b991b3abd0e9637dd7d227f8bb12885
[fbf3db0]: https://github.com/zxfsee/afterburner/commit/fbf3db0c892f764e9dcf70b335d017e961a68a50
[b6d60fc]: https://github.com/zxfsee/afterburner/commit/b6d60fcdfc3a89cb398c72a856561abaa55628bd
[0068d34]: https://github.com/zxfsee/afterburner/commit/0068d34c29e60e6fe47e95ba12f4a826e99b27e1
[f410918]: https://github.com/zxfsee/afterburner/commit/f41091876d28ea6e84be9945460f11020227fcf0
[be2ee09]: https://github.com/zxfsee/afterburner/commit/be2ee090e8cde7d3a988285ed7147e22af9e584f
[8a7d2ba]: https://github.com/zxfsee/afterburner/commit/8a7d2ba1f00b73edcdbb58ec00ea5d797162799f
[c53e0c7]: https://github.com/zxfsee/afterburner/commit/c53e0c7c4af0b2a112fd20a056259ec7fc14bcd2
[8fa3d61]: https://github.com/zxfsee/afterburner/commit/8fa3d615169d9d666c056f95373cc920950998da
[3270de3]: https://github.com/zxfsee/afterburner/commit/3270de3ca9740766b2dbafef9f6ced791d446c1a
[6050723]: https://github.com/zxfsee/afterburner/commit/605072308afc826ae3cc97ff5c2564b1cbd0e5a5
[9d84f8e]: https://github.com/zxfsee/afterburner/commit/9d84f8e9f840fe6af5ad8765835e24f42e9d7a38
[9014e7b]: https://github.com/zxfsee/afterburner/commit/9014e7be92c3709af3e79a3a3564b18530bfc8c7
[e283bfd]: https://github.com/zxfsee/afterburner/commit/e283bfd47fb443f1acd5da25390df2effffe5cd4
[2ab5f99]: https://github.com/zxfsee/afterburner/commit/2ab5f99e7db344c9bbb5d91b8da64d39648e4f91
[f8968ee]: https://github.com/zxfsee/afterburner/commit/f8968eea9a7afd1b9ba1fbc5f2238ac1968cdb32
[d9c24d0]: https://github.com/zxfsee/afterburner/commit/d9c24d0a379c73cb6cbdfdd873389c1642af4e64
[20771ce]: https://github.com/zxfsee/afterburner/commit/20771ce40632f711d92d361dd195e2ef4260bd0b
[3f9bf36]: https://github.com/zxfsee/afterburner/commit/3f9bf36b53bd8e2e1e7b7c5c8c3d5f145919f6fd
[c74ec5c]: https://github.com/zxfsee/afterburner/commit/c74ec5cc69809410f88b20c33203c9c2afced3e7
[85cc1f0]: https://github.com/zxfsee/afterburner/commit/85cc1f09c61aab1db50efe222defd56e154ec4ca
[5ce1ff7]: https://github.com/zxfsee/afterburner/commit/5ce1ff75889fd051023bb9b3eba332a656f1cba6
[8c60b1f]: https://github.com/zxfsee/afterburner/commit/8c60b1fda5ae6d1cd5bd7122de985e5bf4c52cad
[9061c6d]: https://github.com/zxfsee/afterburner/commit/9061c6d31653d6add0585f8ba0175d607844f840
[d29d6b5]: https://github.com/zxfsee/afterburner/commit/d29d6b58f4ec06f4fd03e0a2c8be2f68bf148174
[adf9a4f]: https://github.com/zxfsee/afterburner/commit/adf9a4f3688817b831426d9e5bac19fa4d71f8ae
[a46a6c1]: https://github.com/zxfsee/afterburner/commit/a46a6c1ca554e78279055d5a7fa29710cf08919d
[5de3aa2]: https://github.com/zxfsee/afterburner/commit/5de3aa2667921bcb42799695e7c09d58c6d4d28e
[d4ff6e2]: https://github.com/zxfsee/afterburner/commit/d4ff6e293801a6988b31b2e5fff6e0267ade8ec5
[dcbbcb9]: https://github.com/zxfsee/afterburner/commit/dcbbcb90667fe44de1a788580436427a4b8fa0d2
[cdb8173]: https://github.com/zxfsee/afterburner/commit/cdb817317813b9776fc0425403ebae589683c17f
[5d7b51a]: https://github.com/zxfsee/afterburner/commit/5d7b51a68bac2c025c2878ee1f7e74f3e3eae2e4
[262f68e]: https://github.com/zxfsee/afterburner/commit/262f68e5ee048fe2b9e1e7fdd708c00c26b83689

<!-- generated by git-cliff -->
