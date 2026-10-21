//! Drive the built `dexpa` binary: exact exit codes and the documented output shapes.

use std::process::{Command, Output};

fn dexpa(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_dexpa"))
        .args(args)
        .output()
        .expect("run dexpa")
}

fn fixture(name: &str) -> String {
    format!(
        "{}/../dex-core/tests/data/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("utf-8 stdout")
}

#[test]
fn help_lists_every_option() {
    let output = dexpa(&["--help"]);
    assert_eq!(output.status.code(), Some(0));
    let help = stdout(&output);
    for option in ["--format", "--max-findings", "--fail-on", "--version"] {
        assert!(help.contains(option), "--help lacks {option}");
    }
}

#[test]
fn version_names_the_binary() {
    let output = dexpa(&["--version"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stdout(&output).trim(),
        format!("dexpa {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn text_report_has_the_summary_and_findings() {
    let output = dexpa(&[&fixture("classes.dex"), "--max-findings", "3"]);
    assert_eq!(output.status.code(), Some(0));
    let text = stdout(&output);
    assert!(text.contains("Dex summary for"));
    assert!(text.contains("Classes:        340"));
    assert!(text.contains("Analysis findings (3 total):"));
}

#[test]
fn json_report_has_the_documented_shape() {
    let output = dexpa(&[
        &fixture("classes.dex"),
        "--format",
        "json",
        "--max-findings",
        "3",
    ]);
    assert_eq!(output.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).expect("valid JSON");
    assert_eq!(json["schema_version"], 1);
    let dex = &json["dex_files"][0];
    assert_eq!(dex["index"], 0);
    assert_eq!(dex["summary"]["class_count"], 340);
    assert!(dex["top_classes"].as_array().unwrap().len() <= 10);
    let findings = dex["report"]["findings"].as_array().unwrap();
    assert_eq!(findings.len(), 3);
    for key in ["id", "severity", "location", "message"] {
        assert!(findings[0].get(key).is_some(), "a finding lacks {key}");
    }
}

#[test]
fn apk_input_reports_every_dex() {
    let output = dexpa(&[&fixture("multidex.apk"), "--format", "json"]);
    assert_eq!(output.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).expect("valid JSON");
    let files = json["dex_files"].as_array().unwrap();
    assert_eq!(files.len(), 2);
    assert_eq!(files[1]["index"], 1);
}

#[test]
fn fail_on_exits_2_when_a_finding_meets_the_threshold() {
    // classes.dex reports Medium hardcoded-secret findings.
    let output = dexpa(&[&fixture("classes.dex"), "--fail-on", "medium"]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn fail_on_exits_0_when_no_finding_meets_the_threshold() {
    let output = dexpa(&[&fixture("Test.dex"), "--format", "json"]);
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).expect("valid JSON");
    let has_critical = json["dex_files"][0]["report"]["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["severity"] == "Critical");
    assert!(!has_critical, "the fixture has no Critical finding");
    let output = dexpa(&[&fixture("Test.dex"), "--fail-on", "critical"]);
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn unreadable_input_exits_1_with_a_message() {
    let output = dexpa(&["/nonexistent/classes.dex"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("failed to read"));
}

#[test]
fn non_dex_input_exits_1_with_a_message() {
    let output = dexpa(&[concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("failed to parse DEX file 0"));
}
