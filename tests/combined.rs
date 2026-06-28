//! The protoAction x constructorAction truth table. The input holds both a
//! `constructor.prototype` nesting and a `__proto__` key.

mod common;

use common::*;
use secure_json_parse::{parse, Action, Error, Options};
use serde_json::json;

fn opts(proto: Action, ctor: Action) -> Options {
    Options::default()
        .proto_action(proto)
        .constructor_action(ctor)
}

#[test]
fn remove_remove_strips_both() {
    let v = parse(MIXED, &opts(Action::Remove, Action::Remove))
        .unwrap()
        .unwrap();
    assert_eq!(v, json!({"a": 5, "b": 6}));
}

#[test]
fn ignore_remove_keeps_proto() {
    let v = parse(MIXED, &opts(Action::Ignore, Action::Remove))
        .unwrap()
        .unwrap();
    let expected: serde_json::Value =
        serde_json::from_str(r#"{ "a": 5, "b": 6, "__proto__": { "x": 7 } }"#).unwrap();
    assert_eq!(v, expected);
}

#[test]
fn remove_ignore_keeps_constructor() {
    let v = parse(MIXED, &opts(Action::Remove, Action::Ignore))
        .unwrap()
        .unwrap();
    let expected: serde_json::Value =
        serde_json::from_str(r#"{ "a": 5, "b": 6, "constructor":{"prototype":{"bar":"baz"}} }"#)
            .unwrap();
    assert_eq!(v, expected);
}

#[test]
fn ignore_ignore_keeps_both() {
    let v = parse(MIXED, &opts(Action::Ignore, Action::Ignore))
        .unwrap()
        .unwrap();
    let raw: serde_json::Value = serde_json::from_str(MIXED).unwrap();
    assert_eq!(v, raw);
}

#[test]
fn error_ignore_throws() {
    assert!(matches!(
        parse(MIXED, &opts(Action::Error, Action::Ignore)),
        Err(Error::ForbiddenProperty)
    ));
}

#[test]
fn ignore_error_throws() {
    assert!(matches!(
        parse(MIXED, &opts(Action::Ignore, Action::Error)),
        Err(Error::ForbiddenProperty)
    ));
}

#[test]
fn error_error_throws() {
    assert!(matches!(
        parse(MIXED, &opts(Action::Error, Action::Error)),
        Err(Error::ForbiddenProperty)
    ));
}

#[test]
fn mixed_nested_constructor_then_proto_errors() {
    // A plain constructor key plus a nested __proto__ errors by default.
    assert!(matches!(
        parse(MIXED_NESTED, &Options::default()),
        Err(Error::ForbiddenProperty)
    ));
}
