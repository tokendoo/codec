//! Value interning of repeated string scalars, and its inverse.

use std::collections::HashMap;

use serde_json::{Map, Value};

use crate::json::tabular::TABULAR_SENTINEL;
use crate::json::{CodecError, MAX_DEPTH};

/// Control-character prefix for interned value symbols (`U+0003`).
pub const VALUE_PREFIX: char = '\u{0003}';

/// Encode an ordinal as a value symbol.
pub fn value_symbol(ordinal: usize) -> String {
    format!("{VALUE_PREFIX}{}", to_base36(ordinal))
}

/// Return `true` if any string value starts with [`VALUE_PREFIX`].
pub fn scan_value_sentinel(value: &Value) -> bool {
    scan(value, 0, false)
}

fn scan(value: &Value, depth: u32, in_tabular_schema: bool) -> bool {
    if depth > MAX_DEPTH {
        return false;
    }
    match value {
        Value::String(s) => !in_tabular_schema && s.starts_with(VALUE_PREFIX),
        Value::Array(items) => {
            if items.first().and_then(|v| v.as_str()) == Some(TABULAR_SENTINEL) && items.len() == 3
            {
                // [sentinel, schema, rows] — skip schema strings, scan rows.
                let rows_hit = items
                    .get(2)
                    .map(|rows| scan(rows, depth + 1, false))
                    .unwrap_or(false);
                return rows_hit;
            }
            items.iter().any(|v| scan(v, depth + 1, false))
        }
        Value::Object(map) => map.values().any(|v| scan(v, depth + 1, false)),
        _ => false,
    }
}

/// Apply value interning if profitable. Returns `None` when skipped.
pub fn apply_intern(value: &Value) -> Result<Option<(Value, Vec<String>)>, CodecError> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    count_strings(value, &mut counts, 0, false)?;

    let mut candidates: Vec<(&str, usize)> = counts
        .into_iter()
        .filter(|(s, n)| is_profitable(s, *n))
        .collect();
    if candidates.is_empty() {
        return Ok(None);
    }
    candidates.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));

    let table: Vec<String> = candidates.iter().map(|(s, _)| (*s).to_string()).collect();
    let mut remap: HashMap<&str, String> = HashMap::with_capacity(table.len());
    for (i, s) in table.iter().enumerate() {
        remap.insert(s.as_str(), value_symbol(i));
    }

    let transformed = rewrite(value, &remap, 0, false)?;
    Ok(Some((transformed, table)))
}

fn is_profitable(s: &str, occurrences: usize) -> bool {
    if occurrences < 2 {
        return false;
    }
    let symbol_len = 1 + 1; // prefix + at least one digit
    let saving = occurrences.saturating_mul(s.len().saturating_sub(symbol_len));
    let cost = s.len() + 3;
    saving > cost
}

fn count_strings<'a>(
    value: &'a Value,
    counts: &mut HashMap<&'a str, usize>,
    depth: u32,
    in_schema: bool,
) -> Result<(), CodecError> {
    if depth > MAX_DEPTH {
        return Err(CodecError::DepthExceeded { limit: MAX_DEPTH });
    }
    match value {
        Value::String(s) => {
            if !in_schema {
                *counts.entry(s.as_str()).or_insert(0) += 1;
            }
        }
        Value::Array(items) => {
            if items.first().and_then(|v| v.as_str()) == Some(TABULAR_SENTINEL) && items.len() == 3
            {
                // Skip schema (index 1); count inside rows (index 2).
                if let Some(rows) = items.get(2) {
                    count_strings(rows, counts, depth + 1, false)?;
                }
            } else {
                for item in items {
                    count_strings(item, counts, depth + 1, false)?;
                }
            }
        }
        Value::Object(map) => {
            for v in map.values() {
                count_strings(v, counts, depth + 1, false)?;
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
    in_schema: bool,
) -> Result<Value, CodecError> {
    if depth > MAX_DEPTH {
        return Err(CodecError::DepthExceeded { limit: MAX_DEPTH });
    }
    match value {
        Value::String(s) => {
            if in_schema {
                Ok(Value::String(s.clone()))
            } else if let Some(sym) = remap.get(s.as_str()) {
                Ok(Value::String(sym.clone()))
            } else {
                Ok(Value::String(s.clone()))
            }
        }
        Value::Array(items) => {
            if items.first().and_then(|v| v.as_str()) == Some(TABULAR_SENTINEL) && items.len() == 3
            {
                let rewritten_rows = rewrite(&items[2], remap, depth + 1, false)?;
                Ok(Value::Array(vec![
                    items[0].clone(),
                    items[1].clone(),
                    rewritten_rows,
                ]))
            } else {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(rewrite(item, remap, depth + 1, false)?);
                }
                Ok(Value::Array(out))
            }
        }
        Value::Object(map) => {
            let mut out = Map::new();
            for (k, v) in map {
                out.insert(k.clone(), rewrite(v, remap, depth + 1, false)?);
            }
            Ok(Value::Object(out))
        }
        other => Ok(other.clone()),
    }
}

/// Inverse of [`apply_intern`].
pub fn invert_intern(value: Value, table: &[String]) -> Result<Value, CodecError> {
    invert(value, table, 0, false)
}

fn invert(
    value: Value,
    table: &[String],
    depth: u32,
    in_schema: bool,
) -> Result<Value, CodecError> {
    if depth > MAX_DEPTH {
        return Err(CodecError::DepthExceeded { limit: MAX_DEPTH });
    }
    match value {
        Value::String(s) => {
            if in_schema {
                return Ok(Value::String(s));
            }
            match parse_value_symbol(&s) {
                Some(ord) => {
                    let restored = table
                        .get(ord)
                        .cloned()
                        .ok_or(CodecError::DictionaryIndexOutOfRange)?;
                    Ok(Value::String(restored))
                }
                None => Ok(Value::String(s)),
            }
        }
        Value::Array(items) => {
            if items.first().and_then(|v| v.as_str()) == Some(TABULAR_SENTINEL) && items.len() == 3
            {
                let inverted_rows = invert(items[2].clone(), table, depth + 1, false)?;
                Ok(Value::Array(vec![
                    items[0].clone(),
                    items[1].clone(),
                    inverted_rows,
                ]))
            } else {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(invert(item, table, depth + 1, false)?);
                }
                Ok(Value::Array(out))
            }
        }
        Value::Object(map) => {
            let mut out = Map::new();
            for (k, v) in map {
                out.insert(k, invert(v, table, depth + 1, false)?);
            }
            Ok(Value::Object(out))
        }
        other => Ok(other),
    }
}

fn parse_value_symbol(s: &str) -> Option<usize> {
    let mut chars = s.chars();
    if chars.next() != Some(VALUE_PREFIX) {
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
    fn intern_and_invert() {
        let long = "active_status_value_long";
        let v = json!({
            "a": long,
            "b": long,
            "c": long,
            "d": long,
            "e": long,
        });
        let (transformed, table) = apply_intern(&v).unwrap().expect("should intern");
        let restored = invert_intern(transformed, &table).unwrap();
        assert_eq!(restored, v);
    }

    #[test]
    fn does_not_intern_tabular_schema() {
        let v = json!([
            "\u{0002}T",
            ["userId", "createdAt"],
            [["a", "b"], ["c", "d"], ["e", "f"], ["g", "h"], ["i", "j"]]
        ]);
        // Schema keys are short; even if repeated elsewhere they must stay.
        let result = apply_intern(&v).unwrap();
        if let Some((transformed, _)) = result {
            if let Value::Array(items) = &transformed {
                assert_eq!(items[1], json!(["userId", "createdAt"]));
            }
        }
    }
}
