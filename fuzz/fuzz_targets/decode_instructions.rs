//! Decode every method's bytecode and build the program graphs from arbitrary bytes.
#![no_main]

use dex_core::format::MethodIdx;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(dex) = dex_core::parse_dex(data) else {
        return;
    };
    for idx in 0..dex.method_count() {
        let method = MethodIdx::new(idx as u32);
        let _ = dex.decode_instructions(method);
        let _ = dex_core::graphs::build_method_cfg(&dex, method);
    }
    let _ = dex_core::graphs::build_call_graph(&dex);
    let _ = dex_core::graphs::build_xrefs(&dex);
});
