//! Extra coverage for branches the inline spec cases leave implicit: arrays,
//! duplicate keys, deep nesting, and keys that look forbidden but are not.

mod common;

use secure_json_parse::{parse, Action, Error, Options};
use serde_json::json;

#[test]
fn top_level_array_with_proto_is_walked() {
    let text = r#"[{"__proto__": {"x": 1}}, {"a": 2}]"#;
    // Remove cleans each element.
    let opts = Options::default().proto_action(Action::Remove);
    let v = parse(text, &opts).unwrap().unwrap();
    assert_eq!(v, json!([{}, {"a": 2}]));
    // Error fires on the first element.
    assert!(matches!(
        parse(text, &Options::default()),
        Err(Error::ForbiddenProperty)
    ));
}

#[test]
fn array_nested_in_object_constructor_removed() {
    let text = r#"{"list": [{"constructor": {"prototype": {}}}]}"#;
    let opts = Options::default().constructor_action(Action::Remove);
    let v = parse(text, &opts).unwrap().unwrap();
    assert_eq!(v, json!({"list": [{}]}));
}

#[test]
fn duplicate_proto_keys_still_detected() {
    // JSON allows duplicate keys; the parser keeps the last. The key is still
    // named __proto__, so detection holds.
    let text = r#"{"__proto__": {"x": 1}, "__proto__": {"y": 2}}"#;
    assert!(matches!(
        parse(text, &Options::default()),
        Err(Error::ForbiddenProperty)
    ));
    let opts = Options::default().proto_action(Action::Remove);
    let v = parse(text, &opts).unwrap().unwrap();
    assert_eq!(v, json!({}));
}

#[test]
fn lookalike_keys_do_not_trip() {
    // proto, __proto, constructorr, constructo: none is a forbidden token.
    let inputs = [
        r#"{"proto": {"x": 7}}"#,
        r#"{"__proto": {"x": 7}}"#,
        r#"{"constructorr": {"prototype": {}}}"#,
        r#"{"constructo": {"prototype": {}}}"#,
        r#"{"_proto_": {"x": 7}}"#,
    ];
    for text in inputs {
        let v = parse(text, &Options::default());
        assert!(v.is_ok(), "expected ok for {text:?}");
    }
}

#[test]
fn benchmark_lookalike_input_is_clean() {
    // The "proto"/"__proto" pair from a benchmark input must never trip.
    let text = r#"{ "a": 5, "b": 6, "proto": { "x": 7 }, "c": { "d": 0, "e": "text", "__proto": { "y": 8 }, "f": { "g": 2 } } }"#;
    assert!(parse(text, &Options::default()).is_ok());
}

/// Build JSON nesting `depth` objects under key `a`, with `leaf` at the bottom.
fn nested(depth: usize, leaf: &str) -> String {
    let mut text = String::new();
    for _ in 0..depth {
        text.push_str(r#"{"a":"#);
    }
    text.push_str(leaf);
    for _ in 0..depth {
        text.push('}');
    }
    text
}

#[test]
fn walk_reaches_the_deepest_accepted_input() {
    // serde_json accepts 127 levels and rejects 128. The leaf object that holds
    // __proto__ is itself 2 levels deep, so 125 wrapper levels put the forbidden
    // key at the bottom of the deepest input the parser accepts. The walk must
    // descend all the way and detect it, which proves the walk handles every
    // depth the parser lets through.
    let deepest = nested(125, r#"{"__proto__": {"x": 1}}"#);
    assert!(matches!(
        parse(&deepest, &Options::default()),
        Err(Error::ForbiddenProperty)
    ));
}

#[test]
fn parser_rejects_one_past_the_ceiling() {
    // Depth 128 is past the parser recursion limit. This pins the ceiling, so a
    // future change to the limit shows up as a test failure here.
    let past = nested(128, "1");
    assert!(matches!(
        parse(&past, &Options::default()),
        Err(Error::Syntax(_))
    ));
}

#[test]
fn safe_parse_on_primitive_is_value() {
    use secure_json_parse::{safe_parse, SafeOutcome};
    assert!(matches!(safe_parse("0"), SafeOutcome::Value(_)));
    assert!(matches!(safe_parse("null"), SafeOutcome::Value(_)));
    assert!(matches!(safe_parse(r#""X""#), SafeOutcome::Value(_)));
}
