# Rules

Every finding id `dexpa-analysis` can emit, as of 2026-10-21. The id is `Finding::id`. The OWASP column names the category of the [OWASP Mobile Top 10 (2024)](https://owasp.org/www-project-mobile-top-10/) that the id's `M<n>` prefix refers to; where the prefix and the rule's subject diverge, the row says so. The ids stay as they are for 0.2.0: renaming one breaks anyone filtering on it, and the OWASP column already records the divergences.

Each finding carries the rule's CWE number (`Finding::cwe`) and its remediation text (`Finding::remediation`), both from the catalogue in `dex-analysis/src/rules.rs`. The engine sorts findings by severity, most severe first, and drops exact duplicates before `max_findings` truncates them, so a cap keeps the most severe findings.

Every id below has a positive and a negative test in `dex-analysis/tests/rules_test.rs`, run against DEX fixtures compiled from the Java sources in `dex-analysis/tests/fixtures-src` (see its README). The same test checks that this page, `rules.rs` and the test list name the same ids, CWEs and remediation text.

All detection is per method unless stated. There is no interprocedural flow yet, so findings miss data that crosses method boundaries (planned: interprocedural taint summaries).

## Pattern pass (`pattern.rs`)

| Id | Severity | OWASP | CWE | Fires when | False positives and limits |
|---|---|---|---|---|---|
| `M1_HARDCODED_SECRET` | High (a known pattern, or entropy above 4.5); Medium (keyword only, or entropy from the threshold up to 4.5) | M1 Improper Credential Usage | CWE-798 | A string literal referenced by a method matches a secret pattern, contains a secret keyword (`password=`, `secret:`, `api_key`, `token=`, `authorization: basic`, ...) or exceeds the entropy threshold (`secret_min_entropy`, default 3.5) | High-entropy non-secrets: hashes, base64 assets, UUIDs, public keys. At the default threshold the entropy test also flags ordinary text such as SQL statements, URLs, cipher transformation strings and source file names. One finding per method and trigger. |
| `M5_INSECURE_HTTP` | Medium | M5 Insecure Communication | CWE-319 | A string literal contains `http://`. The location is the referencing method, or unknown when nothing references the literal. | XML namespace URIs (`http://schemas.android.com/...`), documentation links and `localhost` URLs all match. |
| `M10_CIPHER` | High (a weak algorithm string in the same caller); Low (algorithm unknown); Info (Cipher referenced, no call resolved) | M10 Insufficient Cryptography | CWE-327 | A method calls `Cipher.getInstance` and references no recognised strong algorithm | Each string the calling method references is split on `/` and its tokens compared exactly, case-insensitively. Weak: `DES`, `DESede`/`TripleDES`, `RC2`, `RC4`/`ARCFOUR`, `MD2`/`MD4`/`MD5`, `SHA`/`SHA1`/`SHA-1`, and `AES` or `Blowfish` in ECB mode or with no mode (which means ECB on Android). Strong, and not reported: `AES` in any other mode, `ChaCha20`, `RSA` (whose `ECB` token is a naming convention), SHA-2 and SHA-3. The strings are matched per method, not per argument, so a weak name used elsewhere in the same method still marks the call; an algorithm passed in from another method is reported as unknown. |
| `M10_DIGEST` | High (weak algorithm string in the same caller); Low (unknown) | M10 Insufficient Cryptography | CWE-328 | A method calls `MessageDigest.getInstance` and references no recognised strong algorithm | The same classification and limits as `M10_CIPHER`. |

## Structural pass (`structural.rs`)

| Id | Severity | OWASP | CWE | Fires when | False positives and limits |
|---|---|---|---|---|---|
| `M1_RUNTIME_EXEC` | High | prefix M1; the subject (command execution) fits M4 Insufficient Input/Output Validation | CWE-78 | Any invocation of `Runtime.exec` | Fixed, benign commands are reported too; the pass does not check the arguments. |
| `M1_PROCESS_BUILDER` | High | as `M1_RUNTIME_EXEC` | CWE-78 | Any `ProcessBuilder` constructor | As `M1_RUNTIME_EXEC`. |
| `M5_SSL_BYPASS` | High | M5 Insecure Communication | CWE-295 | A method overriding `onReceivedSslError` calls `handler.proceed()` | Conditional proceeds (for example debug builds only) are reported the same way. |
| `M7_WEBVIEW_JS_INTERFACE` | Critical | prefix M7; the subject (WebView configuration) fits M8 Security Misconfiguration | CWE-749 | One method calls `setJavaScriptEnabled` with an argument that is not a literal `false`, and calls `addJavascriptInterface` | An interface deliberately exposed to trusted, local content only is still reported; split across methods it is missed. An argument computed at runtime counts as `true`. |
| `M7_WEBVIEW_FILE_ACCESS` | High | as `M7_WEBVIEW_JS_INTERFACE` | CWE-200 | One method enables JavaScript and calls `setAllowFileAccess`, neither with a literal `false` | As `M7_WEBVIEW_JS_INTERFACE`. |
| `M9_INSECURE_FILE_MODE` | Medium | M9 Insecure Data Storage | CWE-276 | `openFileOutput` is called with a `MODE_WORLD_READABLE`/`MODE_WORLD_WRITEABLE` flag | Low false-positive rate; the flags are deprecated and rejected at runtime on API 24+, so the finding mostly matters for old targets. |

## Taint pass (`taint.rs`)

A finding's id is the sink's id; the source appears in the message and in `extra`. Every taint finding is High, of kind `SensitiveDataLeak`.

Sources: `SRC_INTENT_EXTRA` (`Intent.getStringExtra`), `SRC_SHARED_PREFS` (`SharedPreferences.getString`), `SRC_TELEPHONY_IMEI` (`TelephonyManager.getDeviceId`), `SRC_SECURE_SETTINGS` (`Settings.Secure.getString`), `SRC_BUNDLE_STRING` (`Bundle.getString`).

| Id | Sink | OWASP | CWE |
|---|---|---|---|
| `SNK_LOG_DEBUG` | `Log.d` | M6 Inadequate Privacy Controls | CWE-532 |
| `SNK_LOG_ERROR` | `Log.e` | M6 Inadequate Privacy Controls | CWE-532 |
| `SNK_URL_CONSTRUCTOR` | `java.net.URL.<init>` | M5 Insecure Communication | CWE-200 |
| `SNK_WEBVIEW_LOAD_URL` | `WebView.loadUrl` | M4 Insufficient Input/Output Validation | CWE-200 |
| `SNK_SQLITE_EXEC` | `SQLiteDatabase.execSQL` | M4 Insufficient Input/Output Validation | CWE-89 |
| `SNK_SQLITE_RAW_QUERY` | `SQLiteDatabase.rawQuery` | M4 Insufficient Input/Output Validation | CWE-89 |
| `SNK_RUNTIME_EXEC` | `Runtime.exec` | M4 Insufficient Input/Output Validation | CWE-78 |

False positives and limits:
- Taint propagates through registers inside one method and through simple pass-through calls such as `StringBuilder` chains. A sanitizer or a constant check between source and sink does not clear the taint.
- Flows through fields or across method calls are not followed, so most real leaks that cross a method boundary are missed.

## Remediation

The text each finding carries in `remediation`.

- `M1_HARDCODED_SECRET`: Remove the credential from the app; fetch it at runtime from a backend or the Android Keystore, and rotate the exposed value.
- `M5_INSECURE_HTTP`: Use an https:// endpoint and keep cleartext traffic disabled in the network security config.
- `M10_CIPHER`: Use an authenticated cipher such as AES/GCM/NoPadding with a random IV; never DES, RC4, RC2 or ECB mode.
- `M10_DIGEST`: Use SHA-256 or stronger; never MD5 or SHA-1 for anything security-relevant.
- `M1_RUNTIME_EXEC`: Avoid shelling out; if a command is unavoidable, pass a fixed argument array and never build it from input.
- `M1_PROCESS_BUILDER`: Avoid spawning processes; if unavoidable, use a fixed command and validate every argument.
- `M5_SSL_BYPASS`: Call handler.cancel() in onReceivedSslError, or remove the override, so certificate errors stop the load.
- `M7_WEBVIEW_JS_INTERFACE`: Expose JavaScript interfaces only to trusted, local content, annotate the exposed methods with @JavascriptInterface, and keep JavaScript off otherwise.
- `M7_WEBVIEW_FILE_ACCESS`: Call setAllowFileAccess(false) when JavaScript is enabled, and serve local content through WebViewAssetLoader.
- `M9_INSECURE_FILE_MODE`: Open files with MODE_PRIVATE; share data through a FileProvider or a ContentProvider with permissions.
- `SNK_LOG_DEBUG`: Do not log values that come from intents, preferences or device identifiers; strip debug logging from release builds.
- `SNK_LOG_ERROR`: Do not log values that come from intents, preferences or device identifiers.
- `SNK_URL_CONSTRUCTOR`: Validate an externally supplied URL against an allow-list of schemes and hosts before using it.
- `SNK_WEBVIEW_LOAD_URL`: Load only allow-listed URLs into a WebView; never load a URL taken from an intent as is.
- `SNK_SQLITE_EXEC`: Use parameterised statements (SQLiteStatement or bind arguments); never concatenate input into SQL.
- `SNK_SQLITE_RAW_QUERY`: Pass input through rawQuery's selectionArgs instead of the SQL string.
- `SNK_RUNTIME_EXEC`: Never execute a command built from intent data; use a fixed command and validate arguments against an allow-list.
