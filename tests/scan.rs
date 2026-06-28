//! scan operates on an already-parsed value. It removes, errors, or detects
//! without any JSON parsing.

mod common;

use common::*;
use secure_json_parse::{scan, Error, Options};
use serde_json::json;

#[test]
fn clean_object_with_default_options_is_ok() {
    let obj = json!({"a": "b"});
    assert_eq!(scan(obj.clone(), &Options::default()).unwrap(), Some(obj));
}

#[test]
fn remove_mutates_nested_value() {
    let obj: serde_json::Value = serde_json::from_str(PROTO_NESTED).unwrap();
    let cleaned = scan(obj, &proto_remove()).unwrap().unwrap();
    assert_eq!(
        cleaned,
        json!({"a": 5, "b": 6, "c": {"d": 0, "e": "text", "f": {"g": 2}}})
    );
}

#[test]
fn remove_matches_parse_remove() {
    // scan with remove on a pre-built value equals parse with remove on text.
    let obj: serde_json::Value = serde_json::from_str(PROTO_NESTED).unwrap();
    let via_scan = scan(obj, &proto_remove()).unwrap().unwrap();
    let via_parse = secure_json_parse::parse(PROTO_NESTED, &proto_remove())
        .unwrap()
        .unwrap();
    assert_eq!(via_scan, via_parse);
}

#[test]
fn error_action_throws() {
    let obj: serde_json::Value = serde_json::from_str(PROTO).unwrap();
    assert!(matches!(
        scan(obj, &Options::default()),
        Err(Error::ForbiddenProperty)
    ));
}

#[test]
fn constructor_remove_in_array() {
    // filter walks array elements, so a constructor inside a list is cleaned.
    let obj = json!({"list": [{"constructor": {"prototype": {}}}]});
    let cleaned = scan(obj, &ctor_remove()).unwrap().unwrap();
    assert_eq!(cleaned, json!({"list": [{}]}));
}
