# dexpa-cli

`dexpa`, the command-line front end of [dex-parser-analyzer](../README.md). It parses a `.dex` file, or every `classes*.dex` in an APK, with [`dexpa-core`](../dex-core/), runs every [`dexpa-analysis`](../dex-analysis/) pass with the default configuration, and prints the results.

```
dexpa [OPTIONS] <FILE>

Arguments:
  <FILE>  A `.dex` file, or an `.apk`/zip whose `classes*.dex` entries are analysed in name order

Options:
      --format <FORMAT>       Output format [default: text] [possible values: text, json]
      --max-findings <COUNT>  At most this many findings per DEX file
      --fail-on <SEVERITY>    Exit with status 2 when any finding is at or above this severity [possible values: info, low, medium, high, critical]
  -h, --help                  Print help
  -V, --version               Print version
```

Install from the workspace with `cargo install --path dex-cli`, or run it in place:

```bash
cargo run -p dexpa-cli -- dex-core/tests/data/classes.dex --max-findings 25
cargo run -p dexpa-cli -- app.apk --format json --fail-on high
```

## Exit codes

| Code | Meaning |
|---|---|
| 0 | The input was analysed, and no finding met `--fail-on` (or `--fail-on` was not given). |
| 1 | The input could not be read, is neither a DEX file nor a zip with `classes*.dex` entries, or failed to parse. The error is on stderr. |
| 2 | The input was analysed, and at least one finding is at or above the `--fail-on` severity. The report is still printed. |

So `dexpa app.apk --fail-on high` in CI fails the job on any High or Critical finding.

## Text output

For each DEX file in the input:
1. the header summary: version, checksum, file size, and the string, type, field, method and class counts;
2. the ten classes with the most methods;
3. the analysis counts and the findings, most severe first (severity, rule id, location, message).

## JSON output (`--format json`)

One document. `schema_version` changes only with a breaking change to the shape below; adding a field is not breaking.

```json
{
  "schema_version": 1,
  "input": "app.apk",
  "dex_files": [
    {
      "index": 0,
      "summary": {
        "version": "35",
        "checksum": 3692947289,
        "file_size": 614592,
        "string_count": 4329,
        "type_count": 596,
        "field_count": 865,
        "method_count": 3602,
        "class_count": 340
      },
      "top_classes": [
        { "descriptor": "Landroid/support/v4/app/Fragment;", "method_count": 95 }
      ],
      "report": {
        "findings": [
          {
            "id": "M1_HARDCODED_SECRET",
            "kind": "HardcodedSecret",
            "severity": "Medium",
            "location": {
              "class_descriptor": "Lcom/example/Foo;",
              "method_name": "bar",
              "method_signature": "(Ljava/lang/String;)V",
              "pc": null
            },
            "message": "Literal `...` matched entropy pattern",
            "cwe": 798,
            "remediation": "Remove the credential from the app; ...",
            "extra": { "pattern": "entropy", "literal": "..." }
          }
        ],
        "stats": { "method_count": 3602, "string_count": 4329, "instruction_count": 26192, "elapsed_ms": 66 }
      }
    }
  ]
}
```

- `dex_files` holds one entry for a `.dex` input, and one per `classes*.dex` for an APK, in entry-name order; `index` is the position.
- `top_classes` lists at most ten classes, most methods first.
- `report` is the `dexpa-analysis` `AnalysisReport` as its serde types serialise it. Findings are listed most severe first. The rule ids, their severities, CWEs and remediation texts are in [docs/rules.md](../docs/rules.md); `cwe` and `remediation` are `null` for an id outside that catalogue.

Licensed under MIT OR Apache-2.0.
