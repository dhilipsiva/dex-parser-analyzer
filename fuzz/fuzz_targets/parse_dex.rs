//! Parse arbitrary bytes, then touch everything the parse exposes lazily: strings and the overview.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(dex) = dex_core::parse_dex(data) {
        for string in dex.strings() {
            let _ = string;
        }
        let _ = dex_core::dto::dex_to_overview(&dex);
    }
});
