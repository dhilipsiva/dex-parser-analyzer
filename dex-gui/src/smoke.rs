//! Browser smoke tests (`just test-wasm`, inside `nix develop .#wasm-test`): the app mounts, and the
//! analyzer, compiled to wasm32, reports and renders the rule fixture's findings.

use std::sync::Arc;

use dexpa_analysis::{config::AnalysisConfig, engine::analyze_dex_bytes, model::Severity};
use leptos::{mount::mount_to, prelude::*, wasm_bindgen::JsCast};
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

use crate::ui::AnalysisView;

wasm_bindgen_test_configure!(run_in_browser);

const RULES_POSITIVE: &[u8] = include_bytes!("../../dex-analysis/tests/data/rules-positive.dex");

#[wasm_bindgen_test]
fn the_app_mounts() {
    crate::mount();
    let heading = document()
        .query_selector("h1")
        .expect("valid selector")
        .expect("the app rendered a heading");
    assert_eq!(
        heading.text_content().as_deref(),
        Some("Dex Parser Analyzer")
    );
    assert!(
        document()
            .query_selector("input[type=file]")
            .expect("valid selector")
            .is_some(),
        "the file picker is missing"
    );
}

#[wasm_bindgen_test]
fn the_analyzer_runs_in_wasm_and_its_findings_render() {
    let report = analyze_dex_bytes(RULES_POSITIVE, &AnalysisConfig::default()).expect("analyses");
    assert_eq!(report.findings[0].severity, Severity::Critical);

    let container = document().create_element("div").expect("create a div");
    document()
        .body()
        .expect("a body")
        .append_child(&container)
        .expect("append");
    let container: leptos::web_sys::HtmlElement = container.unchecked_into();
    let report = Arc::new(report);
    mount_to(
        container.clone(),
        move || view! { <AnalysisView report=report.clone()/> },
    )
    .forget();
    let text = container.text_content().unwrap_or_default();
    assert!(
        text.contains("M7_WEBVIEW_JS_INTERFACE"),
        "findings table: {text}"
    );
}
