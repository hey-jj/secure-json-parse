//! constructorAction: error (default), remove, ignore. A violation needs a
//! `constructor` whose value is an object holding a `prototype` key.

mod common;

use common::*;
use secure_json_parse::{parse, Action, Error, Options};
use serde_json::json;

#[test]
fn remove_strips_constructor() {
    let v = parse(CTOR, &ctor_remove()).unwrap().unwrap();
    assert_eq!(v, json!({"a": 5, "b": 6}));
}

#[test]
fn remove_keeps_constructor_without_prototype() {
    // No prototype child means no violation. The key stays.
    let v = parse(CTOR_NO_PROTO, &ctor_remove()).unwrap().unwrap();
    assert_eq!(v, json!({"a": 5, "b": 6, "constructor": {"bar": "baz"}}));
}

#[test]
fn remove_strips_nested_constructor() {
    let v = parse(CTOR_NESTED, &ctor_remove()).unwrap().unwrap();
    assert_eq!(
        v,
        json!({"a": 5, "b": 6, "c": {"d": 0, "e": "text", "f": {"g": 2}}})
    );
}

#[test]
fn ignore_keeps_constructor() {
    let v = parse(CTOR, &ctor_ignore()).unwrap().unwrap();
    let raw: serde_json::Value = serde_json::from_str(CTOR).unwrap();
    assert_eq!(v, raw);
}

#[test]
fn constructor_as_value_does_not_trip() {
    let v = parse(CTOR_VALUE, &Options::default()).unwrap().unwrap();
    assert_eq!(v, json!({"a": 5, "b": "constructor"}));
}

#[test]
fn error_on_constructor_default() {
    assert!(matches!(
        parse(CTOR, &Options::default()),
        Err(Error::ForbiddenProperty)
    ));
}

#[test]
fn error_on_constructor_explicit() {
    let opts = Options::default().constructor_action(Action::Error);
    assert!(matches!(parse(CTOR, &opts), Err(Error::ForbiddenProperty)));
}

#[test]
fn no_throw_when_constructor_lacks_prototype() {
    // Default actions, but constructor has no prototype child.
    let v = parse(CTOR_NO_PROTO, &Options::default()).unwrap().unwrap();
    assert_eq!(v, json!({"a": 5, "b": 6, "constructor": {"bar": "baz"}}));
}

#[test]
fn error_on_constructor_whitespace_variants() {
    let variants = [
        r#"{ "a": 5, "b": 6, "constructor": {"prototype":{"bar":"baz"}} }"#,
        "{ \"a\": 5, \"b\": 6, \"constructor\" : {\"prototype\":{\"bar\":\"baz\"}} }",
        "{ \"a\": 5, \"b\": 6, \"constructor\" \n\r\t : {\"prototype\":{\"bar\":\"baz\"}} }",
        "{ \"a\": 5, \"b\": 6, \"constructor\" \n \r \t : {\"prototype\":{\"bar\":\"baz\"}} }",
    ];
    for v in variants {
        assert!(
            matches!(parse(v, &Options::default()), Err(Error::ForbiddenProperty)),
            "expected error for {v:?}"
        );
    }
}

#[test]
fn constructor_null_is_safe_under_all_actions() {
    // A null value for constructor is never a violation.
    for opts in [ctor_remove(), Options::default(), ctor_ignore()] {
        let v = parse(CTOR_NULL, &opts).unwrap().unwrap();
        assert_eq!(v, json!({"constructor": null}));
    }
}

#[test]
fn error_on_constructor_unicode_variants() {
    // Each input escapes some or all of constructor with \uXXXX.
    let variants = [
        r#"{ "a": 5, "b": 6, "\u0063\u006fnstructor": {"prototype":{"bar":"baz"}} }"#,
        r#"{ "a": 5, "b": 6, "\u0063\u006f\u006e\u0073\u0074ructor": {"prototype":{"bar":"baz"}} }"#,
        r#"{ "a": 5, "b": 6, "\u0063\u006f\u006e\u0073\u0074\u0072\u0075\u0063\u0074\u006f\u0072": {"prototype":{"bar":"baz"}} }"#,
        r#"{ "a": 5, "b": 6, "\u0063\u006Fnstructor": {"prototype":{"bar":"baz"}} }"#,
        r#"{ "a": 5, "b": 6, "\u0063\u006F\u006E\u0073\u0074\u0072\u0075\u0063\u0074\u006F\u0072": {"prototype":{"bar":"baz"}} }"#,
    ];
    for v in variants {
        assert!(
            matches!(parse(v, &Options::default()), Err(Error::ForbiddenProperty)),
            "expected error for {v:?}"
        );
    }
}
