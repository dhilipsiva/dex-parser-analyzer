# dex-cli

The command-line front end of [dex-parser-analyzer](../README.md). It parses one `.dex` file with [`dex-core`](../dex-core/), runs every [`dex-analysis`](../dex-analysis/) pass with the default configuration, and prints the results.

```
dex-cli [OPTIONS] <DEX>

Arguments:
  <DEX>  Path to the `.dex` file to inspect

Options:
      --max-findings <COUNT>  Optional maximum number of findings to print
  -h, --help                  Print help
  -V, --version               Print version
```

From the workspace:

```bash
cargo run -p dex-cli -- dex-core/tests/data/classes.dex --max-findings 25
```

The output, in order:
1. the header summary: version, checksum, file size, and the string, type, field, method and class counts;
2. the ten classes with the most methods;
3. the analysis counts and the findings (severity, rule id, location, message).

Exit codes: 0 on success; 1 when the file cannot be read or parsed, with the error on stderr.

Input is a single `.dex` file. APK and multi-dex input, a `--format json` mode and `--fail-on <severity>` for CI gating are planned for v0.2.0, when the binary is renamed `dexpa`.

Licensed under MIT OR Apache-2.0.
