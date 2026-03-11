# Changelog

## TODO

- Cargo workspace boundary split + dependency gate [Frameworks, Runtime Infra]
  - Goal: Split into Cargo workspace boundaries (`core` and adapters) and add an enforceable dependency gate so codebase growth remains maintainable and CI can target crates.
  - Kind: `mixed`
  - Boundary: `core-contract`
  - Contracts: `none`
  - Scope: `Cargo.toml`, `crates/`, `tests/`, `justfile`, `ARCHITECTURE.md`

- Custom kernel adoption threshold contract [Kernels, Numerics]
  - Goal: Define measurable thresholds and compatibility gates for custom kernel introduction so optimization work is evidence-driven and reversible.
  - Kind: `mixed`
  - Boundary: `core-contract`
  - Contracts: `artifact`
  - Scope: `src/train.rs`, `src/model.rs`, `tests/kernel_logits_shape.rs`, `fixtures/kernel_adoption_thresholds.json`, `docs/adr/`

- Async/runtime decision record (Tokio ecosystem) [Runtime Infra, Frameworks]
  - Goal: Decide if async runtime adoption is required for adapters and document constraints/trade-offs before introducing Tokio-dependent code.
  - Kind: `behavior`
  - Boundary: `adapter-http`
  - Contracts: `none`
  - Scope: `docs/adr/`, `ARCHITECTURE.md`

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
- Add observability dashboard adapter ([33fe53e])
- Align infer eval monitoring semantics ([49d584c])

### Changed

- Standardize artifacts layout and include taplo.toml in flake ([41b6336])
- Rename artifacts/inference to artifacts/infer and update references ([173ebc1])
- Rename artifacts/infer → artifacts/inference (update code/docs/justfile) ([cadcadb])
- Return [1,28,28] tensor from mnist_image_to_tensor ([5ab0036])
- Unify CLI into afterburner subcommands ([13179ee])

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
[33fe53e]: https://github.com/zxfsee/afterburner/commit/33fe53e5e96293831fa2fd3494340a3a560a5c81
[49d584c]: https://github.com/zxfsee/afterburner/commit/49d584c17e1f7d55f9ee35cb060d7a4d73811343

<!-- generated by git-cliff -->
