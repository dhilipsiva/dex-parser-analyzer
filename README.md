Status: alpha, unreleased (every crate is 0.1.0). The parser and analyzer run on the bundled fixtures; v0.2.0 will publish the crates to crates.io as `dexpa-core`, `dexpa-analysis` and `dexpa-cli`.

# dex-parser-analyzer

A DEX parser and static-analysis toolkit for Android bytecode, in Rust: a library, a CLI, and an in-browser GUI. The GUI runs the parser and analyzer as WebAssembly, so the file you analyze never leaves your machine.

In-browser demo: https://dhilipsiva.github.io/dex-parser-analyzer/

## Workspace

| Crate | What it is |
|---|---|
| [`dex-core`](dex-core/) | Zero-copy DEX parser: header and sections, classes, methods, Dalvik bytecode decoding, control-flow and call graphs, cross-references, serde DTOs, and multi-dex/APK input. |
| [`dex-analysis`](dex-analysis/) | Static-analysis passes over a parsed file (pattern, structural and intraprocedural taint), producing serde-serializable findings. The rules are listed in [docs/rules.md](docs/rules.md). |
| [`dex-cli`](dex-cli/) | A command-line summary of a DEX file plus its findings. |
| `dex-gui` | The Leptos and WebAssembly GUI behind the demo (not published to crates.io). |

How the crates fit together is described in [docs/architecture.md](docs/architecture.md).

## Usage

### Library

```rust
use dex_analysis::{config::AnalysisConfig, engine::analyze_dex};
use dex_core::parse_dex;

fn run(bytes: &[u8]) -> anyhow::Result<()> {
    let dex = parse_dex(bytes)?;
    println!("DEX version {}", dex.header().version);
    let report = analyze_dex(&dex, &AnalysisConfig::default());
    for finding in report.findings {
        println!("{:?} {}: {}", finding.severity, finding.id, finding.message);
    }
    Ok(())
}
```

### CLI

```bash
cargo run -p dex-cli -- path/to/classes.dex --max-findings 25
```

The CLI prints the header summary, the ten classes with the most methods, and the findings. Its options are documented in [dex-cli/README.md](dex-cli/README.md).

### GUI

```bash
nix develop
cd dex-gui && trunk serve
```

## Building and testing

Development happens inside the flake's dev shell, which pins the toolchain in `rust-toolchain.toml` (Rust 1.99.1, with the wasm32 target, trunk, just and cargo-deny):

```bash
nix develop
just ci        # fmt, clippy (native and wasm32), tests, docs, cargo deny
```

`just --list` shows the individual recipes. Without Nix, any rustup install picks up `rust-toolchain.toml`. The project's conventions are in [products/CONVENTIONS.md](https://github.com/dhilipsiva/products/blob/main/CONVENTIONS.md). Contributions: [CONTRIBUTING.md](CONTRIBUTING.md). Security reports: [SECURITY.md](SECURITY.md). Changes: [CHANGELOG.md](CHANGELOG.md).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option. The DEX and APK test fixtures under `dex-core/tests/data/` derive from the Androguard test corpus and keep its Apache-2.0 terms; see [NOTICE](NOTICE).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
