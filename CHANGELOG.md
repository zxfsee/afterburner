# Changelog

## TODO

- Deployment stack launch evidence handoff adapter [Serving/Deployment Infra, Runtime Infra]
  - Goal: Export one stable handoff artifact over the deployment stack launch evidence bundle so downstream deployment or transport tooling can resolve a single stack-launch locator without depending on local bundle path conventions.
  - Kind: `mixed`
  - Boundary: `adapter-deployment`
  - Contracts: `artifact`, `event`, `ops`
  - Scope: `src/`, `fixtures/`, `tests/`, `README.md`, `justfile`, `docs/adr/`

- Workflow reference completeness gate [Runtime Infra, Serving/Deployment Infra]
  - Goal: Keep `docs/workflows.md` aligned with the canonical `just` workflow surface and the current artifact/receipt families so README can stay frontpage-first without drifting from operator entrypoints.
  - Kind: `gate`
  - Boundary: `none`
  - Contracts: `none`
  - Scope: `README.md`, `docs/workflows.md`, `justfile`, `tests/`

- JSON to RON format fit investigation gate [Data Infra, Runtime Infra]
  - Goal: Evaluate whether any current JSON-based artifact or event surfaces should migrate to RON, and whether the readability or ergonomics gains would justify the contract churn and tooling impact.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `artifact`, `event`
  - Scope: `docs/adr/`, `ARCHITECTURE.md`, `README.md`, `tests/`

- Burn distributed optimizer and checkpoint state gate [Distributed Training, Runtime Infra]
  - Goal: Define the minimum in-job optimizer-state and checkpoint-group contract needed for distributed recovery so future Burn-side multi-device training can resume consistently across shards and ranks without coupling cluster scheduling into checkpoint semantics.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `artifact`
  - Scope: `docs/adr/`, `ARCHITECTURE.md`, `README.md`, `tests/`

- Distributed shard lineage evidence bundle handoff gate [Distributed Training, Data Infra]
  - Goal: Define the minimum handoff contract for distributed shard lineage evidence bundles so downstream distributed tooling can resolve one stable bundle entrypoint without depending on surrounding checkpoint workspace layout.
  - Kind: `gate`
  - Boundary: `core-contract`
  - Contracts: `artifact`
  - Scope: `docs/adr/`, `ARCHITECTURE.md`, `README.md`, `tests/`

- kube-rs GPU scheduler placement fit gate [Runtime Infra, Serving/Deployment Infra]
  - Goal: Define a kube-rs-compatible control-plane contract for node inventory, topology-aware GPU placement, queue admission, and job lifecycle reconciliation so the later cluster path can reuse the same lease and state-boundary model as the single-node scheduler.
  - Kind: `gate`
  - Boundary: `adapter-deployment`
  - Contracts: `ops`
  - Scope: `docs/adr/`, `README.md`, `tests/`

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
- Cut over recipe to current pointer ([c62bd5a])
- Add stack launch evidence bundle adapter ([bac2905])

### Changed

- Standardize artifacts layout and include taplo.toml in flake ([41b6336])
- Rename artifacts/inference to artifacts/infer and update references ([173ebc1])
- Rename artifacts/infer → artifacts/inference (update code/docs/justfile) ([cadcadb])
- Return [1,28,28] tensor from mnist_image_to_tensor ([5ab0036])
- Unify CLI into afterburner subcommands ([13179ee])
- Extract artifact event helper ([93c5056])
- Split runtime artifacts metadata and events ([bc12a72])

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
- Reprioritize train profiling and readme items ([1b54d33])
- Sharpen frontpage ([7b4695e])
- Prioritize readme frontpage pass ([6b0e370])
- Sharpen frontpage ([a644f55])
- Split workflow reference from frontpage ([e402eff])
- Split frontpage reference index ([b167a4c])
- Expand contract coverage index ([4941233])

### Fixed

- Align artifact resolution and runtime fallback ([fbfa228])
- Enforce rollout budget schema gate ([e2c0b4c])
- Enforce strict pretraining metadata shape ([f7b370f])
- Align single infer success envelope ([1040623])

### Other

- Revert "feat(devenv): enable Rust and mold in devenv.nix" ([271f315])
- Revert "chore(devenv): add devenv & direnv configs, update .gitignore, add initial model.rs" ([3d1724d])

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
[1b54d33]: https://github.com/zxfsee/afterburner/commit/1b54d335da4c11337157429027e503afae6bfa59
[bc12a72]: https://github.com/zxfsee/afterburner/commit/bc12a72c3bea08bd2ad02fb615cf4b3fba288ca1
[c62bd5a]: https://github.com/zxfsee/afterburner/commit/c62bd5aa3a3336058116b0c053ba73d9be48f626
[7b4695e]: https://github.com/zxfsee/afterburner/commit/7b4695e538e327013e210fe88611b6543d2ac892
[6b0e370]: https://github.com/zxfsee/afterburner/commit/6b0e3706b0f2e95c06051dc9250cda005de9597d
[a644f55]: https://github.com/zxfsee/afterburner/commit/a644f558a247e988bd28f6c5a68e26c0bc34e05e
[e402eff]: https://github.com/zxfsee/afterburner/commit/e402effb5b470422758b5c210b247324cd41b7e7
[bac2905]: https://github.com/zxfsee/afterburner/commit/bac29050c944ee9721ffa8a2338dc4abc53b4220
[b167a4c]: https://github.com/zxfsee/afterburner/commit/b167a4cf94ec26deeeb9582bfd6520e55bc890a4
[4941233]: https://github.com/zxfsee/afterburner/commit/4941233114d9c8529ca850ea23717566d0002244

<!-- generated by git-cliff -->
