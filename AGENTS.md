# Agent guide

dex-parser-analyzer is a Cargo workspace of four crates (layout and module roles: `docs/architecture.md`):

- `dex-core/`: the DEX parser, bytecode decoder, graphs and DTOs. Tests are in `dex-core/tests/`, fixtures in `dex-core/tests/data/` (Androguard-derived, Apache-2.0; see NOTICE).
- `dex-analysis/`: the pattern, structural and taint passes. Every rule id is in `docs/rules.md`, which must change in the same commit as any rule.
- `dex-cli/`: the command-line front end.
- `dex-gui/`: the Leptos 0.8 browser GUI (wasm32, built with trunk; `publish = false`).

## Commands

Work inside the dev shell, which pins the toolchain from `rust-toolchain.toml`:

| Command | What it runs |
|---|---|
| `nix develop` | enter the dev shell (Rust 1.99.1, the wasm32 target, trunk, just, cargo-deny) |
| `just ci` | the gate: `lint`, `clippy-wasm`, `test`, `doc` and `deny`; it must pass before a commit |
| `just lint` | `cargo fmt --check` and clippy with `-D warnings` |
| `just clippy-wasm` | clippy for `dex-gui` on `wasm32-unknown-unknown` |
| `just test` | `cargo test --workspace` |
| `just doc` | rustdoc with `-D warnings` |
| `just deny` | `cargo deny check advisories bans licenses sources` (policy in `deny.toml`) |
| `cargo run -p dex-cli -- <file.dex>` | run the CLI on a file |
| `cd dex-gui && trunk serve` | run the GUI locally |

CI (`.github/workflows/ci.yml`) runs the same checks.

## Rules

- Parsing code must return a `DexError`, never panic, on malformed input: the GUI parses untrusted uploads.
- A new rule needs a positive and a negative fixture test, plus its row in `docs/rules.md`.
- Keep `CHANGELOG.md`'s `[Unreleased]` section current with every user-visible change.
- Never commit `dex-gui/dist/` or other build output; it is gitignored.
- Keep commit subjects short and imperative, as the history already does.
