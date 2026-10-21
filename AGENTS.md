# Agent guide

dex-parser-analyzer is a Cargo workspace of four crates (layout and module roles: `docs/architecture.md`):

- `dex-core/` (package `dexpa-core`): the DEX parser, bytecode decoder, graphs and DTOs. Tests are in `dex-core/tests/`, fixtures in `dex-core/tests/data/` (Androguard-derived, Apache-2.0; see NOTICE).
- `dex-analysis/` (package `dexpa-analysis`): the pattern, structural and taint passes. Every rule id is in `docs/rules.md`, which must change in the same commit as any rule.
- `dex-cli/`: the `dexpa` command-line front end (package `dexpa-cli`; integration tests in `dex-cli/tests/cli.rs`).
- `dex-gui/`: the Leptos 0.8 browser GUI (wasm32, built with trunk; `publish = false`).

## Commands

Work inside the dev shell, which pins the toolchain from `rust-toolchain.toml`:

| Command | What it runs |
|---|---|
| `nix develop` | enter the dev shell (Rust 1.99.1, the wasm32 target, trunk, just, cargo-deny) |
| `just ci` | the gate: `lint`, `clippy-wasm`, `test`, `doc` and `deny`; it must pass before a commit |
| `just lint` | `cargo fmt --check` and clippy with `-D warnings` |
| `just test-wasm` | inside `nix develop .#wasm-test` (Chromium and chromedriver): the GUI's browser smoke tests, `dex-gui/src/smoke.rs` |
| `just clippy-wasm` | clippy for `dex-gui` on `wasm32-unknown-unknown` |
| `just test` | `cargo test --workspace` |
| `just doc` | rustdoc with `-D warnings` |
| `just fixtures` | inside `nix develop .#fixtures` (JDK 21 and a pinned R8): recompile `dex-analysis/tests/fixtures-src` into `dex-analysis/tests/data/rules-{positive,negative}.dex` |
| `just release` | the local release rehearsal: `ci`, `cargo publish --dry-run` for the three crates, and `nix build` of both packages; uploads nothing (RELEASING.md) |
| `nix build .#dex-cli` / `nix build .#dex-gui` | the `dexpa` binary / the static GUI bundle |
| `just deny` | `cargo deny check advisories bans licenses sources` (policy in `deny.toml`) |
| `cargo run -p dexpa-cli -- <file.dex or .apk>` | run the `dexpa` CLI |
| `cd dex-gui && trunk serve` | run the GUI locally |

CI (`.github/workflows/ci.yml`) runs the same checks. `.github/workflows/release.yml` releases on a `v*` tag; only the owner tags (RELEASING.md).

## Rules

- Parsing code must return a `DexError`, never panic, on malformed input: the GUI parses untrusted uploads.
- A new rule needs a Java class in both `dex-analysis/tests/fixtures-src/positive` and `.../negative`, a line in the `rule_tests!` list in `dex-analysis/tests/rules_test.rs`, an entry in `dex-analysis/src/rules.rs`, and its row and remediation line in `docs/rules.md`; `docs_catalogue_and_tests_agree` fails until all four agree.
- Keep `CHANGELOG.md`'s `[Unreleased]` section current with every user-visible change.
- Never commit `dex-gui/dist/` or other build output; it is gitignored.
- Keep commit subjects short and imperative, as the history already does.
