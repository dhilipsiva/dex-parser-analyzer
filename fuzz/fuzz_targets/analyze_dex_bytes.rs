//! The analyzer's whole pipeline on arbitrary bytes, as the GUI runs it on an upload.
#![no_main]

use dex_analysis::{config::AnalysisConfig, engine::analyze_dex_bytes};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = analyze_dex_bytes(data, &AnalysisConfig::default());
});
