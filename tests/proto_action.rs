//! protoAction: error (default), remove, ignore. Plus whitespace and unicode
//! key variants that decode to the literal `__proto__`.

mod common;

use common::*;
use secure_json_parse::{parse, Action, Error, Options};
use serde_json::json;

#[test]
fn remove_strips_proto() {
    let v = parse(PROTO, &proto_remove()).unwrap().unwrap();
    assert_eq!(v, json!({"a": 5, "b": 6}));
}

#[test]
fn remove_strips_nested_proto() {
    let v = parse(PROTO_NESTED, &proto_remove()).unwrap().unwrap();
    assert_eq!(
        v,
        json!({"a": 5, "b": 6, "c": {"d": 0, "e": "text", "f": {"g": 2}}})
    );
}

#[test]
fn ignore_keeps_proto() {
    // Ignore is identical to a plain parse: the __proto__ key stays.
    let v = parse(PROTO, &proto_ignore()).unwrap().unwrap();
    let raw: serde_json::Value = serde_json::from_str(PROTO).unwrap();
    assert_eq!(v, raw);
}

#[test]
fn proto_as_value_does_not_trip() {
    let v = parse(PROTO_VALUE, &Options::default()).unwrap().unwrap();
    assert_eq!(v, json!({"a": 5, "b": "__proto__"}));
}

#[test]
fn error_on_proto_default() {
    assert!(matches!(
        parse(PROTO, &Options::default()),
        Err(Error::ForbiddenProperty)
    ));
}

#[test]
fn error_on_proto_explicit() {
    let opts = Options::default().proto_action(Action::Error);
    assert!(matches!(parse(PROTO, &opts), Err(Error::ForbiddenProperty)));
}

#[test]
fn error_on_proto_whitespace_variants() {
    // Whitespace between the key and colon does not change detection.
    let variants = [
        r#"{ "a": 5, "b": 6, "__proto__": { "x": 7 } }"#,
        "{ \"a\": 5, \"b\": 6, \"__proto__\" : { \"x\": 7 } }",
        "{ \"a\": 5, \"b\": 6, \"__proto__\" \n\r\t : { \"x\": 7 } }",
        "{ \"a\": 5, \"b\": 6, \"__proto__\" \n \r \t : { \"x\": 7 } }",
    ];
    for v in variants {
        assert!(
            matches!(parse(v, &Options::default()), Err(Error::ForbiddenProperty)),
            "expected error for {v:?}"
        );
    }
}

#[test]
fn error_on_proto_unicode_variants() {
    // Each input escapes some or all of __proto__ with \uXXXX. serde_json
    // decodes them to the literal key, which the walk then detects.
    let variants = [
        r#"{ "a": 5, "b": 6, "\u005f_proto__": { "x": 7 } }"#,
        r#"{ "a": 5, "b": 6, "_\u005fp\u0072oto__": { "x": 7 } }"#,
        r#"{ "a": 5, "b": 6, "\u005f\u005f\u0070\u0072\u006f\u0074\u006f\u005f\u005f": { "x": 7 } }"#,
        r#"{ "a": 5, "b": 6, "\u005F_proto__": { "x": 7 } }"#,
        r#"{ "a": 5, "b": 6, "_\u005Fp\u0072oto__": { "x": 7 } }"#,
        r#"{ "a": 5, "b": 6, "\u005F\u005F\u0070\u0072\u006F\u0074\u006F\u005F\u005F": { "x": 7 } }"#,
    ];
    for v in variants {
        assert!(
            matches!(parse(v, &Options::default()), Err(Error::ForbiddenProperty)),
            "expected error for {v:?}"
        );
    }
}

#[test]
fn nested_proto_errors_by_default() {
    assert!(matches!(
        parse(PROTO_NESTED, &Options::default()),
        Err(Error::ForbiddenProperty)
    ));
}

#[test]
fn first_and_last_key_position_both_detected() {
    let first = r#"{"__proto__": {"x": 1}, "a": 2}"#;
    let last = r#"{"a": 2, "__proto__": {"x": 1}}"#;
    for v in [first, last] {
        assert!(matches!(
            parse(v, &Options::default()),
            Err(Error::ForbiddenProperty)
        ));
    }
}

#[test]
fn non_object_proto_value_still_errors() {
    // Detection keys on the property name, not the value shape. A scalar, null,
    // or array value for __proto__ is a violation just like an object value.
    let inputs = [
        r#"{"__proto__": 5}"#,
        r#"{"__proto__": "x"}"#,
        r#"{"__proto__": true}"#,
        r#"{"__proto__": null}"#,
        r#"{"__proto__": [1, 2]}"#,
    ];
    for text in inputs {
        assert!(
            matches!(
                parse(text, &Options::default()),
                Err(Error::ForbiddenProperty)
            ),
            "expected error for {text:?}"
        );
    }
}

#[test]
fn non_object_proto_value_is_removed() {
    // Remove strips the __proto__ key whatever its value, keeping siblings.
    let cases = [
        (r#"{"a": 1, "__proto__": 5}"#, json!({"a": 1})),
        (r#"{"a": 1, "__proto__": "x"}"#, json!({"a": 1})),
        (r#"{"a": 1, "__proto__": null}"#, json!({"a": 1})),
        (r#"{"a": 1, "__proto__": [1, 2]}"#, json!({"a": 1})),
        (r#"{"a": 1, "__proto__": true}"#, json!({"a": 1})),
    ];
    for (text, expected) in cases {
        let v = parse(text, &proto_remove()).unwrap().unwrap();
        assert_eq!(v, expected, "for {text:?}");
    }
}
