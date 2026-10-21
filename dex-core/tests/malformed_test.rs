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
        matches!(
            parse_dex(&bytes),
            Err(DexError::SectionOutOfBounds {
                section: "class_data",
                ..
            })
        ),
        "an out-of-range class_data_off must be an error"
    );
}

#[test]
fn string_data_off_out_of_range_does_not_panic() {
    // string_ids_off is at 0x3C; its first entry is the offset of the first string_data_item.
    // The parser rejects it up front.
    let bytes = test_dex_with_u32_at(0x3C, 0, 0xFFFF_FFF0);
    assert!(matches!(
        parse_dex(&bytes),
        Err(DexError::SectionOutOfBounds {
            section: "string_data_item",
            ..
        })
    ));
}

#[test]
fn string_data_at_the_last_byte_is_an_error_not_a_panic() {
    // An in-bounds offset whose data is cut off by the end of the file: the parse succeeds, and
    // the bad entry must come back from the iterator as an error.
    let mut bytes = std::fs::read("tests/data/Test.dex").expect("read Test.dex");
    let len = bytes.len();
    let string_ids = u32::from_le_bytes(bytes[0x3C..0x40].try_into().unwrap()) as usize;
    bytes[string_ids..string_ids + 4].copy_from_slice(&((len - 1) as u32).to_le_bytes());
    bytes[len - 1] = 0x05;
    let dex = parse_dex(&bytes).expect("the header and tables are still valid");
    assert_eq!(dex.strings().count(), dex.string_count());
    assert!(
        dex.strings().any(|s| s.is_err()),
        "the truncated entry must surface as an error"
    );
}

fn read_uleb(bytes: &[u8], at: usize) -> (u32, usize) {
    let (mut value, mut shift, mut used) = (0u32, 0, 0);
    loop {
        let byte = bytes[at + used];
        value |= u32::from(byte & 0x7f) << shift;
        used += 1;
        if byte & 0x80 == 0 {
            return (value, used);
        }
        shift += 7;
    }
}

#[test]
fn method_idx_out_of_range_returns_an_error() {
    // The first direct method's method_idx_diff points past the 3-entry method_ids table.
    let mut bytes = std::fs::read("tests/data/Test.dex").expect("read Test.dex");
    let class_defs = u32::from_le_bytes(bytes[0x64..0x68].try_into().unwrap()) as usize;
    let mut cursor =
        u32::from_le_bytes(bytes[class_defs + 24..class_defs + 28].try_into().unwrap()) as usize;
    let mut sizes = [0u32; 4];
    for size in &mut sizes {
        let (value, used) = read_uleb(&bytes, cursor);
        *size = value;
        cursor += used;
    }
    for _ in 0..(sizes[0] + sizes[1]) * 2 {
        cursor += read_uleb(&bytes, cursor).1;
    }
    assert!(
        sizes[2] + sizes[3] > 0,
        "the fixture's first class has methods"
    );
    assert_eq!(read_uleb(&bytes, cursor).1, 1, "a one-byte method_idx_diff");
    bytes[cursor] = 0x7f;
    assert!(matches!(
        parse_dex(&bytes),
        Err(DexError::InvalidIndex {
            table: "method_ids",
            ..
        })
    ));
}
