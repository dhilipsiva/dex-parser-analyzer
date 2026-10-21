# Rule fixtures

The Java sources behind `dex-analysis/tests/data/rules-positive.dex` and `rules-negative.dex`, which `tests/rules_test.rs` analyses.

- `positive/fixtures/positive/`: one class per rule id, written so that the rule must fire. The class comment names the rule.
- `negative/fixtures/negative/`: a class with the same name per rule id, written so that the rule must not fire. Each is the closest safe variant of its positive, for example `AES/GCM/NoPadding` instead of `DES/ECB/PKCS5Padding`, or `setJavaScriptEnabled(false)` instead of `true`.
- `stubs/`: the few Android framework classes and members the rules match (`Intent`, `Log`, `WebView`, `WebSettings`, `SQLiteDatabase`, ...). They are compiled onto the classpath only, so the fixtures call the real descriptors without the DEX files containing the stubs.

A test looks only at the findings located in its own class (`Lfixtures/positive/<Class>;` or `Lfixtures/negative/<Class>;`), so a fixture cannot satisfy or break another rule's test.

## Rebuilding

```bash
nix develop .#fixtures -c just fixtures
```

The `fixtures` dev shell provides JDK 21 and D8 from a pinned R8 jar (9.5.28, fetched by hash). The recipe compiles the stubs and both fixture sets with `javac --release 17`, converts each set with `D8 --release --min-api 26`, and copies the two `classes.dex` files into `tests/data/`. Rebuilding from unchanged sources gives byte-identical files. Commit the sources and the rebuilt `.dex` files together.

## Adding a rule

Add a positive and a negative class with the same name, rebuild, and add the rule to the `rule_tests!` list in `tests/rules_test.rs`. The rule also needs its entry in `src/rules.rs` and its row and remediation line in `docs/rules.md`; `docs_catalogue_and_tests_agree` fails until all of them agree.
