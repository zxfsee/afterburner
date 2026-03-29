{
  description = "Build a cargo project";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1";

    crane.url = "https://flakehub.com/f/ipetkov/crane/0";

    flake-parts.url = "https://flakehub.com/f/hercules-ci/flake-parts/0.1";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };

    deploy-rs = {
      url = "github:serokell/deploy-rs";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "https://flakehub.com/f/oxalica/rust-overlay/0.1";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix.url = "https://flakehub.com/f/numtide/treefmt-nix/0.1";

  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      crane,
      flake-parts,
      advisory-db,
      deploy-rs,
      rust-overlay,
      treefmt-nix,
      ...
    }:
    let
      deploymentProfile = builtins.fromJSON (
        builtins.readFile ./fixtures/deployment_target_profile.example.json
      );
    in
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-darwin"
      ];

      flake = {
        deploy.nodes.${deploymentProfile.profile_name} = {
          hostname = deploymentProfile.deploy_hostname;
          sshUser = deploymentProfile.ssh_user;
          profiles.afterburner = {
            user = deploymentProfile.ssh_user;
            profilePath = "${deploymentProfile.artifact_root}/profiles/afterburner";
            path =
              deploy-rs.lib.${deploymentProfile.system}.activate.custom
                self.packages.${deploymentProfile.system}.default
                "./bin/afterburner";
          };
        };
      };

      imports = [
        treefmt-nix.flakeModule
      ];

      perSystem =
        {
          system,
          lib,
          config,
          ...
        }:
        let
          isLinux = lib.hasSuffix "-linux" system;

          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              rust-overlay.overlays.default
            ]
            ++ lib.optionals isLinux [
              (final: prev: {
                stdenv = prev.useWildLinker prev.stdenv;
              })
            ];
          };

          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            targets = [ "wasm32-wasip2" ];
          };

          taploConfig = builtins.path {
            path = ./taplo.toml;
            name = "taplo.toml";
          };

          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
          src = lib.cleanSourceWith {
            src = ./.;
            filter =
              path: type:
              let
                rel = lib.removePrefix (toString ./. + "/") (toString path);
              in
              craneLib.filterCargoSources path type
              # Tests intentionally read tracked fixtures and repo-owned docs/workflow files
              # from CARGO_MANIFEST_DIR; generated artifacts stay out of the Nix source.
              || lib.hasPrefix "fixtures/" rel
              || lib.hasPrefix "docs/" rel
              || builtins.elem rel [
                "ARCHITECTURE.md"
                "README.md"
                "docs"
                "flake.nix"
                "fixtures"
                "justfile"
              ];
          };

          # Common arguments can be set here to avoid repeating them later
          commonArgs = {
            inherit src;
            strictDeps = true;

            nativeBuildInputs = lib.optionals pkgs.stdenv.isLinux [
              pkgs.wild
            ];

            buildInputs = [
              # Add additional build inputs here
            ]
            ++ lib.optionals pkgs.stdenv.isDarwin [
              # Additional darwin specific inputs can be set here
              pkgs.libiconv
            ];

            # Additional environment variables can be set directly
            # MY_CUSTOM_VAR = "some value";
          };

          # Build *just* the cargo dependencies, so we can reuse
          # all of that work (e.g. via cachix) when running in CI
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          # Build the actual crate itself, reusing the dependency
          # artifacts from above.
          afterburner = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
              doCheck = false;
            }
          );
        in
        {
          checks = {
            # Build the crate as part of `nix flake check` for convenience
            inherit afterburner;

            # Run clippy (and deny all warnings) on the crate source,
            # again, reusing the dependency artifacts from above.
            #
            # Note that this is done as a separate derivation so that
            # we can block the CI if there are issues here, but not
            # prevent downstream consumers from building our crate by itself.
            afterburner-clippy = craneLib.cargoClippy (
              commonArgs
              // {
                inherit cargoArtifacts;
                cargoClippyExtraArgs = "--all-targets -- --deny warnings";
              }
            );

            afterburner-doc = craneLib.cargoDoc (
              commonArgs
              // {
                inherit cargoArtifacts;
                # This can be commented out or tweaked as necessary, e.g. set to
                # `--deny rustdoc::broken-intra-doc-links` to only enforce that lint
                env.RUSTDOCFLAGS = "--deny warnings";
              }
            );

            # Check formatting
            formatting = config.treefmt.build.check self;

            # Audit dependencies
            afterburner-audit = craneLib.cargoAudit {
              inherit src advisory-db;
            };

            # Audit licenses
            afterburner-deny = craneLib.cargoDeny {
              inherit src;
            };

            # Run tests with cargo-nextest
            # Consider setting `doCheck = false` on `afterburner` if you do not want
            # the tests to run twice
            afterburner-nextest = craneLib.cargoNextest (
              commonArgs
              // {
                inherit cargoArtifacts;
                # Some integration tests intentionally shell out to repo workflow tools.
                nativeBuildInputs = (commonArgs.nativeBuildInputs or [ ]) ++ [
                  pkgs.jujutsu
                  pkgs.just
                ];
                partitions = 1;
                partitionType = "count";
                cargoNextestPartitionsExtraArgs = "--no-tests=pass";
              }
            );
          }
          // lib.optionalAttrs (system == deploymentProfile.system) (
            deploy-rs.lib.${system}.deployChecks self.deploy
          );

          packages = {
            default = afterburner.overrideAttrs (old: {
              meta = (old.meta or { }) // {
                description = "Afterburner training binary (Burn + Nix)";
              };
            });
          };

          apps.default = {
            type = "app";
            program = "${afterburner}/bin/afterburner";
            meta = {
              description = "Afterburner training binary (Burn + Nix)";
            };
          };

          devShells.default = craneLib.devShell {
            # Inherit inputs from checks.
            checks = self.checks.${system};

            # Additional dev-shell environment variables can be set directly
            # MY_CUSTOM_DEVELOPMENT_VAR = "something else";
            RUSTFLAGS = lib.optionalString pkgs.stdenv.isLinux "-C linker=clang -C link-arg=-fuse-ld=wild";

            # Extra inputs can be added here; cargo and rustc are provided by default.
            packages =
              with pkgs;
              [
                cargo-flamegraph
                deploy-rs.packages.${system}.default
                git-cliff
                just
                nushell
              ]
              ++ lib.optionals pkgs.stdenv.isLinux [
                clang
                wild
              ];
          };

          treefmt = {
            projectRootFile = "flake.nix";
            programs = {
              nixfmt.enable = true;
              rustfmt.enable = true;
              taplo.enable = true;
              just.enable = true;
            };
            # TODO: migrate toml to nix.
            settings.formatter.taplo.options = [
              "--config"
              (toString taploConfig)
            ];
          };

          formatter = config.treefmt.build.wrapper;
        };
    };
}
