//! Crafted inputs that used to crash or exhaust the parser. Each test builds its input from a
//! fixture in memory; the parse must return (an error, where the input is invalid) and never panic,
//! overflow the stack or try to allocate memory the input cannot justify.

use dexpa_core::{DexError, format::MethodIdx, parse_dex};

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(format!("tests/data/{name}")).expect("read fixture")
}

fn put_u32(bytes: &mut [u8], at: usize, value: u32) {
    bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

fn get_u32(bytes: &[u8], at: usize) -> usize {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) as usize
}

fn uleb_len(bytes: &[u8], at: usize) -> usize {
    bytes[at..]
        .iter()
        .position(|b| b & 0x80 == 0)
        .expect("terminated uleb")
        + 1
}

#[test]
fn overlong_sleb128_handler_size_is_an_error() {
    // The size of a method's first encoded_catch_handler becomes a six-byte sleb128.
    let (original, at) = [
        "ExceptionHandling.dex",
        "AnalysisTest.dex",
        "classes.dex",
        "Annotation_classes.dex",
    ]
    .iter()
    .find_map(|name| {
        let original = fixture(name);
        let dex = parse_dex(&original).expect("fixture parses");
        let base = original.as_ptr() as usize;
        let at = (0..dex.method_count())
            .filter_map(|m| dex.code_item(MethodIdx::new(m as u32)))
            .find(|code| code.tries_size > 0)
            .map(|code| {
                let insns = code.insns.as_ptr() as usize - base;
                let pad = if code.insns_size.is_multiple_of(2) {
                    0
                } else {
                    2
                };
                let list =
                    insns + code.insns_size as usize * 2 + pad + code.tries_size as usize * 8;
                list + uleb_len(&original, list)
            })?;
        drop(dex);
        Some((original, at))
    })
    .expect("a fixture with try blocks");
    let mut bytes = original.clone();
    bytes[at..at + 6].copy_from_slice(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x00]);
    assert!(matches!(parse_dex(&bytes), Err(DexError::Leb128 { .. })));
}

#[test]
fn data_size_overflow_does_not_panic() {
    let mut bytes = fixture("Test.dex");
    put_u32(&mut bytes, 0x68, 0xFFFF_FFFF); // data_size
    put_u32(&mut bytes, 0x6C, 0x10); // data_off
    let _ = parse_dex(&bytes);
}

#[test]
fn huge_map_list_count_is_an_error_not_an_allocation() {
    let mut bytes = fixture("Test.dex");
    let map_off = get_u32(&bytes, 0x34);
    put_u32(&mut bytes, map_off, 0xFFFF_FFFF);
    assert!(parse_dex(&bytes).is_err());
}

#[test]
fn huge_static_fields_count_is_an_error_not_an_allocation() {
    // A class's class_data_off moves to a fresh class_data_item whose static_fields_size is
    // 0xFFFFFFFF, with nothing after it.
    let mut bytes = fixture("Test.dex");
    let class_defs = get_u32(&bytes, 0x64);
    let class_count = get_u32(&bytes, 0x60);
    let entry = (0..class_count)
        .map(|i| class_defs + i * 32 + 24)
        .find(|&p| get_u32(&bytes, p) != 0)
        .expect("a class with class data");
    let fresh = bytes.len();
    bytes.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0x0F, 0, 0, 0]);
    put_u32(&mut bytes, entry, fresh as u32);
    assert!(parse_dex(&bytes).is_err());
}

#[test]
fn deeply_nested_encoded_values_are_an_error_not_a_stack_overflow() {
    // A map_list with one annotation_item whose element value is 500,000 nested arrays.
    let mut bytes = fixture("Test.dex");
    while !bytes.len().is_multiple_of(4) {
        bytes.push(0);
    }
    let map_off = bytes.len();
    let item_off = map_off + 4 + 12;
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&0x2004u16.to_le_bytes()); // TYPE_ANNOTATION_ITEM
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&(item_off as u32).to_le_bytes());
    bytes.extend_from_slice(&[0x00, 0x00, 0x01, 0x00]); // visibility, type_idx, size 1, name_idx
    for _ in 0..500_000 {
        bytes.extend_from_slice(&[0x1C, 0x01]); // VALUE_ARRAY of one element
    }
    bytes.push(0x1E); // VALUE_NULL
    put_u32(&mut bytes, 0x34, map_off as u32);
    assert!(parse_dex(&bytes).is_err());
}
