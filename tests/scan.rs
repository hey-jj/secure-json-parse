//! scan operates on an already-parsed value. It removes, errors, or detects
//! without any JSON parsing.

mod common;

use common::*;
use secure_json_parse::{scan, Error, Options};
use serde_json::{json, Map, Value};

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
    // scan walks array elements, so a constructor inside a list is cleaned.
    let obj = json!({"list": [{"constructor": {"prototype": {}}}]});
    let cleaned = scan(obj, &ctor_remove()).unwrap().unwrap();
    assert_eq!(cleaned, json!({"list": [{}]}));
}

#[test]
fn removed_deep_subtrees_do_not_overflow_the_stack() {
    fn nested_value(depth: usize) -> Value {
        let mut value = Value::Bool(true);
        for _ in 0..depth {
            let mut outer = Map::new();
            outer.insert("next".into(), value);
            value = Value::Object(outer);
        }
        value
    }

    let mut root = Map::new();
    root.insert("__proto__".into(), nested_value(100_000));
    root.insert("keep".into(), Value::Bool(true));

    let cleaned = scan(Value::Object(root), &proto_remove()).unwrap().unwrap();
    assert_eq!(cleaned, json!({"keep": true}));

    let mut constructor = Map::new();
    constructor.insert("prototype".into(), nested_value(100_000));

    let mut root = Map::new();
    root.insert("constructor".into(), Value::Object(constructor));
    root.insert("keep".into(), Value::Bool(true));

    let cleaned = scan(Value::Object(root), &ctor_remove()).unwrap().unwrap();
    assert_eq!(cleaned, json!({"keep": true}));
}

#[test]
fn deeply_nested_value_does_not_overflow_the_stack() {
    // scan takes a caller-built Value, which can be far deeper than the parser
    // would ever accept. The walk uses a worklist, not recursion, so a 100k-deep
    // tree returns instead of aborting the process with a stack overflow.
    use serde_json::{Map, Value};
    let mut value = Value::Object(Map::new());
    for _ in 0..100_000 {
        let mut outer = Map::new();
        outer.insert("c".into(), value);
        value = Value::Object(outer);
    }

    let result = scan(value, &ctor_remove());
    assert!(result.is_ok());

    // serde_json::Value drops recursively, so a 100k-deep tree would overflow
    // the stack on teardown and mask the walk's own stack-safety. Take the tree
    // apart top down so the scan, not the drop, is what this test exercises.
    let mut value = result.unwrap().unwrap();
    while let Value::Object(mut map) = value {
        match map.remove("c") {
            Some(child) => value = child,
            None => break,
        }
    }
}

#[test]
fn deeply_nested_rejection_does_not_overflow_the_stack() {
    // A violation at the root rejects before the deep sibling is walked. scan
    // consumes the value, so it must drain the rejected tree without recursing
    // into the deep sibling on drop.
    use serde_json::{Map, Value};
    let mut deep = Value::Object(Map::new());
    for _ in 0..100_000 {
        let mut outer = Map::new();
        outer.insert("c".into(), deep);
        deep = Value::Object(outer);
    }
    let mut root = Map::new();
    root.insert("__proto__".into(), Value::Bool(true));
    root.insert("deep".into(), deep);

    let result = scan(Value::Object(root), &Options::default());
    assert!(matches!(result, Err(Error::ForbiddenProperty)));
}
