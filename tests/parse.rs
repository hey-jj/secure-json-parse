//! Core parse behavior: primitives, objects, bytes, and BOM.

mod common;

use common::*;
use secure_json_parse::{parse, parse_bytes, Error, Options};
use serde_json::json;

#[test]
fn parses_object_string() {
    let v = parse(OBJ, &Options::default()).unwrap().unwrap();
    assert_eq!(v, json!({"a": 5, "b": 6}));
}

#[test]
fn parses_null_string() {
    let v = parse("null", &Options::default()).unwrap().unwrap();
    assert_eq!(v, json!(null));
}

#[test]
fn parses_zero_string() {
    let v = parse("0", &Options::default()).unwrap().unwrap();
    assert_eq!(v, json!(0));
}

#[test]
fn parses_string_string() {
    let v = parse(r#""X""#, &Options::default()).unwrap().unwrap();
    assert_eq!(v, json!("X"));
}

#[test]
fn parses_bytes() {
    let v = parse_bytes(b"\"X\"", &Options::default()).unwrap().unwrap();
    assert_eq!(v, json!("X"));
}

#[test]
fn parses_array() {
    let v = parse(r#"[1, 2, 3]"#, &Options::default()).unwrap().unwrap();
    assert_eq!(v, json!([1, 2, 3]));
}

#[test]
fn malformed_json_is_syntax_error() {
    let err = parse(INVALID, &Options::default());
    assert!(matches!(err, Err(Error::Syntax(_))));
}

#[test]
fn empty_input_is_syntax_error() {
    assert!(matches!(
        parse("", &Options::default()),
        Err(Error::Syntax(_))
    ));
}

#[test]
fn empty_object_is_identity() {
    let v = parse("{}", &Options::default()).unwrap().unwrap();
    assert_eq!(v, json!({}));
}

#[test]
fn empty_array_is_identity() {
    let v = parse("[]", &Options::default()).unwrap().unwrap();
    assert_eq!(v, json!([]));
}

#[test]
fn hasownproperty_data_key_survives_removal() {
    // A data key named hasOwnProperty must be kept while __proto__ is removed.
    let v = parse(HASOWN, &proto_remove()).unwrap().unwrap();
    assert_eq!(v, json!({"a": 5, "b": 6, "hasOwnProperty": "text"}));
}
