//! The rule catalogue: the CWE and the remediation for every finding id the passes emit. It is
//! the code-side twin of docs/rules.md; the engine attaches both to each finding.

/// One rule's catalogue entry.
#[derive(Debug)]
pub struct Rule {
    /// The `Finding::id` the rule emits.
    pub id: &'static str,
    /// The CWE the rule maps to.
    pub cwe: u32,
    /// What to change in the app to resolve the finding.
    pub remediation: &'static str,
}

/// Every rule id `dexpa-analysis` can emit.
pub const RULES: &[Rule] = &[
    Rule {
        id: "M1_HARDCODED_SECRET",
        cwe: 798,
        remediation: "Remove the credential from the app; fetch it at runtime from a backend or the Android Keystore, and rotate the exposed value.",
    },
    Rule {
        id: "M5_INSECURE_HTTP",
        cwe: 319,
        remediation: "Use an https:// endpoint and keep cleartext traffic disabled in the network security config.",
    },
    Rule {
        id: "M10_CIPHER",
        cwe: 327,
        remediation: "Use an authenticated cipher such as AES/GCM/NoPadding with a random IV; never DES, RC4, RC2 or ECB mode.",
    },
    Rule {
        id: "M10_DIGEST",
        cwe: 328,
        remediation: "Use SHA-256 or stronger; never MD5 or SHA-1 for anything security-relevant.",
    },
    Rule {
        id: "M1_RUNTIME_EXEC",
        cwe: 78,
        remediation: "Avoid shelling out; if a command is unavoidable, pass a fixed argument array and never build it from input.",
    },
    Rule {
        id: "M1_PROCESS_BUILDER",
        cwe: 78,
        remediation: "Avoid spawning processes; if unavoidable, use a fixed command and validate every argument.",
    },
    Rule {
        id: "M5_SSL_BYPASS",
        cwe: 295,
        remediation: "Call handler.cancel() in onReceivedSslError, or remove the override, so certificate errors stop the load.",
    },
    Rule {
        id: "M7_WEBVIEW_JS_INTERFACE",
        cwe: 749,
        remediation: "Expose JavaScript interfaces only to trusted, local content, annotate the exposed methods with @JavascriptInterface, and keep JavaScript off otherwise.",
    },
    Rule {
        id: "M7_WEBVIEW_FILE_ACCESS",
        cwe: 200,
        remediation: "Call setAllowFileAccess(false) when JavaScript is enabled, and serve local content through WebViewAssetLoader.",
    },
    Rule {
        id: "M9_INSECURE_FILE_MODE",
        cwe: 276,
        remediation: "Open files with MODE_PRIVATE; share data through a FileProvider or a ContentProvider with permissions.",
    },
    Rule {
        id: "SNK_LOG_DEBUG",
        cwe: 532,
        remediation: "Do not log values that come from intents, preferences or device identifiers; strip debug logging from release builds.",
    },
    Rule {
        id: "SNK_LOG_ERROR",
        cwe: 532,
        remediation: "Do not log values that come from intents, preferences or device identifiers.",
    },
    Rule {
        id: "SNK_URL_CONSTRUCTOR",
        cwe: 200,
        remediation: "Validate an externally supplied URL against an allow-list of schemes and hosts before using it.",
    },
    Rule {
        id: "SNK_WEBVIEW_LOAD_URL",
        cwe: 200,
        remediation: "Load only allow-listed URLs into a WebView; never load a URL taken from an intent as is.",
    },
    Rule {
        id: "SNK_SQLITE_EXEC",
        cwe: 89,
        remediation: "Use parameterised statements (SQLiteStatement or bind arguments); never concatenate input into SQL.",
    },
    Rule {
        id: "SNK_SQLITE_RAW_QUERY",
        cwe: 89,
        remediation: "Pass input through rawQuery's selectionArgs instead of the SQL string.",
    },
    Rule {
        id: "SNK_RUNTIME_EXEC",
        cwe: 78,
        remediation: "Never execute a command built from intent data; use a fixed command and validate arguments against an allow-list.",
    },
];

/// The catalogue entry for a finding id, if it has one.
pub fn lookup(id: &str) -> Option<&'static Rule> {
    RULES.iter().find(|rule| rule.id == id)
}
