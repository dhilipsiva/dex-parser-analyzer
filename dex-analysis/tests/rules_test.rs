//! One positive and one negative test per rule id in docs/rules.md.
//!
//! The fixtures are compiled from `tests/fixtures-src` (`just fixtures` inside
//! `nix develop .#fixtures`): `rules-positive.dex` holds a class per rule that must trigger it, and
//! `rules-negative.dex` a class of the same name that must not. Each test looks only at findings
//! located in its own class, so one fixture cannot satisfy or break another rule's test.

use std::{collections::BTreeMap, sync::OnceLock};

use dexpa_analysis::{
    config::AnalysisConfig,
    engine::analyze_dex_bytes,
    model::{AnalysisReport, Finding, Severity},
    rules::RULES,
};

fn report(kind: &str) -> &'static AnalysisReport {
    static POSITIVE: OnceLock<AnalysisReport> = OnceLock::new();
    static NEGATIVE: OnceLock<AnalysisReport> = OnceLock::new();
    let cell = if kind == "positive" {
        &POSITIVE
    } else {
        &NEGATIVE
    };
    cell.get_or_init(|| {
        let path = format!("{}/tests/data/rules-{kind}.dex", env!("CARGO_MANIFEST_DIR"));
        let bytes = std::fs::read(&path).unwrap_or_else(|err| panic!("read {path}: {err}"));
        let config = AnalysisConfig {
            max_findings: None,
            ..Default::default()
        };
        analyze_dex_bytes(&bytes, &config).expect("fixture analyses")
    })
}

fn findings_in(kind: &str, class: &str, rule: &str) -> Vec<&'static Finding> {
    let descriptor = format!("Lfixtures/{kind}/{class};");
    report(kind)
        .findings
        .iter()
        .filter(|f| f.id == rule && f.location.class_descriptor == descriptor)
        .collect()
}

macro_rules! rule_tests {
    ($($rule:literal => $class:literal, $positive:ident, $negative:ident;)*) => {
        /// Every rule id that has a positive and a negative test below.
        const TESTED_RULES: &[&str] = &[$($rule),*];
        $(
            #[test]
            fn $positive() {
                let found = findings_in("positive", $class, $rule);
                assert!(!found.is_empty(), "{} did not fire on fixtures.positive.{}", $rule, $class);
                for finding in found {
                    assert!(finding.cwe.is_some(), "{} finding lacks a CWE", $rule);
                    assert!(finding.remediation.is_some(), "{} finding lacks remediation text", $rule);
                }
            }

            #[test]
            fn $negative() {
                let found = findings_in("negative", $class, $rule);
                assert!(found.is_empty(), "{} fired on fixtures.negative.{}: {:?}", $rule, $class, found);
            }
        )*
    };
}

rule_tests! {
    "M1_HARDCODED_SECRET" => "HardcodedSecret", m1_hardcoded_secret_positive, m1_hardcoded_secret_negative;
    "M5_INSECURE_HTTP" => "InsecureHttp", m5_insecure_http_positive, m5_insecure_http_negative;
    "M10_CIPHER" => "WeakCipher", m10_cipher_positive, m10_cipher_negative;
    "M10_DIGEST" => "WeakDigest", m10_digest_positive, m10_digest_negative;
    "M1_RUNTIME_EXEC" => "RuntimeExec", m1_runtime_exec_positive, m1_runtime_exec_negative;
    "M1_PROCESS_BUILDER" => "ProcessBuilderExec", m1_process_builder_positive, m1_process_builder_negative;
    "M5_SSL_BYPASS" => "SslBypass", m5_ssl_bypass_positive, m5_ssl_bypass_negative;
    "M7_WEBVIEW_JS_INTERFACE" => "WebViewJsInterface", m7_webview_js_interface_positive, m7_webview_js_interface_negative;
    "M7_WEBVIEW_FILE_ACCESS" => "WebViewFileAccess", m7_webview_file_access_positive, m7_webview_file_access_negative;
    "M9_INSECURE_FILE_MODE" => "InsecureFileMode", m9_insecure_file_mode_positive, m9_insecure_file_mode_negative;
    "SNK_LOG_DEBUG" => "LogDebugLeak", snk_log_debug_positive, snk_log_debug_negative;
    "SNK_LOG_ERROR" => "LogErrorLeak", snk_log_error_positive, snk_log_error_negative;
    "SNK_URL_CONSTRUCTOR" => "UrlLeak", snk_url_constructor_positive, snk_url_constructor_negative;
    "SNK_WEBVIEW_LOAD_URL" => "WebViewLoadLeak", snk_webview_load_url_positive, snk_webview_load_url_negative;
    "SNK_SQLITE_EXEC" => "SqliteExecLeak", snk_sqlite_exec_positive, snk_sqlite_exec_negative;
    "SNK_SQLITE_RAW_QUERY" => "SqliteRawQueryLeak", snk_sqlite_raw_query_positive, snk_sqlite_raw_query_negative;
    "SNK_RUNTIME_EXEC" => "RuntimeExecLeak", snk_runtime_exec_positive, snk_runtime_exec_negative;
}

#[test]
fn entropy_alone_reports_a_random_token() {
    let found = findings_in("positive", "EntropySecret", "M1_HARDCODED_SECRET");
    assert!(
        found.iter().any(|f| f.extra["pattern"] == "entropy"),
        "the random token was not reported: {found:?}"
    );
}

/// The negative fixture is full of names, descriptors, SQL, URLs and transformation strings; none
/// of them is a secret, wherever it sits.
#[test]
fn no_secret_is_reported_anywhere_in_the_negative_fixture() {
    let found: Vec<_> = report("negative")
        .findings
        .iter()
        .filter(|f| f.id == "M1_HARDCODED_SECRET")
        .collect();
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn weak_algorithms_are_reported_high() {
    for (class, rule) in [("WeakCipher", "M10_CIPHER"), ("WeakDigest", "M10_DIGEST")] {
        let found = findings_in("positive", class, rule);
        assert!(
            found.iter().all(|f| f.severity == Severity::High),
            "{rule} on {class}"
        );
    }
}

#[test]
fn findings_are_sorted_by_severity_and_unique() {
    let findings = &report("positive").findings;
    assert!(
        findings.windows(2).all(|w| w[0].severity >= w[1].severity),
        "findings are not in descending severity order"
    );
    let mut keys: Vec<_> = findings
        .iter()
        .map(|f| {
            (
                &f.id,
                &f.location.class_descriptor,
                &f.location.method_name,
                f.location.pc,
                &f.message,
            )
        })
        .collect();
    let total = keys.len();
    keys.sort();
    keys.dedup();
    assert_eq!(keys.len(), total, "duplicate findings");
}

#[test]
fn max_findings_keeps_the_most_severe() {
    let bytes = std::fs::read(format!(
        "{}/tests/data/rules-positive.dex",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("read fixture");
    let config = AnalysisConfig {
        max_findings: Some(1),
        ..Default::default()
    };
    let report = analyze_dex_bytes(&bytes, &config).expect("fixture analyses");
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].severity, Severity::Critical);
}

/// docs/rules.md, the `rules.rs` catalogue and the tests above name the same ids, with the same CWE
/// and remediation text.
#[test]
fn docs_catalogue_and_tests_agree() {
    let path = format!("{}/../docs/rules.md", env!("CARGO_MANIFEST_DIR"));
    let docs = std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {path}: {err}"));
    let mut documented_cwe = BTreeMap::new();
    let mut documented_remediation = BTreeMap::new();
    for line in docs.lines() {
        if let Some(row) = line.strip_prefix("| `") {
            let id = row.split('`').next().expect("id cell");
            let cwe = row
                .split('|')
                .map(str::trim)
                .find_map(|cell| cell.strip_prefix("CWE-"))
                .unwrap_or_else(|| panic!("{id}: no CWE cell"));
            let cwe: u32 = cwe.parse().unwrap_or_else(|_| panic!("{id}: CWE-{cwe}"));
            assert!(
                documented_cwe.insert(id, cwe).is_none(),
                "{id} documented twice"
            );
        } else if let Some(entry) = line.strip_prefix("- `")
            && let Some((id, text)) = entry.split_once("`: ")
        {
            documented_remediation.insert(id, text);
        }
    }
    let catalogue_cwe: BTreeMap<_, _> = RULES.iter().map(|rule| (rule.id, rule.cwe)).collect();
    let catalogue_remediation: BTreeMap<_, _> = RULES
        .iter()
        .map(|rule| (rule.id, rule.remediation))
        .collect();
    assert_eq!(
        documented_cwe, catalogue_cwe,
        "docs/rules.md tables vs rules.rs"
    );
    assert_eq!(
        documented_remediation, catalogue_remediation,
        "docs/rules.md remediation vs rules.rs"
    );
    let mut tested: Vec<_> = TESTED_RULES.to_vec();
    tested.sort_unstable();
    let catalogued: Vec<_> = catalogue_cwe.keys().copied().collect();
    assert_eq!(tested, catalogued, "rules with tests vs rules.rs");
}
