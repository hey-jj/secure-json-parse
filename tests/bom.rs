//! Byte order mark stripping. The UTF-8 BOM (EF BB BF) is removed before parse
//! for both text and byte input, on parse and safe_parse.

mod common;

use secure_json_parse::{parse, parse_bytes, safe_parse, safe_parse_bytes, Options, SafeOutcome};
use serde_json::json;

/// `{"hello":"world"}` prefixed with the UTF-8 BOM bytes.
fn bom_bytes() -> Vec<u8> {
    let mut v = vec![0xEF, 0xBB, 0xBF];
    v.extend_from_slice(br#"{"hello":"world"}"#);
    v
}

fn safe_value(outcome: SafeOutcome) -> serde_json::Value {
    match outcome {
        SafeOutcome::Value(v) => v,
        other => panic!("expected a value, got {other:?}"),
    }
}

#[test]
fn parse_string_with_bom() {
    let text = String::from_utf8(bom_bytes()).unwrap();
    let v = parse(&text, &Options::default()).unwrap().unwrap();
    assert_eq!(v, json!({"hello": "world"}));
}

#[test]
fn parse_bytes_with_bom() {
    let v = parse_bytes(&bom_bytes(), &Options::default())
        .unwrap()
        .unwrap();
    assert_eq!(v, json!({"hello": "world"}));
}

#[test]
fn safe_parse_string_with_bom() {
    let text = String::from_utf8(bom_bytes()).unwrap();
    assert_eq!(safe_value(safe_parse(&text)), json!({"hello": "world"}));
}

#[test]
fn safe_parse_bytes_with_bom() {
    assert_eq!(
        safe_value(safe_parse_bytes(&bom_bytes())),
        json!({"hello": "world"})
    );
}
