//! Canonicalization helpers.
//!
//! Parsing with `serde_json` already sorts object keys (BTreeMap) and
//! normalizes number formatting. This module exposes helpers used by tests
//! and the pipeline to re-emit a canonical serialization.

use serde_json::Value;

use crate::json::CodecError;

/// Serialize `value` to compact canonical JSON.
pub fn canonicalize(value: &Value) -> Result<String, CodecError> {
    serde_json::to_string(value).map_err(|e| CodecError::from_serde(&e))
}

/// Parse then re-emit so insignificant whitespace and key order are normalized.
pub fn canonicalize_text(input: &str) -> Result<String, CodecError> {
    let value: Value = serde_json::from_str(input).map_err(|e| CodecError::from_serde(&e))?;
    canonicalize(&value)
}
