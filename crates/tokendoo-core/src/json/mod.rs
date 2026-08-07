//! Structural JSON codec pipeline.
//!
//! Transforms are applied in order: canonicalize → key dictionary → tabular
//! collapse → value interning → emit. Each step records its bit in the header
//! flag when applied. The whole pipeline falls back to the identity form when
//! the transformed **prompt** (body + dictionary + legend) is not strictly
//! smaller than the identity prompt.

pub mod alphabet;
pub mod canonical;
mod emit;
pub mod header;
pub mod intern;
pub mod keys;
pub mod legend;
pub mod tabular;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use self::alphabet::AlphabetKind;
use self::emit::{emit_body, parse_body};
use self::header::{
    alphabet_flags, alphabet_from_flags, encode_header, parse_header, FLAG_IDENTITY, FLAG_KEYS,
    FLAG_TABULAR, FLAG_VALUES, FLAG_WRAPPED_STRING,
};
use self::intern::{apply_intern_with, invert_intern_with};
use self::keys::{apply_keys_with, invert_keys_with};
use self::legend::legend_bytes_for_flags;
use self::tabular::{apply_tabular_with, invert_tabular_with};

/// Hard upper bound on input payload size in bytes.
pub const MAX_INPUT_BYTES: usize = 256 * 1024;

/// Maximum nesting depth walked by any recursive transform.
pub const MAX_DEPTH: u32 = 64;

/// Closed error type for the codec. No variant carries payload-derived text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecError {
    /// JSON parse failed at a structural location.
    ParseFailure {
        /// 1-based line number reported by the parser.
        line: u32,
        /// 1-based column number reported by the parser.
        column: u32,
    },
    /// Nesting exceeded [`MAX_DEPTH`].
    DepthExceeded {
        /// The configured depth limit.
        limit: u32,
    },
    /// Input exceeded [`MAX_INPUT_BYTES`].
    InputTooLarge {
        /// Configured byte limit.
        limit: usize,
        /// Observed byte length.
        actual: usize,
    },
    /// Compressed header was missing, truncated, or had reserved bits set.
    MalformedHeader,
    /// Header flags disagree with the supplied dictionary tables.
    DictionaryMismatch,
    /// A tabular row length did not match its schema.
    SchemaArityMismatch {
        /// Expected number of columns.
        expected: usize,
        /// Observed number of columns in a row.
        actual: usize,
    },
    /// An ordinal symbol referenced an out-of-range dictionary entry.
    DictionaryIndexOutOfRange,
    /// Requested compression mode is not implemented.
    UnsupportedMode,
}

impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseFailure { line, column } => {
                write!(f, "parse failure at line {line} column {column}")
            }
            Self::DepthExceeded { limit } => {
                write!(f, "nesting depth exceeded limit {limit}")
            }
            Self::InputTooLarge { limit, actual } => {
                write!(f, "input size {actual} exceeds limit {limit}")
            }
            Self::MalformedHeader => write!(f, "malformed compressed header"),
            Self::DictionaryMismatch => write!(f, "dictionary does not match header flags"),
            Self::SchemaArityMismatch { expected, actual } => {
                write!(f, "schema arity mismatch: expected {expected}, got {actual}")
            }
            Self::DictionaryIndexOutOfRange => write!(f, "dictionary index out of range"),
            Self::UnsupportedMode => write!(f, "unsupported compression mode"),
        }
    }
}

impl std::error::Error for CodecError {}

impl CodecError {
    /// Convert a `serde_json` error into a structural [`CodecError`], discarding
    /// any message text that might echo payload content.
    pub fn from_serde(err: &serde_json::Error) -> Self {
        Self::ParseFailure {
            line: err.line() as u32,
            column: err.column() as u32,
        }
    }
}

/// Options controlling which optional transforms may run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompressOptions {
    /// Allow the key-dictionary transform.
    #[serde(default = "default_true")]
    pub key_dictionary: bool,
    /// Allow tabular collapse of homogeneous object arrays.
    #[serde(default = "default_true")]
    pub tabular: bool,
    /// Allow value interning of repeated scalar strings.
    #[serde(default = "default_true")]
    pub intern_values: bool,
    /// Symbol alphabet (not part of the HTTP API; used by benches and core).
    #[serde(skip)]
    pub alphabet: AlphabetKind,
}

fn default_true() -> bool {
    true
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self {
            // TOK-002: key dictionary alone failed the ≥10% token rule on bare;
            // tabular + intern remain default-on.
            key_dictionary: false,
            tabular: true,
            intern_values: true,
            alphabet: AlphabetKind::default(),
        }
    }
}

/// Symbol tables shipped alongside a compressed body.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dictionary {
    /// Key dictionary: index `i` maps to symbol ordinal `i`.
    #[serde(default)]
    pub k: Vec<String>,
    /// Value dictionary: index `i` maps to intern ordinal `i`.
    #[serde(default)]
    pub v: Vec<String>,
}

/// Successful compression output.
#[derive(Debug, Clone, PartialEq)]
pub struct CompressResult {
    /// Header-prefixed compact body.
    pub compressed: String,
    /// Accompanying symbol tables.
    pub dictionary: Dictionary,
    /// Byte length of the input as received.
    pub bytes_before: usize,
    /// Byte length of `compressed` plus the serialized dictionary.
    pub bytes_after: usize,
    /// Whether the identity fallback was chosen.
    pub identity: bool,
    /// Header flags actually written.
    pub flags: u8,
}

/// Parse `input` as JSON, applying the size and depth caps.
fn parse_input(input: &str) -> Result<(Value, bool), CodecError> {
    if input.len() > MAX_INPUT_BYTES {
        return Err(CodecError::InputTooLarge {
            limit: MAX_INPUT_BYTES,
            actual: input.len(),
        });
    }
    let value: Value = serde_json::from_str(input).map_err(|e| CodecError::from_serde(&e))?;
    check_depth(&value, 0)?;
    let wrapped = matches!(value, Value::String(_));
    Ok((value, wrapped))
}

fn check_depth(value: &Value, depth: u32) -> Result<(), CodecError> {
    if depth > MAX_DEPTH {
        return Err(CodecError::DepthExceeded { limit: MAX_DEPTH });
    }
    match value {
        Value::Array(items) => {
            for item in items {
                check_depth(item, depth + 1)?;
            }
        }
        Value::Object(map) => {
            for child in map.values() {
                check_depth(child, depth + 1)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn serialized_dictionary_len(dict: &Dictionary) -> usize {
    // Exact length of the on-the-wire dictionary object the client must send.
    serde_json::to_string(dict).map(|s| s.len()).unwrap_or(0)
}

/// Compress a JSON text payload with the default options.
pub fn compress(input: &str) -> Result<CompressResult, CodecError> {
    compress_with_options(input, CompressOptions::default())
}

/// Compress a JSON text payload with explicit options.
pub fn compress_with_options(
    input: &str,
    options: CompressOptions,
) -> Result<CompressResult, CodecError> {
    let bytes_before = input.len();
    let (value, wrapped_string) = parse_input(input)?;
    compress_value_inner(value, bytes_before, wrapped_string, options)
}

/// Compress an already-parsed [`Value`].
///
/// The `bytes_before` figure is the length of the compact serialization of
/// `value`, since no original wire text is available.
pub fn compress_value(value: Value) -> Result<CompressResult, CodecError> {
    check_depth(&value, 0)?;
    let canonical = serde_json::to_string(&value).map_err(|e| CodecError::from_serde(&e))?;
    let bytes_before = canonical.len();
    compress_value_inner(value, bytes_before, false, CompressOptions::default())
}

fn compress_value_inner(
    value: Value,
    bytes_before: usize,
    wrapped_string: bool,
    options: CompressOptions,
) -> Result<CompressResult, CodecError> {
    let alphabet = options.alphabet;
    let canonical_body =
        serde_json::to_string(&value).map_err(|e| CodecError::from_serde(&e))?;

    let mut tree = value;
    let mut flags: u8 = 0;
    let mut dictionary = Dictionary::default();

    if options.key_dictionary {
        if let Some((transformed, table)) = apply_keys_with(&tree, alphabet)? {
            tree = transformed;
            dictionary.k = table;
            flags |= FLAG_KEYS;
        }
    }

    if options.tabular {
        let (transformed, applied) = apply_tabular_with(tree, alphabet)?;
        tree = transformed;
        if applied {
            flags |= FLAG_TABULAR;
        }
    }

    if options.intern_values {
        if let Some((transformed, table)) = apply_intern_with(&tree, alphabet)? {
            tree = transformed;
            dictionary.v = table;
            flags |= FLAG_VALUES;
        }
    }

    let transformed_body = emit_body(&tree)?;
    let empty_dict = Dictionary::default();
    let alpha_bits = alphabet_flags(alphabet);

    // Prompt-level sizes: wire framing + dictionary + the legend the client
    // must send for the model to read the form. Comparing wire alone would
    // compress payloads whose legend erases the gain.
    let mut candidate_flags = flags | alpha_bits;
    if wrapped_string {
        candidate_flags |= FLAG_WRAPPED_STRING;
    }
    let mut identity_flags = FLAG_IDENTITY | alpha_bits;
    if wrapped_string {
        identity_flags |= FLAG_WRAPPED_STRING;
    }

    let candidate_prompt = header::HEADER_LEN
        + transformed_body.len()
        + serialized_dictionary_len(&dictionary)
        + legend_bytes_for_flags(candidate_flags);
    let identity_prompt = header::HEADER_LEN
        + canonical_body.len()
        + serialized_dictionary_len(&empty_dict)
        + legend_bytes_for_flags(identity_flags);

    let (final_flags, final_body, final_dict, identity) =
        if candidate_prompt >= identity_prompt || flags == 0 {
            (identity_flags, canonical_body, empty_dict, true)
        } else {
            (candidate_flags, transformed_body, dictionary, false)
        };

    let compressed = encode_header(final_flags, &final_body);
    let bytes_after = compressed.len() + serialized_dictionary_len(&final_dict);

    Ok(CompressResult {
        compressed,
        dictionary: final_dict,
        bytes_before,
        bytes_after,
        identity,
        flags: final_flags,
    })
}

/// Decompress a `(compressed, dictionary)` pair into the canonical JSON text
/// of the recovered value.
pub fn decompress(compressed: &str, dictionary: &Dictionary) -> Result<String, CodecError> {
    let value = decompress_to_value(compressed, dictionary)?;
    serde_json::to_string(&value).map_err(|e| CodecError::from_serde(&e))
}

/// Decompress into a [`Value`].
pub fn decompress_to_value(
    compressed: &str,
    dictionary: &Dictionary,
) -> Result<Value, CodecError> {
    if compressed.len() > MAX_INPUT_BYTES + header::HEADER_LEN {
        return Err(CodecError::InputTooLarge {
            limit: MAX_INPUT_BYTES,
            actual: compressed.len(),
        });
    }
    let (flags, body) = parse_header(compressed)?;

    let keys_set = flags & FLAG_KEYS != 0;
    let values_set = flags & FLAG_VALUES != 0;
    let identity = flags & FLAG_IDENTITY != 0;

    if identity && (keys_set || values_set || flags & FLAG_TABULAR != 0) {
        return Err(CodecError::MalformedHeader);
    }
    if keys_set == dictionary.k.is_empty() {
        return Err(CodecError::DictionaryMismatch);
    }
    if values_set == dictionary.v.is_empty() {
        return Err(CodecError::DictionaryMismatch);
    }

    let alphabet = alphabet_from_flags(flags);
    let mut tree = parse_body(body)?;
    check_depth(&tree, 0)?;

    if identity {
        return Ok(tree);
    }

    if values_set {
        tree = invert_intern_with(tree, &dictionary.v, alphabet)?;
    }
    if flags & FLAG_TABULAR != 0 {
        tree = invert_tabular_with(tree, alphabet)?;
    }
    if keys_set {
        tree = invert_keys_with(tree, &dictionary.k, alphabet)?;
    }

    Ok(tree)
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn codec_error_display_has_no_string_payload() {
        let err = CodecError::ParseFailure {
            line: 1,
            column: 2,
        };
        assert_eq!(err.to_string(), "parse failure at line 1 column 2");
    }
}
