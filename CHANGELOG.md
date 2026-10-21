# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Changed
- **Breaking:** `DexFile::strings()` yields `DexResult<&str>` instead of `&str`, so a malformed entry is reported instead of panicking.
- The project is dual-licensed MIT OR Apache-2.0 (previously MIT under the FossRust name). Every published crate ships both licence files, and NOTICE attributes the Androguard-derived test fixtures.
- Every crate uses edition 2024 and the shared `[workspace.package]` metadata (description, repository, keywords, categories). `rust-version` is 1.99, and `rust-toolchain.toml` pins Rust 1.99.1.
- The README demo link, the GUI title and the GUI header link point at github.com/dhilipsiva (the fossrust.github.io URL returned 404).
- The GUI moves to Leptos 0.8. Its release bundle shrinks from about 6 MB to 2.4 MB of WebAssembly.
- Dependencies: zip 9 (fixes RUSTSEC-2026-0367, -0368 and -0369), anyhow 1.0.104 (RUSTSEC-2026-0190), bytes 1.12.1 (RUSTSEC-2026-0007), petgraph 0.8 and thiserror 2. The unused nom dependency is removed.
- The fixture-sweep tests fail when a fixture is not a DEX file, instead of skipping it.

### Added
- `deny.toml`, with `cargo deny` in CI, and Dependabot updates for Cargo and GitHub Actions.
- A rebuilt CI workflow: fmt, clippy (native and wasm32), tests, docs, cargo deny, MSRV and workflow lint, with SHA-pinned actions.
- `just ci` as the local gate.
- docs/architecture.md and docs/rules.md, per-crate READMEs, CONTRIBUTING.md and SECURITY.md.

### Removed
- The committed GUI build (`dex-gui/dist`) and two fixtures that were 14-byte "404: Not Found" pages rather than DEX files.

### Fixed
- Three crashes on crafted input are now errors. An out-of-range `class_data_off` and an out-of-range `string_data_off` are rejected by `parse_dex` with `SectionOutOfBounds`. A literal whose 64-byte preview cut fell inside a multibyte character no longer panics the analyzer (thanks to @bilieebiliee1-design, whose PR #1 found it).
- Every leb128 read in the parser is bounds-checked, as is the raw-bytes copy of an undecoded instruction.
- The GUI page no longer requests a nonexistent `dex-gui.js`.
