# dex-parser-analyzer recipes, from products/templates/Justfile. Run them inside `nix develop`
# (the toolchain is pinned in rust-toolchain.toml). `just ci` is the one local gate.

set shell := ["bash", "-euo", "pipefail", "-c"]

export RUST_BACKTRACE := "1"

# List the recipes.
default:
    @just --list

# Type-check every crate and target.
check:
    cargo check --locked --workspace --all-targets

# Run the tests.
test:
    cargo test --locked --workspace

# Formatting and Clippy, with warnings as errors (C5).
lint:
    cargo fmt --all -- --check
    cargo clippy --locked --workspace --all-targets -- -D warnings

# Clippy for the browser GUI on its real target.
clippy-wasm:
    cargo clippy --locked -p dex-gui --target wasm32-unknown-unknown -- -D warnings

# Build the docs, with warnings as errors.
doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --no-deps

# Advisories, bans, licences and sources against the committed deny.toml (C8).
deny:
    cargo deny check advisories bans licenses sources

# The local gate: everything CI checks.
ci: lint clippy-wasm test doc deny

# Package and verify-build every publishable crate without uploading (skips `publish = false`).
release-dry-run:
    cargo publish --locked --dry-run --workspace
