//! Key dictionary transform and its inverse.

use std::collections::HashMap;

use serde_json::{Map, Value};

use crate::json::{CodecError, MAX_DEPTH};

/// Control-character prefix for key symbols (`U+0001`).
pub const KEY_PREFIX: char = '\u{0001}';

/// Encode an ordinal as a key symbol.
pub fn key_symbol(ordinal: usize) -> String {
    format!("{KEY_PREFIX}{}", to_base36(ordinal))
}

/// Return `true` if any object key in the tree starts with [`KEY_PREFIX`].
pub fn scan_key_sentinel(value: &Value) -> bool {
    scan(value, 0)
}

fn scan(value: &Value, depth: u32) -> bool {
    if depth > MAX_DEPTH {
        return false;
    }
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                if k.starts_with(KEY_PREFIX) {
                    return true;
                }
                if scan(v, depth + 1) {
                    return true;
                }
            }
            false
        }
        Value::Array(items) => items.iter().any(|v| scan(v, depth + 1)),
        _ => false,
    }
}

/// Apply the key dictionary if it is profitable. Returns `None` when skipped.
pub fn apply_keys(value: &Value) -> Result<Option<(Value, Vec<String>)>, CodecError> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    count_keys(value, &mut counts, 0)?;

    let mut candidates: Vec<(&str, usize)> = counts
        .into_iter()
        .filter(|(k, n)| is_profitable(k, *n))
        .collect();
    if candidates.is_empty() {
        return Ok(None);
    }
    candidates.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));

    let table: Vec<String> = candidates.iter().map(|(k, _)| (*k).to_string()).collect();
    let mut remap: HashMap<&str, String> = HashMap::with_capacity(table.len());
    for (i, key) in table.iter().enumerate() {
        remap.insert(key.as_str(), key_symbol(i));
    }

    let transformed = rewrite(value, &remap, 0)?;
    Ok(Some((transformed, table)))
}

fn is_profitable(key: &str, occurrences: usize) -> bool {
    if occurrences == 0 {
        return false;
    }
    // Estimate: each occurrence saves (len(key) - len(symbol)) quoted bytes;
    // table cost is roughly len(key) + 3 (quotes + comma).
    let symbol_len = 1 + base36_len(0); // worst-case underestimate uses short symbol
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

/// Inverse of [`apply_keys`].
pub fn invert_keys(value: Value, table: &[String]) -> Result<Value, CodecError> {
    invert(value, table, 0)
}

fn invert(value: Value, table: &[String], depth: u32) -> Result<Value, CodecError> {
    if depth > MAX_DEPTH {
        return Err(CodecError::DepthExceeded { limit: MAX_DEPTH });
    }
    match value {
        Value::Object(map) => {
            let mut out = Map::new();
            for (k, v) in map {
                let restored = match parse_key_symbol(&k) {
                    Some(ord) => table
                        .get(ord)
                        .cloned()
                        .ok_or(CodecError::DictionaryIndexOutOfRange)?,
                    None => k,
                };
                out.insert(restored, invert(v, table, depth + 1)?);
            }
            Ok(Value::Object(out))
        }
        Value::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(invert(item, table, depth + 1)?);
            }
            Ok(Value::Array(out))
        }
        other => Ok(other),
    }
}

fn parse_key_symbol(s: &str) -> Option<usize> {
    let mut chars = s.chars();
    if chars.next() != Some(KEY_PREFIX) {
        return None;
    }
    from_base36(chars.as_str())
}

fn to_base36(mut n: usize) -> String {
    const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    if n == 0 {
        return "0".to_string();
    }
    let mut buf = Vec::new();
    while n > 0 {
        buf.push(DIGITS[n % 36]);
        n /= 36;
    }
    buf.reverse();
    String::from_utf8(buf).unwrap_or_else(|_| "0".to_string())
}

fn base36_len(n: usize) -> usize {
    to_base36(n).len()
}

fn from_base36(s: &str) -> Option<usize> {
    if s.is_empty() {
        return None;
    }
    let mut n: usize = 0;
    for c in s.chars() {
        let digit = match c {
            '0'..='9' => (c as u8 - b'0') as usize,
            'a'..='z' => (c as u8 - b'a') as usize + 10,
            'A'..='Z' => (c as u8 - b'A') as usize + 10,
            _ => return None,
        };
        n = n.checked_mul(36)?.checked_add(digit)?;
    }
    Some(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn symbol_roundtrip() {
        assert_eq!(parse_key_symbol(&key_symbol(0)), Some(0));
        assert_eq!(parse_key_symbol(&key_symbol(35)), Some(35));
        assert_eq!(parse_key_symbol(&key_symbol(36)), Some(36));
    }

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
}
