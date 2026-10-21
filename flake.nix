{
  description = "dex-parser-analyzer development flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    # nixpkgs 26.11 dropped x86_64-darwin, so the default system list no longer evaluates.
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ] (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        # The toolchain is pinned once, in rust-toolchain.toml, which CI's rustup reads too.
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        rustPlatform = pkgs.makeRustPlatform { cargo = rustToolchain; rustc = rustToolchain; };
        lib = pkgs.lib;

        version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).workspace.package.version;
        # Only what a build reads, so a docs or workflow edit does not rebuild the packages.
        src = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            ./Cargo.toml
            ./Cargo.lock
            ./dex-core
            ./dex-analysis
            ./dex-cli
            (lib.fileset.difference ./dex-gui (lib.fileset.maybeMissing ./dex-gui/dist))
          ];
        };
        meta = {
          homepage = "https://github.com/dhilipsiva/dex-parser-analyzer";
          license = with lib.licenses; [ mit asl20 ];
        };

        # trunk refuses a wasm-bindgen CLI that differs from the crate in Cargo.lock; fail here,
        # with the fix, instead.
        lockedWasmBindgen = (lib.findFirst (p: p.name == "wasm-bindgen")
          (throw "Cargo.lock has no wasm-bindgen")
          (builtins.fromTOML (builtins.readFile ./Cargo.lock)).package).version;
        wasmBindgenCli = pkgs.wasm-bindgen-cli_0_2_129;
      in
      {
        formatter = pkgs.nixpkgs-fmt;

        # `nix build .#dex-cli` (or `nix run . -- app.apk`): the dexpa binary.
        packages.dex-cli = rustPlatform.buildRustPackage {
          pname = "dexpa";
          inherit version src;
          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = [ "--package" "dexpa-cli" ];
          cargoTestFlags = [ "--package" "dexpa-cli" ];
          meta = meta // {
            description = "Parse and statically analyse Android DEX files and APKs";
            mainProgram = "dexpa";
          };
        };
        packages.default = self.packages.${system}.dex-cli;

        # `nix build .#dex-gui`: the static browser bundle (index.html, JS glue and wasm), served
        # from a relative public URL so it works under any path.
        packages.dex-gui = assert lib.assertMsg (wasmBindgenCli.version == lockedWasmBindgen)
          "Cargo.lock pins wasm-bindgen ${lockedWasmBindgen}, the flake uses wasm-bindgen-cli ${wasmBindgenCli.version}: update wasmBindgenCli in flake.nix";
          rustPlatform.buildRustPackage {
          pname = "dex-gui";
          inherit version src;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = [ pkgs.trunk wasmBindgenCli pkgs.binaryen ];
          buildPhase = ''
            runHook preBuild
            export HOME=$TMPDIR
            (cd dex-gui && trunk build --release --offline --public-url ./)
            runHook postBuild
          '';
          installPhase = ''
            runHook preInstall
            cp -r dex-gui/dist $out
            runHook postInstall
          '';
          doCheck = false;
          meta = meta // { description = "dex-parser-analyzer in the browser (static bundle)"; };
        };

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
