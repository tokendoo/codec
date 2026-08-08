//! Compact body emit / parse.

use serde_json::Value;

use crate::json::CodecError;

/// Emit a compact JSON body for the transformed tree.
pub fn emit_body(value: &Value) -> Result<String, CodecError> {
    serde_json::to_string(value).map_err(|e| CodecError::from_serde(&e))
}

/// Parse a compact JSON body.
pub fn parse_body(body: &str) -> Result<Value, CodecError> {
    serde_json::from_str(body).map_err(|e| CodecError::from_serde(&e))
}
