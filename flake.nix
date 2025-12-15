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
      rust-overlay,
      treefmt-nix,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-darwin"
      ];

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
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };

          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            targets = [ "wasm32-wasip2" ];
          };

          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
          src = craneLib.cleanCargoSource ./.;

          # Common arguments can be set here to avoid repeating them later
          commonArgs = {
            inherit src;
            strictDeps = true;

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
                partitions = 1;
                partitionType = "count";
                cargoNextestPartitionsExtraArgs = "--no-tests=pass";
              }
            );
          };

          packages = {
            default = afterburner;
          };

          apps.default = {
            type = "app";
            program = "${afterburner}/bin/afterburner";
          };

          devShells.default = craneLib.devShell {
            # Inherit inputs from checks.
            checks = self.checks.${system};

            # Additional dev-shell environment variables can be set directly
            # MY_CUSTOM_DEVELOPMENT_VAR = "something else";

            # Extra inputs can be added here; cargo and rustc are provided by default.
            packages = with pkgs; [
              just
              direnv
            ];
          };

          treefmt = {
            projectRootFile = "flake.nix";
            programs = {
              nixfmt.enable = true;
              rustfmt.enable = true;
              taplo.enable = true;
            };
            settings.formatter.taplo.options = [
              "--config"
              (toString ./taplo.toml)
            ];
          };

          formatter = config.treefmt.build.wrapper;
        };
    };
}
