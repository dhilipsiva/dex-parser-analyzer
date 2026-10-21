# Fuzzing

cargo-fuzz (libFuzzer) targets for the code that reads untrusted bytes:

| Target | What it exercises |
|---|---|
| `parse_dex` | `dex_core::parse_dex`, every string, and the overview DTO |
| `decode_instructions` | bytecode decoding and the CFG for every method, the call graph and the cross-references |
| `analyze_dex_bytes` | the whole analyzer, as the GUI runs it on an upload |

Fuzzing needs a nightly toolchain, so it has its own dev shell and its own workspace (`fuzz/Cargo.toml`):

```bash
nix develop .#fuzz
just fuzz parse_dex 600        # target, then seconds; seeds come from the small fixtures in dex-core/tests/data
```

A crash leaves its input under `fuzz/artifacts/<target>/`. Reproduce it with `cargo fuzz run <target> <file>`, fix it, and add a regression test that builds the input (see `dex-core/tests/hostile_input_test.rs`) before deleting the artifact. `.github/workflows/fuzz.yml` runs each target for 20 minutes every week and keeps any crash artifacts.
