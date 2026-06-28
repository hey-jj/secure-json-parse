# secure-json-parse

Parse JSON while blocking prototype-poisoning keys.

This crate parses untrusted JSON into a `serde_json::Value` and detects two key
patterns that pollute a JavaScript object's prototype once the value is copied,
merged, or iterated:

1. a key literally named `__proto__`
2. a key named `constructor` whose value is an object containing a `prototype`
   key (a `constructor.prototype` nesting)

For each pattern you pick an action: error, remove, or ignore. A `safe` flag
turns any detected violation into `Ok(None)` instead of an error.

## Installation

```toml
[dependencies]
secure-json-parse = "0.1"
```

## Usage

```rust
use secure_json_parse::{parse, Action, Options, Error};

// Default options error on a forbidden key.
let err = parse(r#"{"__proto__": {"x": 7}}"#, &Options::default());
assert!(matches!(err, Err(Error::ForbiddenProperty)));

// Remove the key instead.
let opts = Options::default().proto_action(Action::Remove);
let value = parse(r#"{"a": 5, "__proto__": {"x": 7}}"#, &opts)
    .unwrap()
    .unwrap();
assert_eq!(value, serde_json::json!({"a": 5}));
```

### Safe parsing

`safe_parse` folds every outcome into one three-valued result:

```rust
use secure_json_parse::{safe_parse, SafeOutcome};

assert!(matches!(safe_parse(r#"{"a": 1}"#), SafeOutcome::Value(_)));
assert!(matches!(safe_parse(r#"{"__proto__": {}}"#), SafeOutcome::Violation));
assert!(matches!(safe_parse(r#"{"a": "#), SafeOutcome::Malformed));
```

`Value` means a clean parse, `Violation` means a forbidden key was found, and
`Malformed` means the JSON text was invalid.

### Scanning a parsed value

`scan` runs the same checks on a value you already hold, with no JSON parsing:

```rust
use secure_json_parse::{scan, Action, Options};
use serde_json::json;

let opts = Options::default().constructor_action(Action::Remove);
let cleaned = scan(json!({"constructor": {"prototype": {}}, "a": 1}), &opts)
    .unwrap()
    .unwrap();
assert_eq!(cleaned, json!({"a": 1}));
```

## Options

| Field                 | Values                          | Default  |
| --------------------- | ------------------------------- | -------- |
| `proto_action`        | `Error`, `Remove`, `Ignore`     | `Error`  |
| `constructor_action`  | `Error`, `Remove`, `Ignore`     | `Error`  |
| `safe`                | `true`, `false`                 | `false`  |

A `constructor` key counts as a violation only when its value is an object that
holds a `prototype` key. A `null`, array, string, number, or bool value for
`constructor` is kept. So is a `constructor` object with no `prototype` child.

Scalars and `null` skip scanning. Objects and arrays are walked, including
arrays nested anywhere in the tree.

## Byte input

`parse_bytes` and `safe_parse_bytes` take `&[u8]`. A leading UTF-8 byte order
mark (`EF BB BF`) is stripped before parsing. The text path strips a leading
`U+FEFF` for the same reason.

## License

Licensed under the [MIT license](LICENSE).
