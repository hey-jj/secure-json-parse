//! The tree walk that detects, removes, or rejects forbidden keys.

use crate::{Action, Error, Options};
use serde_json::Value;

/// Walk a parsed value and apply the prototype-poisoning checks.
///
/// `value` is taken by value and returned on success, mutated in place when an
/// action is [`Action::Remove`]. Scalars and `null` are returned unchanged.
/// Objects and arrays are walked.
///
/// Returns `Ok(Some(value))` when the value is clean or only had keys removed.
/// Returns `Ok(None)` when `safe` is on and a violation is found. Returns
/// [`Error::ForbiddenProperty`] when a violation is found, the matching
/// [`Action`] is [`Action::Error`], and `safe` is off.
///
/// Use this when you already hold a [`Value`] and want the same checks without
/// re-parsing. It does no JSON parsing.
///
/// # Errors
///
/// Returns [`Error::ForbiddenProperty`] on a violation under [`Action::Error`]
/// with `safe` off.
///
/// # Examples
///
/// ```
/// use secure_json_parse::{scan, Action, Options};
/// use serde_json::json;
///
/// let opts = Options::default().proto_action(Action::Remove);
/// let cleaned = scan(json!({"a": 1, "__proto__": {"x": 2}}), &opts)
///     .unwrap()
///     .unwrap();
/// assert_eq!(cleaned, json!({"a": 1}));
/// ```
pub fn scan(mut value: Value, options: &Options) -> Result<Option<Value>, Error> {
    if options.proto_action == Action::Ignore && options.constructor_action == Action::Ignore {
        return Ok(Some(value));
    }
    match walk(&mut value, options) {
        Walk::Clean => Ok(Some(value)),
        Walk::Null => Ok(None),
        Walk::Error => Err(Error::ForbiddenProperty),
    }
}

/// Result of walking one subtree.
enum Walk {
    /// No violation, or violations removed.
    Clean,
    /// A violation under `safe` mode. Maps to `Ok(None)`.
    Null,
    /// A violation under [`Action::Error`]. Maps to [`Error::ForbiddenProperty`].
    Error,
}

/// Recursively check and clean one node.
///
/// Order within a node matches the source algorithm: the `__proto__` check runs
/// first, then the `constructor` check, then children are visited. For
/// [`Action::Remove`] the removed subtree is dropped before recursion, so it is
/// never visited again.
fn walk(node: &mut Value, options: &Options) -> Walk {
    let Value::Object(map) = node else {
        // Arrays still hold scannable children; scalars and null do not.
        if let Value::Array(items) = node {
            for item in items.iter_mut() {
                match walk(item, options) {
                    Walk::Clean => {}
                    other => return other,
                }
            }
        }
        return Walk::Clean;
    };

    if options.proto_action != Action::Ignore && map.contains_key("__proto__") {
        if options.safe {
            return Walk::Null;
        }
        if options.proto_action == Action::Error {
            return Walk::Error;
        }
        map.remove("__proto__");
    }

    if options.constructor_action != Action::Ignore && is_constructor_violation(map) {
        if options.safe {
            return Walk::Null;
        }
        if options.constructor_action == Action::Error {
            return Walk::Error;
        }
        map.remove("constructor");
    }

    for child in map.values_mut() {
        match walk(child, options) {
            Walk::Clean => {}
            other => return other,
        }
    }
    Walk::Clean
}

/// Test whether a map has a `constructor` key that nests a `prototype` key.
///
/// A violation needs all of: an own `constructor` key, whose value is a JSON
/// object, that object holds a `prototype` key. A `null`, array, string,
/// number, or boolean value for `constructor` is not a violation, and neither
/// is an object without a `prototype` child.
fn is_constructor_violation(map: &serde_json::Map<String, Value>) -> bool {
    match map.get("constructor") {
        Some(Value::Object(inner)) => inner.contains_key("prototype"),
        _ => false,
    }
}
