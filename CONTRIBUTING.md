# Contributing

Thanks for helping. Bug reports, crashing inputs, rule ideas and pull requests are all welcome.

## Set up

```bash
nix develop     # Rust 1.99.1 from rust-toolchain.toml, plus the wasm32 target, trunk, just and cargo-deny
just ci         # fmt, clippy (native and wasm32), tests, docs, cargo deny
```

Without Nix, a rustup install picks up `rust-toolchain.toml`, but you then need trunk, just and cargo-deny yourself.

## Pull requests

- Run `just ci` before you push. CI runs the same checks.
- Parser changes must never panic on malformed input; return a `DexError` and add a malformed-input test under `dex-core/tests/`.
- A new or changed analysis rule needs a positive and a negative fixture test and an updated row in [docs/rules.md](docs/rules.md).
- Add a line under `[Unreleased]` in [CHANGELOG.md](CHANGELOG.md) for anything a user would notice.
- Keep commits focused, with short imperative subjects.

## Security issues

Do not open a public issue for a vulnerability, or for an input that crashes the parser in a way you think is exploitable. Follow [SECURITY.md](SECURITY.md) instead.

## Licence

By contributing, you agree that your contributions are dual-licensed under MIT OR Apache-2.0, as the README states.
