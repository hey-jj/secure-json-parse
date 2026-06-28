//! The safe flag on parse and scan. A violation returns Ok(None). The check is
//! skipped under Ignore, so safe never fires there.

mod common;

use common::*;
use secure_json_parse::{parse, scan, Action, Error, Options};
use serde_json::json;

fn safe() -> Options {
    Options::default().safe(true)
}

#[test]
fn parse_safe_proto_returns_none() {
    assert_eq!(parse(PROTO, &safe()).unwrap(), None);
}

#[test]
fn parse_safe_constructor_returns_none() {
    assert_eq!(parse(CTOR, &safe()).unwrap(), None);
}

#[test]
fn parse_safe_valid_returns_value() {
    assert_eq!(parse(OBJ, &safe()).unwrap(), Some(json!({"a": 5, "b": 6})));
}

#[test]
fn parse_safe_wins_over_remove_proto() {
    let opts = safe().proto_action(Action::Remove);
    assert_eq!(parse(PROTO, &opts).unwrap(), None);
}

#[test]
fn parse_safe_wins_over_remove_constructor() {
    let opts = safe().constructor_action(Action::Remove);
    assert_eq!(parse(CTOR, &opts).unwrap(), None);
}

#[test]
fn parse_not_safe_proto_throws() {
    let opts = Options::default().safe(false);
    assert!(matches!(parse(PROTO, &opts), Err(Error::ForbiddenProperty)));
}

#[test]
fn parse_not_safe_constructor_throws() {
    let opts = Options::default().safe(false);
    assert!(matches!(parse(CTOR, &opts), Err(Error::ForbiddenProperty)));
}

#[test]
fn parse_safe_nested_proto_returns_none() {
    let text = r#"{ "a": 5, "c": { "d": 0, "__proto__": { "y": 8 } } }"#;
    assert_eq!(parse(text, &safe()).unwrap(), None);
}

#[test]
fn parse_safe_nested_constructor_returns_none() {
    let text = r#"{ "a": 5, "c": { "d": 0, "constructor": {"prototype": {"bar": "baz"}} } }"#;
    assert_eq!(parse(text, &safe()).unwrap(), None);
}

#[test]
fn parse_safe_ignore_proto_keeps_value() {
    let opts = safe().proto_action(Action::Ignore);
    let raw: serde_json::Value = serde_json::from_str(PROTO).unwrap();
    assert_eq!(parse(PROTO, &opts).unwrap(), Some(raw));
}

#[test]
fn parse_safe_ignore_constructor_keeps_value() {
    let opts = safe().constructor_action(Action::Ignore);
    let raw: serde_json::Value = serde_json::from_str(CTOR).unwrap();
    assert_eq!(parse(CTOR, &opts).unwrap(), Some(raw));
}

#[test]
fn parse_safe_with_both_proto_and_constructor() {
    // Both forbidden keys present. Safe mode still folds to None.
    assert_eq!(parse(MIXED, &safe()).unwrap(), None);
}

#[test]
fn scan_safe_proto_returns_none() {
    let obj: serde_json::Value = serde_json::from_str(PROTO).unwrap();
    assert_eq!(scan(obj, &safe()).unwrap(), None);
}

#[test]
fn scan_safe_constructor_returns_none() {
    let obj: serde_json::Value = serde_json::from_str(CTOR).unwrap();
    assert_eq!(scan(obj, &safe()).unwrap(), None);
}

#[test]
fn scan_safe_valid_returns_value() {
    let obj = json!({"a": 5, "b": 6});
    assert_eq!(scan(obj.clone(), &safe()).unwrap(), Some(obj));
}

#[test]
fn scan_not_safe_proto_throws() {
    let obj: serde_json::Value = serde_json::from_str(PROTO).unwrap();
    let opts = Options::default().safe(false);
    assert!(matches!(scan(obj, &opts), Err(Error::ForbiddenProperty)));
}

#[test]
fn scan_not_safe_constructor_throws() {
    let obj: serde_json::Value = serde_json::from_str(CTOR).unwrap();
    let opts = Options::default().safe(false);
    assert!(matches!(scan(obj, &opts), Err(Error::ForbiddenProperty)));
}
