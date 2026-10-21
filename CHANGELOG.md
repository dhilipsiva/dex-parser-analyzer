# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Changed
- The GUI loads nothing from a third party: Bootstrap's CSS is self-hosted, the unused highlight.js (it ran before the app rendered, so it never highlighted anything) is gone, and a release build carries a Content-Security-Policy that allows only the page's own scripts, styles and wasm. The Pages deploy builds the bundle with the flake (pinned trunk, cached) and runs on changes to any crate the GUI links, not only `dex-gui/`.
- **Breaking:** `Finding` gains `cwe` (the rule's CWE number) and `remediation` (what to change), filled by the engine from the new `rules::RULES` catalogue; both deserialize as `null` when absent, so older JSON reports still load.
- The report lists findings most severe first, with exact duplicates removed, before `max_findings` truncates it, so a cap keeps the most severe findings (previously it kept the first ones emitted). `Severity` is now `Ord`.
- **Breaking:** the crates are renamed `dexpa-core`, `dexpa-analysis` and `dexpa-cli` (import them as `dexpa_core` and `dexpa_analysis`; `dex-cli` is taken on crates.io), and the CLI binary is `dexpa`. The directories keep their names.
- **Breaking:** `DexFile::strings()` yields `DexResult<&str>` instead of `&str`, so a malformed entry is reported instead of panicking.
- The project is dual-licensed MIT OR Apache-2.0 (previously MIT under the FossRust name). Every published crate ships both licence files, and NOTICE attributes the Androguard-derived test fixtures.
- Every crate uses edition 2024 and the shared `[workspace.package]` metadata (description, repository, keywords, categories). `rust-version` is 1.99, and `rust-toolchain.toml` pins Rust 1.99.1.
- The README demo link, the GUI title and the GUI header link point at github.com/dhilipsiva (the fossrust.github.io URL returned 404).
- The GUI moves to Leptos 0.8. Its release bundle shrinks from about 6 MB to 2.4 MB of WebAssembly.
- Dependencies: zip 9 (fixes RUSTSEC-2026-0367, -0368 and -0369), anyhow 1.0.104 (RUSTSEC-2026-0190), bytes 1.12.1 (RUSTSEC-2026-0007), petgraph 0.8 and thiserror 2. The unused nom dependency is removed.
- The fixture-sweep tests fail when a fixture is not a DEX file, instead of skipping it.

### Added
- Browser smoke tests for the GUI (wasm-bindgen-test in headless Chromium, `nix develop .#wasm-test -c just test-wasm`, and the CI job `gui-smoke`): the app mounts, and the analyzer runs in wasm32 and its findings render.
- A release workflow (`.github/workflows/release.yml`, runbook in RELEASING.md): a `v*` tag publishes `dexpa-core`, `dexpa-analysis` and `dexpa-cli` to crates.io, attaches `dexpa` binaries for Linux (x86_64, aarch64), macOS (aarch64) and Windows (x86_64) with SHA256SUMS and build provenance to the GitHub Release, and redeploys the GUI. A manual run is a rehearsal that creates nothing. `just release` rehearses locally.
- Flake packages: `nix build .#dex-cli` (also the default, so `nix run . -- app.apk` works) and `nix build .#dex-gui`, the static GUI bundle.
- A positive and a negative test for every rule id, against DEX fixtures compiled from documented Java sources in `dex-analysis/tests/fixtures-src` (`nix develop .#fixtures -c just fixtures`, with a pinned R8), and a test that docs/rules.md, the catalogue and the tests list the same ids, CWEs and remediation text.
- `dexpa` reads APKs and multi-dex zips as well as `.dex` files, prints `--format json` (a versioned schema, documented in dex-cli/README.md), and exits with 2 when a finding meets `--fail-on <severity>` (1 on unreadable or unparsable input). Integration tests drive the binary.
- `deny.toml`, with `cargo deny` in CI, and Dependabot updates for Cargo and GitHub Actions.
- A rebuilt CI workflow: fmt, clippy (native and wasm32), tests, docs, cargo deny, MSRV and workflow lint, with SHA-pinned actions.
- `just ci` as the local gate.
- cargo-fuzz targets for the parser, bytecode decoding and the analyzer (`fuzz/`, `just fuzz`), with a pinned-nightly dev shell and a weekly fuzz workflow.
- docs/architecture.md and docs/rules.md, per-crate READMEs, CONTRIBUTING.md and SECURITY.md.

### Removed
- The committed GUI build (`dex-gui/dist`) and two fixtures that were 14-byte "404: Not Found" pages rather than DEX files.

### Fixed
- `M1_HARDCODED_SECRET` no longer reports ordinary text on entropy alone: the entropy trigger only considers token-shaped literals (20+ base64, base64url or hex characters with letters and digits, not made of words, not an algorithm name), so SQL, URLs, messages, identifiers, file names and transformation strings no longer match. On the bundled sample `Annotation_classes.dex` the secret findings drop from 587 to 1 (the remaining one is a `password=` pattern), and on `classes.dex` from 145 to 0; none of the dropped literals was a token.
- `SNK_URL_CONSTRUCTOR` and `SNK_WEBVIEW_LOAD_URL` never fired: they checked the receiver register instead of the URL argument.
- `M10_CIPHER` and `M10_DIGEST` compare whole algorithm tokens instead of substrings, so a string such as `"description"` no longer marks a call as DES and `RSA/ECB/...` is not reported as ECB. Calls using a recognised strong algorithm (AES in a mode other than ECB, ChaCha20, RSA, SHA-2, SHA-3) are no longer reported at Low. A bare `"AES"`, which means ECB on Android, is reported High.
- `M7_WEBVIEW_JS_INTERFACE` and `M7_WEBVIEW_FILE_ACCESS` no longer count `setJavaScriptEnabled(false)` or `setAllowFileAccess(false)` as enabling.
- Three crashes on crafted input are now errors. An out-of-range `class_data_off` and an out-of-range `string_data_off` are rejected by `parse_dex` with `SectionOutOfBounds`. A literal whose 64-byte preview cut fell inside a multibyte character no longer panics the analyzer (thanks to @bilieebiliee1-design, whose PR #1 found it).
- Crafted counts and nesting no longer exhaust memory or the stack: declared counts cap their allocations at the input length, encoded values nest at most 64 deep, a sleb128 is at most five bytes, and `data_off + data_size` saturates.
- Every leb128 read in the parser starts at a bounds-checked offset, and a `method_idx` in class data that points past `method_ids` is an `InvalidIndex` error instead of a panic.
- The GUI page no longer requests a nonexistent `dex-gui.js`.
