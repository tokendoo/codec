//! Key dictionary transform and its inverse.

use std::collections::HashMap;

use serde_json::{Map, Value};

use crate::json::alphabet::AlphabetKind;
use crate::json::{CodecError, MAX_DEPTH};

/// Control-character prefix for key symbols (`U+0001`) — legacy alias for tests.
pub const KEY_PREFIX: char = '\u{0001}';

/// Encode an ordinal as a key symbol using the default (control) alphabet.
pub fn key_symbol(ordinal: usize) -> String {
    AlphabetKind::Control.key_symbol(ordinal)
}

/// Return `true` if any object key collides with the control alphabet.
pub fn scan_key_sentinel(value: &Value) -> bool {
    AlphabetKind::Control.scan_key_collision(value)
}

/// Apply the key dictionary if profitable. Returns `None` when skipped.
pub fn apply_keys(value: &Value) -> Result<Option<(Value, Vec<String>)>, CodecError> {
    apply_keys_with(value, AlphabetKind::Control)
}

/// Apply the key dictionary with an explicit alphabet.
pub fn apply_keys_with(
    value: &Value,
    alphabet: AlphabetKind,
) -> Result<Option<(Value, Vec<String>)>, CodecError> {
    if alphabet.scan_key_collision(value) {
        return Ok(None);
    }

    let mut counts: HashMap<&str, usize> = HashMap::new();
    count_keys(value, &mut counts, 0)?;

    let mut candidates: Vec<(&str, usize)> = counts
        .into_iter()
        .filter(|(k, n)| is_profitable(alphabet, k, *n))
        .collect();
    if candidates.is_empty() {
        return Ok(None);
    }
    candidates.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));

    let table: Vec<String> = candidates.iter().map(|(k, _)| (*k).to_string()).collect();
    // See the matching guard in `intern`: an ordinal past the alphabet's
    // capacity would be unreadable on the way back.
    if alphabet
        .ordinal_capacity()
        .is_some_and(|capacity| table.len() > capacity)
    {
        return Ok(None);
    }
    let mut remap: HashMap<&str, String> = HashMap::with_capacity(table.len());
    for (i, key) in table.iter().enumerate() {
        remap.insert(key.as_str(), alphabet.key_symbol(i));
    }

    let transformed = rewrite(value, &remap, 0)?;
    Ok(Some((transformed, table)))
}

fn is_profitable(alphabet: AlphabetKind, key: &str, occurrences: usize) -> bool {
    if occurrences == 0 {
        return false;
    }
    let symbol_len = alphabet.key_symbol_len(0);
    let saving = occurrences.saturating_mul(key.len().saturating_sub(symbol_len));
    let cost = key.len() + 3;
    saving > cost
}

fn count_keys<'a>(
    value: &'a Value,
    counts: &mut HashMap<&'a str, usize>,
    depth: u32,
) -> Result<(), CodecError> {
    if depth > MAX_DEPTH {
        return Err(CodecError::DepthExceeded { limit: MAX_DEPTH });
    }
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                *counts.entry(k.as_str()).or_insert(0) += 1;
                count_keys(v, counts, depth + 1)?;
            }
        }
        Value::Array(items) => {
            for item in items {
                count_keys(item, counts, depth + 1)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn rewrite(
    value: &Value,
    remap: &HashMap<&str, String>,
    depth: u32,
) -> Result<Value, CodecError> {
    if depth > MAX_DEPTH {
        return Err(CodecError::DepthExceeded { limit: MAX_DEPTH });
    }
    match value {
        Value::Object(map) => {
            let mut out = Map::new();
            for (k, v) in map {
                let new_key = remap.get(k.as_str()).cloned().unwrap_or_else(|| k.clone());
                out.insert(new_key, rewrite(v, remap, depth + 1)?);
            }
            Ok(Value::Object(out))
        }
        Value::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(rewrite(item, remap, depth + 1)?);
            }
            Ok(Value::Array(out))
        }
        other => Ok(other.clone()),
    }
}

/// Inverse of [`apply_keys`] using the control alphabet.
pub fn invert_keys(value: Value, table: &[String]) -> Result<Value, CodecError> {
    invert_keys_with(value, table, AlphabetKind::Control)
}

/// Inverse with an explicit alphabet.
pub fn invert_keys_with(
    value: Value,
    table: &[String],
    alphabet: AlphabetKind,
) -> Result<Value, CodecError> {
    invert(value, table, alphabet, 0)
}

fn invert(
    value: Value,
    table: &[String],
    alphabet: AlphabetKind,
    depth: u32,
) -> Result<Value, CodecError> {
    if depth > MAX_DEPTH {
        return Err(CodecError::DepthExceeded { limit: MAX_DEPTH });
    }
    match value {
        Value::Object(map) => {
            let mut out = Map::new();
            for (k, v) in map {
                let restored = match alphabet.parse_key_symbol(&k) {
                    Some(ord) => table
                        .get(ord)
                        .cloned()
                        .ok_or(CodecError::DictionaryIndexOutOfRange)?,
                    None => k,
                };
                out.insert(restored, invert(v, table, alphabet, depth + 1)?);
            }
            Ok(Value::Object(out))
        }
        Value::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(invert(item, table, alphabet, depth + 1)?);
            }
            Ok(Value::Array(out))
        }
        other => Ok(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn apply_and_invert() {
        let v = json!([
            {"userId": 1, "createdAt": "a"},
            {"userId": 2, "createdAt": "b"},
            {"userId": 3, "createdAt": "c"},
            {"userId": 4, "createdAt": "d"},
            {"userId": 5, "createdAt": "e"},
        ]);
        let (transformed, table) = apply_keys(&v).unwrap().expect("should apply");
        let restored = invert_keys(transformed, &table).unwrap();
        assert_eq!(restored, v);
    }

    #[test]
    fn printable_alphabet_roundtrip() {
        let v = json!([
            {"userId": 1, "createdAt": "a"},
            {"userId": 2, "createdAt": "b"},
            {"userId": 3, "createdAt": "c"},
            {"userId": 4, "createdAt": "d"},
            {"userId": 5, "createdAt": "e"},
            {"userId": 6, "createdAt": "f"},
        ]);
        let (transformed, table) =
            apply_keys_with(&v, AlphabetKind::Printable).unwrap().expect("apply");
        let restored = invert_keys_with(transformed, &table, AlphabetKind::Printable).unwrap();
        assert_eq!(restored, v);
    }
}
