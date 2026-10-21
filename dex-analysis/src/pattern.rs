use std::collections::{HashMap, HashSet};

use aho_corasick::AhoCorasick;
use dexpa_core::{
    format::{MethodIdx, StringIdx},
    graphs,
    model::DexFile,
};
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::json;

use crate::{
    config::AnalysisConfig,
    model::{Finding, Location, Severity, VulnerabilityKind, describe_method},
};

const SECRET_MIN_LENGTH: usize = 8;

/// The shortest literal the entropy trigger considers; shorter strings cannot reach a useful
/// entropy (20 distinct characters give at most log2(20) = 4.3 bits).
const ENTROPY_MIN_LENGTH: usize = 20;

/// Identifiers (`TaskStackBuilderApi16Impl`, `Base_V7_Theme_AppCompat`) are runs of same-case
/// letters; in random base64 fewer than one adjacent pair in six is two lower-case letters. A
/// literal whose share of same-case letter pairs reaches this is treated as words, not a token.
const ENTROPY_MAX_WORD_PAIRS: f32 = 0.4;

static SECRET_REGEXES: Lazy<Vec<(&str, Regex)>> = Lazy::new(|| {
    vec![
        ("aws_access_key", Regex::new(r"AKIA[0-9A-Z]{16}").unwrap()),
        (
            "private_key",
            Regex::new(r"BEGIN [A-Z ]+PRIVATE KEY").unwrap(),
        ),
        (
            "slack_token",
            Regex::new(r"xox[baprs]-[A-Za-z0-9-]{10,}").unwrap(),
        ),
        (
            "jwt",
            Regex::new(r"eyJ[a-zA-Z0-9_-]{20,}\.[a-zA-Z0-9_-]{20,}\.[a-zA-Z0-9_-]{10,}").unwrap(),
        ),
        (
            "password_assignment",
            Regex::new(r"(?i)password\s*[:=]").unwrap(),
        ),
    ]
});

static SECRET_KEYWORDS: Lazy<AhoCorasick> = Lazy::new(|| {
    AhoCorasick::new([
        "password=",
        "password:",
        "secret=",
        "secret:",
        "api_key",
        "token=",
        "token:",
        "authorization: basic",
    ])
    .expect("aho-corasick build")
});

/// Algorithm names (the first `/`-separated token, lower-cased) that are weak whatever the mode.
static WEAK_ALGORITHMS: &[(&str, &str)] = &[
    ("md2", "MD2"),
    ("md4", "MD4"),
    ("md5", "MD5"),
    ("sha", "SHA1"),
    ("sha1", "SHA1"),
    ("sha-1", "SHA1"),
    ("des", "DES"),
    ("desede", "3DES"),
    ("tripledes", "3DES"),
    ("rc2", "RC2"),
    ("rc4", "RC4"),
    ("arcfour", "RC4"),
];

/// Symmetric block ciphers, which are weak in ECB mode; a bare name such as `AES` means ECB on
/// Android.
static BLOCK_CIPHERS: &[&str] = &["aes", "aes_128", "aes_192", "aes_256", "blowfish"];

/// Algorithm names recognised as acceptable; a call using one of these is not reported.
static STRONG_ALGORITHMS: &[&str] = &[
    "chacha20",
    "chacha20-poly1305",
    "rsa",
    "sha-224",
    "sha-256",
    "sha-384",
    "sha-512",
    "sha224",
    "sha256",
    "sha384",
    "sha512",
    "sha3-224",
    "sha3-256",
    "sha3-384",
    "sha3-512",
];

/// What a string literal says about the algorithm a crypto call uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AlgorithmClass {
    Weak(&'static str),
    Strong,
}

struct MethodPattern {
    id: &'static str,
    class: &'static str,
    name: &'static str,
    signature: &'static str,
    description: &'static str,
}

static CRYPTO_SINKS: &[MethodPattern] = &[
    MethodPattern {
        id: "M10_CIPHER",
        class: "Ljavax/crypto/Cipher;",
        name: "getInstance",
        signature: "(Ljava/lang/String;)Ljavax/crypto/Cipher;",
        description: "javax.crypto.Cipher.getInstance",
    },
    MethodPattern {
        id: "M10_DIGEST",
        class: "Ljava/security/MessageDigest;",
        name: "getInstance",
        signature: "(Ljava/lang/String;)Ljava/security/MessageDigest;",
        description: "java.security.MessageDigest.getInstance",
    },
];

/// Execute pattern based checks (secrets + weak crypto heuristics).
pub fn run_pattern_checks(
    dex: &DexFile<'_>,
    config: &AnalysisConfig,
    findings: &mut Vec<Finding>,
    _stats: &mut crate::model::AnalysisStats,
) {
    let Ok(xrefs) = graphs::build_xrefs(dex) else {
        return;
    };
    let string_to_methods = build_string_to_methods(&xrefs);
    detect_hardcoded_secrets(dex, config, findings, &string_to_methods);
    detect_insecure_http(dex, findings, &string_to_methods);
    detect_weak_crypto(dex, findings, &xrefs, &string_to_methods);
}

fn detect_hardcoded_secrets(
    dex: &DexFile<'_>,
    config: &AnalysisConfig,
    findings: &mut Vec<Finding>,
    string_map: &HashMap<u32, Vec<u32>>,
) {
    let mut emitted = HashSet::new();
    for idx in 0..dex.string_count() {
        let string_idx = StringIdx::new(idx as u32);
        let Some(value) = dex.string(string_idx) else {
            continue;
        };
        if value.trim().is_empty() {
            continue;
        }
        if value.len() < SECRET_MIN_LENGTH && SECRET_KEYWORDS.find(value.as_bytes()).is_none() {
            continue;
        }

        let trigger = match classify_secret(value, config.secret_min_entropy) {
            Some(trigger) => trigger,
            None => continue,
        };

        let methods = string_map.get(&(idx as u32));
        if let Some(methods) = methods {
            for method in methods {
                if !emitted.insert((*method, trigger.label())) {
                    continue;
                }
                findings.push(Finding {
                    id: "M1_HARDCODED_SECRET".into(),
                    kind: VulnerabilityKind::HardcodedSecret,
                    severity: trigger.severity(),
                    location: Location::from_method(dex, MethodIdx::new(*method), None),
                    message: format!(
                        "Literal `{}` matched {} pattern",
                        preview_literal(value),
                        trigger.label()
                    ),
                    cwe: None,
                    remediation: None,
                    extra: json!({
                        "pattern": trigger.label(),
                        "literal": value,
                    }),
                });
            }
        } else if emitted.insert((u32::MAX, trigger.label())) {
            findings.push(Finding {
                id: "M1_HARDCODED_SECRET".into(),
                kind: VulnerabilityKind::HardcodedSecret,
                severity: trigger.severity(),
                location: Location::unknown(),
                message: format!(
                    "Literal `{}` matched {} pattern",
                    preview_literal(value),
                    trigger.label()
                ),
                cwe: None,
                remediation: None,
                extra: json!({
                    "pattern": trigger.label(),
                    "literal": value,
                }),
            });
        }
    }
}

fn detect_weak_crypto(
    dex: &DexFile<'_>,
    findings: &mut Vec<Finding>,
    xrefs: &graphs::Xrefs,
    string_to_methods: &HashMap<u32, Vec<u32>>,
) {
    let sink_lookup = build_sink_lookup(dex);
    if sink_lookup.is_empty() {
        if references_cipher_descriptor(dex) {
            findings.push(Finding {
                id: "M10_CIPHER".into(),
                kind: VulnerabilityKind::WeakCrypto,
                severity: Severity::Info,
                location: Location::unknown(),
                message: "Cipher APIs referenced but no invocations were resolved".to_string(),
                cwe: None,
                remediation: None,
                extra: json!({ "reason": "cipher_descriptor" }),
            });
        }
        return;
    }
    let method_to_strings = invert_string_map(string_to_methods);
    let algorithm_strings = classify_algorithm_strings(dex);
    let mut emitted = HashSet::new();
    let mut produced = false;

    for (caller, callee) in &xrefs.method_calls {
        let Some(sink) = sink_lookup.get(callee) else {
            continue;
        };
        let classes: Vec<AlgorithmClass> = method_to_strings
            .get(caller)
            .map(|strings| {
                strings
                    .iter()
                    .filter_map(|idx| algorithm_strings.get(idx).copied())
                    .collect()
            })
            .unwrap_or_default();
        let algorithm = classes.iter().find_map(|class| match class {
            AlgorithmClass::Weak(label) => Some(*label),
            AlgorithmClass::Strong => None,
        });
        if algorithm.is_none() && classes.contains(&AlgorithmClass::Strong) {
            continue;
        }
        let severity = if algorithm.is_some() {
            Severity::High
        } else {
            Severity::Low
        };
        let key = (*caller, sink.id, algorithm.unwrap_or("UNKNOWN"));
        if !emitted.insert(key) {
            continue;
        }
        produced = true;
        let message = if let Some(alg) = algorithm {
            format!("{} invoked with weak algorithm `{alg}`", sink.description)
        } else {
            format!(
                "{} invoked (algorithm unknown – review manually)",
                sink.description
            )
        };
        findings.push(Finding {
            id: sink.id.into(),
            kind: VulnerabilityKind::WeakCrypto,
            severity,
            location: Location::from_method(dex, MethodIdx::new(*caller), None),
            message,
            cwe: None,
            remediation: None,
            extra: json!({
                "sink": sink.description,
                "algorithm": algorithm.unwrap_or("unknown"),
            }),
        });
    }

    if !produced && references_cipher_descriptor(dex) {
        findings.push(Finding {
            id: "M10_CIPHER".into(),
            kind: VulnerabilityKind::WeakCrypto,
            severity: Severity::Info,
            location: Location::unknown(),
            message: "Cipher APIs referenced but no invocations were resolved".to_string(),
            cwe: None,
            remediation: None,
            extra: json!({ "reason": "cipher_descriptor" }),
        });
    }
}

fn detect_insecure_http(
    dex: &DexFile<'_>,
    findings: &mut Vec<Finding>,
    string_map: &HashMap<u32, Vec<u32>>,
) {
    let mut emitted = HashSet::new();
    for idx in 0..dex.string_count() {
        let idx_u32 = idx as u32;
        let Some(value) = dex.string(StringIdx::new(idx_u32)) else {
            continue;
        };
        if !value.contains("http://") {
            continue;
        }
        let message = format!(
            "Insecure HTTP literal `{}` detected",
            preview_literal(value)
        );
        if let Some(methods) = string_map.get(&idx_u32) {
            for method in methods {
                if !emitted.insert((*method, idx_u32)) {
                    continue;
                }
                findings.push(Finding {
                    id: "M5_INSECURE_HTTP".into(),
                    kind: VulnerabilityKind::InsecureCommunication,
                    severity: Severity::Medium,
                    location: Location::from_method(dex, MethodIdx::new(*method), None),
                    message: message.clone(),
                    cwe: None,
                    remediation: None,
                    extra: json!({
                        "literal": value,
                        "kind": "http_literal",
                    }),
                });
            }
        } else if emitted.insert((u32::MAX, idx_u32)) {
            findings.push(Finding {
                id: "M5_INSECURE_HTTP".into(),
                kind: VulnerabilityKind::InsecureCommunication,
                severity: Severity::Medium,
                location: Location::unknown(),
                message: message.clone(),
                cwe: None,
                remediation: None,
                extra: json!({
                    "literal": value,
                    "kind": "http_literal",
                }),
            });
        }
    }
}

fn build_sink_lookup<'a>(dex: &'a DexFile<'_>) -> HashMap<u32, &'a MethodPattern> {
    let mut lookup = HashMap::new();
    for idx in 0..dex.method_count() {
        let method_idx = MethodIdx::new(idx as u32);
        let summary = describe_method(dex, method_idx);
        for sink in CRYPTO_SINKS {
            if summary.class == sink.class
                && summary.name == sink.name
                && summary.signature == sink.signature
            {
                lookup.insert(idx as u32, sink);
            }
        }
    }
    lookup
}

fn classify_secret(value: &str, entropy_threshold: f32) -> Option<SecretTrigger> {
    if let Some((label, _)) = SECRET_REGEXES
        .iter()
        .find(|(_, regex)| regex.is_match(value))
    {
        return Some(SecretTrigger::Pattern(label));
    }
    if SECRET_KEYWORDS.find(value.as_bytes()).is_some() {
        return Some(SecretTrigger::Keyword);
    }
    if is_token_shaped(value) {
        let entropy = shannon_entropy(value);
        if entropy >= entropy_threshold {
            return Some(SecretTrigger::Entropy(entropy));
        }
    }
    None
}

/// Whether a literal looks like a generated token (an API key, a base64 or hex secret) rather
/// than text: long enough, only base64/base64url/hex characters with optional `=` padding, both
/// letters and digits, not made of words, and not a cipher or digest name. URLs, SQL, file names,
/// descriptors and identifiers fail one of these tests.
fn is_token_shaped(value: &str) -> bool {
    if value.len() < ENTROPY_MIN_LENGTH {
        return false;
    }
    let body = value.trim_end_matches('=');
    if value.len() - body.len() > 2
        || !body
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'/' | b'_' | b'-'))
    {
        return false;
    }
    if !body.bytes().any(|b| b.is_ascii_digit()) || !body.bytes().any(|b| b.is_ascii_alphabetic()) {
        return false;
    }
    word_pair_share(body) < ENTROPY_MAX_WORD_PAIRS && classify_algorithm(body).is_none()
}

/// The larger of the shares of adjacent lower-lower and upper-upper letter pairs.
fn word_pair_share(value: &str) -> f32 {
    let bytes = value.as_bytes();
    let (mut lower, mut upper) = (0usize, 0usize);
    for pair in bytes.windows(2) {
        lower += usize::from(pair[0].is_ascii_lowercase() && pair[1].is_ascii_lowercase());
        upper += usize::from(pair[0].is_ascii_uppercase() && pair[1].is_ascii_uppercase());
    }
    lower.max(upper) as f32 / (bytes.len().saturating_sub(1).max(1)) as f32
}

fn classify_algorithm_strings(dex: &DexFile<'_>) -> HashMap<u32, AlgorithmClass> {
    let mut map = HashMap::new();
    for idx in 0..dex.string_count() {
        let idx_u32 = idx as u32;
        let Some(value) = dex.string(StringIdx::new(idx_u32)) else {
            continue;
        };
        if let Some(class) = classify_algorithm(value) {
            map.insert(idx_u32, class);
        }
    }
    map
}

/// Classify a `Cipher`/`MessageDigest` algorithm or transformation string (`ALG`, `ALG/MODE` or
/// `ALG/MODE/PADDING`) by its exact tokens, so `description` is not read as DES; `None` for a
/// string that names no known algorithm.
fn classify_algorithm(value: &str) -> Option<AlgorithmClass> {
    let lower = value.to_ascii_lowercase();
    let tokens: Vec<&str> = lower.split('/').collect();
    if tokens.len() > 3 {
        return None;
    }
    let algorithm = tokens[0];
    if let Some((_, label)) = WEAK_ALGORITHMS.iter().find(|(name, _)| *name == algorithm) {
        return Some(AlgorithmClass::Weak(label));
    }
    if BLOCK_CIPHERS.contains(&algorithm) {
        return Some(match tokens.get(1) {
            None | Some(&"ecb") => AlgorithmClass::Weak("ECB"),
            Some(_) => AlgorithmClass::Strong,
        });
    }
    STRONG_ALGORITHMS
        .contains(&algorithm)
        .then_some(AlgorithmClass::Strong)
}

fn build_string_to_methods(xrefs: &graphs::Xrefs) -> HashMap<u32, Vec<u32>> {
    let mut map: HashMap<u32, Vec<u32>> = HashMap::new();
    for (method, string_idx) in &xrefs.method_strings {
        map.entry(*string_idx).or_default().push(*method);
    }
    map
}

fn invert_string_map(map: &HashMap<u32, Vec<u32>>) -> HashMap<u32, Vec<u32>> {
    let mut inverted: HashMap<u32, Vec<u32>> = HashMap::new();
    for (string_idx, methods) in map {
        for method in methods {
            inverted.entry(*method).or_default().push(*string_idx);
        }
    }
    inverted
}

fn shannon_entropy(value: &str) -> f32 {
    let mut counts = [0u32; 256];
    let bytes = value.as_bytes();
    if bytes.is_empty() {
        return 0.0;
    }
    for byte in bytes {
        counts[*byte as usize] += 1;
    }
    let len = bytes.len() as f32;
    let mut entropy = 0.0f32;
    for count in counts {
        if count == 0 {
            continue;
        }
        let probability = (count as f32) / len;
        entropy -= probability * probability.log2();
    }
    entropy
}

fn preview_literal(value: &str) -> String {
    let trimmed = value.trim();
    const MAX_LEN: usize = 64;
    if trimmed.len() <= MAX_LEN {
        trimmed.to_string()
    } else {
        // Cut on a character boundary: a multibyte character may straddle byte 64.
        format!("{}…", &trimmed[..trimmed.floor_char_boundary(MAX_LEN)])
    }
}

fn references_cipher_descriptor(dex: &DexFile<'_>) -> bool {
    dex.strings()
        .flatten()
        .any(|value| value.contains("javax/crypto/Cipher"))
}

enum SecretTrigger {
    Pattern(&'static str),
    Keyword,
    Entropy(f32),
}

impl SecretTrigger {
    fn label(&self) -> &'static str {
        match self {
            SecretTrigger::Pattern(label) => label,
            SecretTrigger::Keyword => "keyword",
            SecretTrigger::Entropy(_) => "entropy",
        }
    }

    fn severity(&self) -> Severity {
        match self {
            SecretTrigger::Pattern(_) => Severity::High,
            SecretTrigger::Keyword => Severity::Medium,
            SecretTrigger::Entropy(ent) if *ent > 4.5 => Severity::High,
            SecretTrigger::Entropy(_) => Severity::Medium,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AlgorithmClass, SecretTrigger, classify_algorithm, classify_secret};

    #[test]
    fn entropy_reports_tokens_not_text() {
        for token in [
            "t8rNQwaGaAoYFlByH6RRPkidNPdcJWUV",
            "dGhpcyBpcyBub3QgYSByZWFsIGtleQ1=",
            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
        ] {
            assert!(
                matches!(classify_secret(token, 3.5), Some(SecretTrigger::Entropy(_))),
                "{token}"
            );
        }
        for text in [
            "AES/GCM/NoPadding",
            "DES/ECB/PKCS5Padding",
            "CREATE TABLE notes (id INTEGER)",
            "SELECT COUNT(*) FROM notes",
            "https://example.com/",
            "InsecureFileMode.java",
            "TaskStackBuilderApi16Impl",
            "SlidingPanelLayoutImplJBMR1",
            "Base_V7_Theme_AppCompat_Dialog",
            "ACTION_MEDIA_BUTTON_V2_COMPAT",
            "Landroid/support/v4/util/ArrayMap;",
            "abcdefghijklmnopqrstuvwxyz0123",
            "t8rNQwaGaAoYFlByH6RRPkidNPdcJ===",
        ] {
            assert!(classify_secret(text, 3.5).is_none(), "{text}");
        }
    }

    #[test]
    fn algorithms_are_classified_by_exact_tokens() {
        for (value, expected) in [
            ("MD5", Some(AlgorithmClass::Weak("MD5"))),
            ("SHA-1", Some(AlgorithmClass::Weak("SHA1"))),
            ("SHA", Some(AlgorithmClass::Weak("SHA1"))),
            ("DES/ECB/PKCS5Padding", Some(AlgorithmClass::Weak("DES"))),
            (
                "DESede/CBC/PKCS5Padding",
                Some(AlgorithmClass::Weak("3DES")),
            ),
            ("RC4", Some(AlgorithmClass::Weak("RC4"))),
            ("AES/ECB/PKCS5Padding", Some(AlgorithmClass::Weak("ECB"))),
            ("AES", Some(AlgorithmClass::Weak("ECB"))),
            ("AES/GCM/NoPadding", Some(AlgorithmClass::Strong)),
            ("AES_256/GCM/NoPadding", Some(AlgorithmClass::Strong)),
            (
                "RSA/ECB/OAEPWithSHA-256AndMGF1Padding",
                Some(AlgorithmClass::Strong),
            ),
            ("SHA-256", Some(AlgorithmClass::Strong)),
            ("SHA-512/256", Some(AlgorithmClass::Strong)),
            ("ChaCha20-Poly1305", Some(AlgorithmClass::Strong)),
            ("description", None),
            ("Settings", None),
            ("https://example.com/ecb", None),
            ("des des", None),
        ] {
            assert_eq!(classify_algorithm(value), expected, "{value}");
        }
    }
}
