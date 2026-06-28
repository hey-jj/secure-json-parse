//! Parse JSON while blocking prototype-poisoning keys.
//!
//! This crate parses untrusted JSON and detects two key patterns that pollute a
//! JavaScript object's prototype once the parsed value is copied, merged, or
//! iterated:
//!
//! 1. a key literally named `__proto__`
//! 2. a key named `constructor` whose value is an object that contains a
//!    `prototype` key (a `constructor.prototype` nesting)
//!
//! For each pattern you choose an [`Action`]: error out, remove the key, or
//! ignore it and behave like a plain JSON parse. A `safe` flag turns any
//! detected violation into `Ok(None)` instead of an error.
//!
//! The parsed value is a [`serde_json::Value`]. In a `Value` tree `__proto__`
//! and `constructor` are ordinary map keys, so the danger is latent rather than
//! immediate. The job here is to detect, remove, or reject those keys before the
//! value reaches code that would treat them as a prototype.
//!
//! # Quick start
//!
//! ```
//! use secure_json_parse::{parse, Action, Options, Error};
//!
//! // Default options error on a forbidden key.
//! let err = parse(r#"{"__proto__": {"x": 7}}"#, &Options::default());
//! assert!(matches!(err, Err(Error::ForbiddenProperty)));
//!
//! // Remove the key instead.
//! let opts = Options::default().proto_action(Action::Remove);
//! let value = parse(r#"{"a": 5, "__proto__": {"x": 7}}"#, &opts).unwrap().unwrap();
//! assert_eq!(value, serde_json::json!({"a": 5}));
//! ```
//!
//! # Safe parsing
//!
//! [`safe_parse`] folds every outcome into one three-valued result:
//!
//! ```
//! use secure_json_parse::{safe_parse, SafeOutcome};
//!
//! // Clean input parses to a value.
//! assert!(matches!(safe_parse(r#"{"a": 1}"#), SafeOutcome::Value(_)));
//! // A forbidden key yields Null.
//! assert!(matches!(safe_parse(r#"{"__proto__": {}}"#), SafeOutcome::Null));
//! // Malformed JSON yields Undefined.
//! assert!(matches!(safe_parse(r#"{"a": "#), SafeOutcome::Undefined));
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod scan;

pub use scan::scan;

use serde_json::Value;

/// What to do when a forbidden key is found.
///
/// `proto_action` and `constructor_action` each take one of these. The default
/// is [`Action::Error`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Action {
    /// Reject the input with [`Error::ForbiddenProperty`].
    #[default]
    Error,
    /// Delete the offending key from the result.
    Remove,
    /// Skip the check for this key and keep it.
    Ignore,
}

/// Configuration for [`parse`], [`parse_bytes`], and [`scan`].
///
/// Build one with [`Options::default`] and the chained setters. The defaults
/// match a strict parser: both actions are [`Action::Error`] and `safe` is off.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Options {
    /// Action for a `__proto__` key.
    pub proto_action: Action,
    /// Action for a `constructor.prototype` nesting.
    pub constructor_action: Action,
    /// When true, a detected violation returns `Ok(None)` instead of an error.
    pub safe: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            proto_action: Action::Error,
            constructor_action: Action::Error,
            safe: false,
        }
    }
}

impl Options {
    /// Set the action for `__proto__` keys.
    #[must_use]
    pub fn proto_action(mut self, action: Action) -> Self {
        self.proto_action = action;
        self
    }

    /// Set the action for `constructor.prototype` nestings.
    #[must_use]
    pub fn constructor_action(mut self, action: Action) -> Self {
        self.constructor_action = action;
        self
    }

    /// Set the `safe` flag. When true a violation returns `Ok(None)`.
    #[must_use]
    pub fn safe(mut self, safe: bool) -> Self {
        self.safe = safe;
        self
    }
}

/// Why a parse failed.
///
/// [`Error::Syntax`] wraps a malformed-JSON error from the underlying parser.
/// [`Error::ForbiddenProperty`] reports a `__proto__` or `constructor.prototype`
/// violation when the relevant [`Action`] is [`Action::Error`] and `safe` is
/// off. Both violation kinds share one variant and one message.
#[derive(Debug)]
pub enum Error {
    /// The JSON text was malformed.
    Syntax(serde_json::Error),
    /// The value held a forbidden prototype property.
    ForbiddenProperty,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Syntax(e) => write!(f, "{e}"),
            Error::ForbiddenProperty => write!(f, "Object contains forbidden prototype property"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Syntax(e) => Some(e),
            Error::ForbiddenProperty => None,
        }
    }
}

/// The three outcomes of [`safe_parse`].
///
/// These mirror the JavaScript `safeParse` return values: a parsed value,
/// `null` for a security violation, and `undefined` for a parse error.
#[derive(Debug)]
pub enum SafeOutcome {
    /// Parsed cleanly. Holds the value.
    Value(Value),
    /// A forbidden prototype property was found. Maps to JavaScript `null`.
    Null,
    /// The JSON text was malformed. Maps to JavaScript `undefined`.
    Undefined,
}

/// Drop a leading `U+FEFF` byte order mark from a string.
///
/// The UTF-8 BOM is `EF BB BF`, which decodes to `U+FEFF`. Removing it before
/// parsing matches a JavaScript parser that strips a leading BOM.
fn strip_bom(text: &str) -> &str {
    text.strip_prefix('\u{FEFF}').unwrap_or(text)
}

/// Parse JSON text and apply prototype-poisoning checks.
///
/// On success returns `Ok(Some(value))`. When `safe` is on and a violation is
/// found, returns `Ok(None)`. A malformed input gives [`Error::Syntax`]; a
/// violation under [`Action::Error`] gives [`Error::ForbiddenProperty`].
///
/// Scalars and `null` skip scanning and are returned as is. Objects and arrays
/// are walked.
///
/// # Errors
///
/// Returns [`Error::Syntax`] if `text` is not valid JSON, or
/// [`Error::ForbiddenProperty`] if a forbidden key is found and the matching
/// [`Action`] is [`Action::Error`] with `safe` off.
///
/// # Examples
///
/// ```
/// use secure_json_parse::{parse, Action, Options};
///
/// let opts = Options::default().constructor_action(Action::Remove);
/// let v = parse(r#"{"constructor": {"prototype": {}}, "a": 1}"#, &opts)
///     .unwrap()
///     .unwrap();
/// assert_eq!(v, serde_json::json!({"a": 1}));
/// ```
pub fn parse(text: &str, options: &Options) -> Result<Option<Value>, Error> {
    let text = strip_bom(text);
    let value: Value = serde_json::from_str(text).map_err(Error::Syntax)?;
    scan(value, options)
}

/// Parse JSON bytes and apply prototype-poisoning checks.
///
/// Same as [`parse`] but takes UTF-8 bytes. A leading UTF-8 byte order mark
/// (`EF BB BF`) is stripped before parsing.
///
/// # Errors
///
/// Returns [`Error::Syntax`] if the bytes are not valid UTF-8 JSON, or
/// [`Error::ForbiddenProperty`] under the same rules as [`parse`].
pub fn parse_bytes(bytes: &[u8], options: &Options) -> Result<Option<Value>, Error> {
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    let value: Value = serde_json::from_slice(bytes).map_err(Error::Syntax)?;
    scan(value, options)
}

/// Parse JSON text and fold every outcome into a [`SafeOutcome`].
///
/// Runs with both actions at [`Action::Error`] and `safe` on. A clean parse
/// returns [`SafeOutcome::Value`]. A forbidden key returns [`SafeOutcome::Null`].
/// Malformed JSON returns [`SafeOutcome::Undefined`]. This never returns an
/// error type.
///
/// # Examples
///
/// ```
/// use secure_json_parse::{safe_parse, SafeOutcome};
///
/// match safe_parse(r#"{"a": 1}"#) {
///     SafeOutcome::Value(v) => assert_eq!(v, serde_json::json!({"a": 1})),
///     _ => panic!("expected a value"),
/// }
/// ```
pub fn safe_parse(text: &str) -> SafeOutcome {
    let opts = Options::default().safe(true);
    fold(parse(text, &opts))
}

/// Parse JSON bytes and fold every outcome into a [`SafeOutcome`].
///
/// The byte counterpart to [`safe_parse`]. A leading UTF-8 byte order mark is
/// stripped before parsing.
pub fn safe_parse_bytes(bytes: &[u8]) -> SafeOutcome {
    let opts = Options::default().safe(true);
    fold(parse_bytes(bytes, &opts))
}

/// Collapse a parse result into a [`SafeOutcome`].
fn fold(result: Result<Option<Value>, Error>) -> SafeOutcome {
    match result {
        Ok(Some(value)) => SafeOutcome::Value(value),
        Ok(None) => SafeOutcome::Null,
        Err(_) => SafeOutcome::Undefined,
    }
}
