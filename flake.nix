{
  description = "dex-parser-analyzer development flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        # The toolchain is pinned once, in rust-toolchain.toml, which CI's rustup reads too.
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in
      {
        formatter = pkgs.nixpkgs-fmt;

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain
            trunk
            just
            cargo-deny
            cargo-watch
            wasm-pack
            pkg-config
            openssl
            llvmPackages.clang
            binaryen
            git
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };

        # Rebuilds the analyzer's rule fixtures from dex-analysis/tests/fixtures-src: javac, then D8
        # (from the pinned R8 jar). `just fixtures` runs it.
        devShells.fixtures = pkgs.mkShell {
          packages = with pkgs; [ jdk21_headless just ];
          R8_JAR = pkgs.fetchurl {
            url = "https://dl.google.com/android/maven2/com/android/tools/r8/9.5.28/r8-9.5.28.jar";
            hash = "sha256-OKn/quu9F9FRTj7L/1qFuYm7E8VRueIkx0CVFO6oT/U=";
          };
        };

        # cargo-fuzz needs nightly (-Zsanitizer); pinned so a run is reproducible. Used by fuzz.yml.
        devShells.fuzz = pkgs.mkShell {
          packages = with pkgs; [
            (rust-bin.nightly."2026-10-20".default.override { extensions = [ "rust-src" ]; })
            cargo-fuzz
            just
          ];
        };
      });
}
