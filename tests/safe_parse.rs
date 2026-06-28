//! safe_parse: the three-valued fold. Value for a clean parse, Null for a
//! security violation, Undefined for malformed JSON.

mod common;

use common::*;
use secure_json_parse::{safe_parse, safe_parse_bytes, SafeOutcome};
use serde_json::json;

fn value(outcome: SafeOutcome) -> serde_json::Value {
    match outcome {
        SafeOutcome::Value(v) => v,
        other => panic!("expected a value, got {other:?}"),
    }
}

#[test]
fn parses_bytes() {
    assert_eq!(value(safe_parse_bytes(b"\"X\"")), json!("X"));
}

#[test]
fn parses_object() {
    assert_eq!(value(safe_parse(OBJ)), json!({"a": 5, "b": 6}));
}

#[test]
fn parses_primitives() {
    assert_eq!(value(safe_parse("0")), json!(0));
    assert_eq!(value(safe_parse("null")), json!(null));
    assert_eq!(value(safe_parse(r#""X""#)), json!("X"));
}

#[test]
fn nested_proto_returns_null() {
    assert!(matches!(safe_parse(PROTO_NESTED), SafeOutcome::Null));
}

#[test]
fn single_proto_returns_null() {
    assert!(matches!(safe_parse(PROTO), SafeOutcome::Null));
}

#[test]
fn mixed_returns_null() {
    assert!(matches!(safe_parse(MIXED_NESTED), SafeOutcome::Null));
}

#[test]
fn constructor_with_prototype_returns_null() {
    assert!(matches!(safe_parse(CTOR), SafeOutcome::Null));
}

#[test]
fn invalid_json_returns_undefined() {
    assert!(matches!(safe_parse(INVALID), SafeOutcome::Undefined));
}

#[test]
fn constructor_without_prototype_returns_value() {
    assert_eq!(
        value(safe_parse(CTOR_NO_PROTO)),
        json!({"a": 5, "b": 6, "constructor": {"bar": "baz"}})
    );
}
