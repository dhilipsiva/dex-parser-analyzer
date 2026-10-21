use dex_core::{DexError, format, parse_dex};

#[test]
fn invalid_magic_rejected() {
    let bytes = vec![0u8; format::HEADER_SIZE];
    match parse_dex(&bytes) {
        Err(DexError::InvalidMagic { .. }) => {}
        Ok(_) => panic!("expected invalid magic"),
        Err(err) => panic!("unexpected error: {err:?}"),
    }
}

#[test]
fn out_of_bounds_section_detected() {
    let mut bytes = std::fs::read("tests/data/Test.dex").expect("fixture missing");
    let len = bytes.len() as u32;
    let string_off = len + 0x100;
    bytes[0x3C..0x40].copy_from_slice(&string_off.to_le_bytes());
    match parse_dex(&bytes) {
        Err(DexError::SectionOutOfBounds {
            section: "string_ids",
            ..
        }) => {}
        Ok(_) => panic!("expected section bounds error"),
        Err(err) => panic!("unexpected error: {err:?}"),
    }
}

/// `Test.dex` with one header-relative u32 field overwritten, the way a crafted upload would.
fn test_dex_with_u32_at(table_ptr: usize, delta: usize, value: u32) -> Vec<u8> {
    let mut bytes = std::fs::read("tests/data/Test.dex").expect("read Test.dex");
    let table = u32::from_le_bytes(bytes[table_ptr..table_ptr + 4].try_into().unwrap()) as usize;
    bytes[table + delta..table + delta + 4].copy_from_slice(&value.to_le_bytes());
    bytes
}

#[test]
fn class_data_off_out_of_range_returns_an_error() {
    // class_defs_off is at 0x64; class_data_off is the 25th byte of the first class_def_item.
    let bytes = test_dex_with_u32_at(0x64, 24, 0xFFFF_FFF0);
    assert!(
        parse_dex(&bytes).is_err(),
        "an out-of-range class_data_off must be an error"
    );
}

#[test]
fn string_data_off_out_of_range_does_not_panic() {
    // string_ids_off is at 0x3C; its first entry is the offset of the first string_data_item.
    // The parser rejects it up front; were it ever accepted, iteration must report it, not panic.
    let bytes = test_dex_with_u32_at(0x3C, 0, 0xFFFF_FFF0);
    assert!(matches!(
        parse_dex(&bytes),
        Err(DexError::SectionOutOfBounds {
            section: "string_data_item",
            ..
        })
    ));
    if let Ok(dex) = parse_dex(&bytes) {
        assert_eq!(dex.strings().count(), dex.string_count());
        assert!(
            dex.strings().any(|s| s.is_err()),
            "the bad entry must surface as an error"
        );
    }
}
