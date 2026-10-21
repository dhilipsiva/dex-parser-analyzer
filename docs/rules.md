# Rules

Every finding id `dex-analysis` can emit, as of 2026-10-21. The id is `Finding::id`. The OWASP column names the category of the [OWASP Mobile Top 10 (2024)](https://owasp.org/www-project-mobile-top-10/) that the id's `M<n>` prefix refers to; where the prefix and the rule's subject diverge, the row says so. Renaming ids is a breaking change for anyone filtering on them, so any rename waits for the rule-catalogue work before v0.2.0 (TODO P3-DEX-02).

All detection is per method unless stated. There is no interprocedural flow yet, so findings miss data that crosses method boundaries (planned: interprocedural taint summaries).

## Pattern pass (`pattern.rs`)

| Id | Severity | OWASP | CWE | Fires when | False positives and limits |
|---|---|---|---|---|---|
| `M1_HARDCODED_SECRET` | High (a known pattern, or entropy above 4.5); Medium (keyword only, or entropy from the threshold up to 4.5) | M1 Improper Credential Usage | CWE-798 | A string literal referenced by a method matches a secret pattern, contains a secret keyword (`password=`, `secret:`, `api_key`, `token=`, `authorization: basic`, ...) or exceeds the entropy threshold (`secret_min_entropy`, default 3.5) | High-entropy non-secrets: hashes, base64 assets, UUIDs, public keys. One finding per method and trigger. |
| `M5_INSECURE_HTTP` | Medium | M5 Insecure Communication | CWE-319 | A string literal contains `http://`. The location is the referencing method, or unknown when nothing references the literal. | XML namespace URIs (`http://schemas.android.com/...`), documentation links and `localhost` URLs all match. |
| `M10_CIPHER` | High (a weak algorithm string in the same caller); Low (algorithm unknown); Info (Cipher referenced, no call resolved) | M10 Insufficient Cryptography | CWE-327 | A method calls `Cipher.getInstance` | The weak-algorithm test is a case-insensitive substring match (`md5`, `sha1`, `sha-1`, `des`, `rc4`, `rc2`, `ecb`) over every string the calling method references, so an unrelated string such as `"description"` marks the call as DES. An algorithm passed from another method is reported as unknown. |
| `M10_DIGEST` | High (weak algorithm string in the same caller); Low (unknown) | M10 Insufficient Cryptography | CWE-328 | A method calls `MessageDigest.getInstance` | The same substring caveats as `M10_CIPHER`. |

## Structural pass (`structural.rs`)

| Id | Severity | OWASP | CWE | Fires when | False positives and limits |
|---|---|---|---|---|---|
| `M1_RUNTIME_EXEC` | High | prefix M1; the subject (command execution) fits M4 Insufficient Input/Output Validation | CWE-78 | Any invocation of `Runtime.exec` | Fixed, benign commands are reported too; the pass does not check the arguments. |
| `M1_PROCESS_BUILDER` | High | as `M1_RUNTIME_EXEC` | CWE-78 | Any `ProcessBuilder` constructor | As `M1_RUNTIME_EXEC`. |
| `M5_SSL_BYPASS` | High | M5 Insecure Communication | CWE-295 | A method overriding `onReceivedSslError` calls `handler.proceed()` | Conditional proceeds (for example debug builds only) are reported the same way. |
| `M7_WEBVIEW_JS_INTERFACE` | Critical | prefix M7; the subject (WebView configuration) fits M8 Security Misconfiguration | CWE-749 | One method enables JavaScript on a WebView and calls `addJavascriptInterface` | An interface deliberately exposed to trusted, local content only is still reported; split across methods it is missed. |
| `M7_WEBVIEW_FILE_ACCESS` | High | as `M7_WEBVIEW_JS_INTERFACE` | CWE-200 | One method enables JavaScript and `setAllowFileAccess(true)` | As `M7_WEBVIEW_JS_INTERFACE`. |
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
