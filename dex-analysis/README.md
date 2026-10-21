# dexpa-analysis

Static-analysis passes over a DEX file parsed by [`dexpa-core`](../dex-core/). It is part of [dex-parser-analyzer](../README.md).

`engine::analyze_dex` runs three passes, each enabled by a flag in `config::AnalysisConfig`, and returns an `AnalysisReport` of findings plus counts:

- **pattern** (`pattern.rs`): hardcoded secrets (regex, keyword and entropy heuristics over string literals), `http://` literals, and weak-crypto calls, found by correlating strings with `Cipher`/`MessageDigest.getInstance` invocations through `dexpa_core::graphs::build_xrefs`.
- **structural** (`structural.rs`): WebView misconfiguration (JavaScript plus a JavaScript interface or file access), SSL-error handlers that call `proceed()`, world-readable/writable file modes, and `Runtime.exec`/`ProcessBuilder` use.
- **taint** (`taint.rs`): an intraprocedural forward data-flow pass (`data_flow.rs`) that tracks values from Android sources (Intent extras, SharedPreferences, device identifiers, Bundle strings) into sinks (logging, `URL`, `WebView.loadUrl`, SQLite, `Runtime.exec`) within one method.

Every rule id, with its severity, its OWASP and CWE mapping, its remediation and its known false positives, is in [docs/rules.md](../docs/rules.md); `rules::RULES` is the same catalogue in code. Each finding carries its rule's `cwe` and `remediation`, and the report lists findings most severe first, de-duplicated, before `max_findings` cuts it.

Each rule has a positive and a negative test (`tests/rules_test.rs`) against DEX fixtures compiled from the Java sources in [`tests/fixtures-src`](tests/fixtures-src/README.md).

```rust
use dexpa_analysis::{config::AnalysisConfig, engine::analyze_dex};
use dexpa_core::parse_dex;

fn findings(bytes: &[u8]) -> Result<usize, dexpa_core::DexError> {
    let dex = parse_dex(bytes)?;
    let config = AnalysisConfig { max_findings: Some(100), ..AnalysisConfig::default() };
    let report = analyze_dex(&dex, &config);
    Ok(report.findings.len())
}
```

Findings and the report are `serde` types, so a report round-trips through JSON. On `wasm32`, `ffi.rs` exports `analyze_dex` and `load_and_analyze_dex` through `wasm-bindgen`; the GUI uses them.

Defaults: all three passes on, a secret-entropy threshold of 3.5, and at most 500 findings.

Licensed under MIT OR Apache-2.0.
