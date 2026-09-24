{
  description = "os9s — k9s-inspired TUI for kolla-ansible OpenStack, written in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    # Fenix: best-in-class Rust toolchain management for Nix
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # Crane: Nix library for building Cargo projects
    crane = {
      url = "github:ipetkov/crane";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        # Stable Rust toolchain with rust-analyzer, clippy, rustfmt
        rustToolchain = fenix.packages.${system}.stable.withComponents [
          "cargo"
          "clippy"
          "rust-src"
          "rustc"
          "rustfmt"
          "rust-analyzer"
        ];

        # Set up crane with our fenix toolchain
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Native build inputs (build-time)
        nativeBuildInputs = with pkgs; [
          pkg-config
        ];

        # Runtime link dependencies
        buildInputs = with pkgs; [
          openssl
          openssl.dev
        ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
        ];

        # Build the release binary via crane
        openstack-tui = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;
          inherit nativeBuildInputs buildInputs;
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };

      in
      {
        # ── Packages ──────────────────────────────────────────────────────────
        packages.default = openstack-tui;
        packages.openstack-tui = openstack-tui;

        # ── App runner ────────────────────────────────────────────────────────
        apps.default = flake-utils.lib.mkApp {
          drv = openstack-tui;
        };

        # ── Dev shell  (`nix develop`) ────────────────────────────────────────
        devShells.default = pkgs.mkShell {
          name = "openstack-tui-dev";

          nativeBuildInputs = nativeBuildInputs ++ [
            rustToolchain

            # Cargo productivity tools
            pkgs.cargo-watch      # cargo watch -x run  (hot reload)
            pkgs.cargo-nextest    # faster parallel test runner
            pkgs.cargo-audit      # CVE / advisory audits
            pkgs.cargo-outdated   # show stale deps
            pkgs.cargo-expand     # expand proc-macros for debugging
            pkgs.tokio-console    # live async task inspector

            # General dev utilities
            pkgs.git
            pkgs.gh               # GitHub CLI
            pkgs.jq               # JSON pretty-print for API debugging
            pkgs.openssl          # CLI for TLS cert inspection
          ];

          buildInputs = buildInputs;

          # ── Environment ───────────────────────────────────────────────────
          RUST_LOG = "openstack_tui=debug,warn";
          RUST_BACKTRACE = "1";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

          shellHook = ''
            echo ""
            echo "  ╔══════════════════════════════════════════════╗"
            echo "  ║   🦀  openstack-tui  dev shell               ║"
            echo "  ╚══════════════════════════════════════════════╝"
            echo "  Rust   : $(rustc --version)"
            echo "  Cargo  : $(cargo --version)"
            echo ""
            echo "  ─── Quick Commands ───────────────────────────"
            echo "  cargo run                  run the TUI"
            echo "  cargo watch -x run         hot-reload dev mode"
            echo "  cargo nextest run          fast parallel tests"
            echo "  cargo clippy -- -D warnings lint (strict)"
            echo "  cargo fmt                  format all sources"
            echo "  cargo audit                CVE check"
            echo "  cargo doc --open           open API docs"
            echo "  ──────────────────────────────────────────────"
            echo ""
          '';
        };

        # ── CI shell (no interactive / GUI tools) ─────────────────────────────
        devShells.ci = pkgs.mkShell {
          nativeBuildInputs = nativeBuildInputs ++ [
            rustToolchain
            pkgs.cargo-nextest
            pkgs.cargo-audit
          ];
          buildInputs = buildInputs;
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };

        # ── Nix formatter ─────────────────────────────────────────────────────
        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
