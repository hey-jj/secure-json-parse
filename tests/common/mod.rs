//! Shared fixture strings and helpers for the conformance tests.
//!
//! Every input here is a JSON literal lifted straight from the behavior spec.
//! Keeping them as `const` strings means each test reads against the same
//! canonical inputs.

#![allow(dead_code)]

use secure_json_parse::{Action, Options};

/// A clean object. Happy path for parse and safe_parse.
pub const OBJ: &str = r#"{"a": 5, "b": 6}"#;

/// A single-level `__proto__` key.
pub const PROTO: &str = r#"{ "a": 5, "b": 6, "__proto__": { "x": 7 } }"#;

/// A `__proto__` key at two depths.
pub const PROTO_NESTED: &str = r#"{ "a": 5, "b": 6, "__proto__": { "x": 7 }, "c": { "d": 0, "e": "text", "__proto__": { "y": 8 }, "f": { "g": 2 } } }"#;

/// A `constructor` key nesting a `prototype` key.
pub const CTOR: &str = r#"{"a": 5, "b": 6, "constructor":{"prototype":{"bar":"baz"}} }"#;

/// A `constructor` key with no `prototype` child. Not a violation.
pub const CTOR_NO_PROTO: &str = r#"{"a": 5, "b": 6,"constructor":{"bar":"baz"} }"#;

/// A `constructor` key whose value is `null`. Not a violation.
pub const CTOR_NULL: &str = r#"{"constructor": null}"#;

/// A `constructor` key nesting a `prototype` child, two depths.
pub const CTOR_NESTED: &str = r#"{ "a": 5, "b": 6, "constructor":{"prototype":{"bar":"baz"}}, "c": { "d": 0, "e": "text", "constructor":{"prototype":{"bar":"baz"}}, "f": { "g": 2 } } }"#;

/// Both a `constructor.prototype` nesting and a `__proto__` key.
pub const MIXED: &str =
    r#"{"a": 5, "b": 6, "constructor":{"prototype":{"bar":"baz"}}, "__proto__": { "x": 7 } }"#;

/// A nested `__proto__` plus a plain `constructor` key (no prototype child).
pub const MIXED_NESTED: &str = r#"{ "a": 5, "b": 6, "constructor": { "x": 7 }, "c": { "d": 0, "e": "text", "__proto__": { "y": 8 }, "f": { "g": 2 } } }"#;

/// `__proto__` as a string value, not a key. Must not trip.
pub const PROTO_VALUE: &str = r#"{"a": 5, "b": "__proto__"}"#;

/// `constructor` as a string value, not a key. Must not trip.
pub const CTOR_VALUE: &str = r#"{"a": 5, "b": "constructor"}"#;

/// A `__proto__` key alongside a data key named `hasOwnProperty`.
pub const HASOWN: &str = r#"{ "a": 5, "b": 6, "hasOwnProperty": "text", "__proto__": { "x": 7 } }"#;

/// Truncated JSON. A syntax error.
pub const INVALID: &str = r#"{"a": 5, "b": 6"#;

/// The default strict options: error on both, safe off.
pub fn error_opts() -> Options {
    Options::default()
}

/// Remove `__proto__` keys.
pub fn proto_remove() -> Options {
    Options::default().proto_action(Action::Remove)
}

/// Ignore `__proto__` keys.
pub fn proto_ignore() -> Options {
    Options::default().proto_action(Action::Ignore)
}

/// Remove `constructor` keys.
pub fn ctor_remove() -> Options {
    Options::default().constructor_action(Action::Remove)
}

/// Ignore `constructor` keys.
pub fn ctor_ignore() -> Options {
    Options::default().constructor_action(Action::Ignore)
}
