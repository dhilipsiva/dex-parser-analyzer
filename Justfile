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

# The local release rehearsal (RELEASING.md, "Before tagging"): the gate, the publish dry run, and
# the two Nix packages (the dexpa binary and the GUI bundle). Uploads nothing.
release: ci release-dry-run
    nix build --no-link --print-out-paths .#dex-cli .#dex-gui

# Fuzz one target for `seconds` (inside `nix develop .#fuzz`); seeds are the small test fixtures.
fuzz target="parse_dex" seconds="60":
    mkdir -p fuzz/corpus/{{target}}
    find dex-core/tests/data -maxdepth 1 -type f -size -64k -exec cp {} fuzz/corpus/{{target}}/ \;
    cd fuzz && cargo fuzz run {{target}} corpus/{{target}} -- -max_total_time={{seconds}} -max_len=65536

# Rebuild dex-analysis/tests/data/rules-{positive,negative}.dex from the Java sources in
# dex-analysis/tests/fixtures-src (inside `nix develop .#fixtures`). The Android stubs are on the
# classpath only, so the fixtures reference the framework classes without containing them.
fixtures:
    #!/usr/bin/env bash
    set -euo pipefail
    src=dex-analysis/tests/fixtures-src
    out=$(mktemp -d)
    javac --release 17 -d "$out/stubs" $(find "$src/stubs" -name '*.java' | sort)
    for kind in positive negative; do
        javac --release 17 -cp "$out/stubs" -d "$out/$kind" $(find "$src/$kind" -name '*.java' | sort)
        mkdir -p "$out/$kind-dex"
        java -cp "$R8_JAR" com.android.tools.r8.D8 --release --min-api 26 \
            --lib "$JAVA_HOME" --classpath "$out/stubs" --output "$out/$kind-dex" \
            $(find "$out/$kind" -name '*.class' | sort)
        cp "$out/$kind-dex/classes.dex" "dex-analysis/tests/data/rules-$kind.dex"
    done
    echo "fixtures written to dex-analysis/tests/data (scratch: $out)"

