//! Value interning of repeated string scalars, and its inverse.

use std::collections::HashMap;

use serde_json::{Map, Value};

use crate::json::alphabet::AlphabetKind;
use crate::json::{CodecError, MAX_DEPTH};

/// Control-character prefix for interned value symbols (`U+0003`).
pub const VALUE_PREFIX: char = '\u{0003}';

/// Encode an ordinal as a value symbol (control alphabet).
pub fn value_symbol(ordinal: usize) -> String {
    AlphabetKind::Control.value_symbol(ordinal)
}

/// Return `true` if any string value collides with the control alphabet.
pub fn scan_value_sentinel(value: &Value) -> bool {
    AlphabetKind::Control.scan_value_collision(value)
}

/// Apply value interning if profitable (control alphabet).
pub fn apply_intern(value: &Value) -> Result<Option<(Value, Vec<String>)>, CodecError> {
    apply_intern_with(value, AlphabetKind::Control)
}

/// Apply value interning with an explicit alphabet.
pub fn apply_intern_with(
    value: &Value,
    alphabet: AlphabetKind,
) -> Result<Option<(Value, Vec<String>)>, CodecError> {
    if alphabet.scan_value_collision(value) {
        return Ok(None);
    }

    let mut counts: HashMap<&str, usize> = HashMap::new();
    count_strings(value, &mut counts, alphabet, 0)?;

    let mut candidates: Vec<(&str, usize)> = counts
        .into_iter()
        .filter(|(s, n)| is_profitable(alphabet, s, *n))
        .collect();
    if candidates.is_empty() {
        return Ok(None);
    }
    candidates.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));

    let table: Vec<String> = candidates.iter().map(|(s, _)| (*s).to_string()).collect();
    // An ordinal past the alphabet's capacity would render as a symbol the
    // decoder cannot read, which breaks the round trip without saying so.
    // Falling back to identity keeps the guarantee whatever the table size.
    if alphabet
        .ordinal_capacity()
        .is_some_and(|capacity| table.len() > capacity)
    {
        return Ok(None);
    }
    let mut remap: HashMap<&str, String> = HashMap::with_capacity(table.len());
    for (i, s) in table.iter().enumerate() {
        remap.insert(s.as_str(), alphabet.value_symbol(i));
    }

    let transformed = rewrite(value, &remap, alphabet, 0)?;
    Ok(Some((transformed, table)))
}

fn is_profitable(alphabet: AlphabetKind, s: &str, occurrences: usize) -> bool {
    if occurrences < 2 {
        return false;
    }
    let symbol_len = alphabet.value_symbol_len(0);
    let saving = occurrences.saturating_mul(s.len().saturating_sub(symbol_len));
    let cost = s.len() + 3;
    saving > cost
}

fn count_strings<'a>(
    value: &'a Value,
    counts: &mut HashMap<&'a str, usize>,
    alphabet: AlphabetKind,
    depth: u32,
) -> Result<(), CodecError> {
    if depth > MAX_DEPTH {
        return Err(CodecError::DepthExceeded { limit: MAX_DEPTH });
    }
    let sentinel = alphabet.tabular_sentinel();
    match value {
        Value::String(s) => {
            *counts.entry(s.as_str()).or_insert(0) += 1;
        }
        Value::Array(items) => {
            if items.first().and_then(|v| v.as_str()) == Some(sentinel) && items.len() == 3 {
                if let Some(rows) = items.get(2) {
                    count_strings(rows, counts, alphabet, depth + 1)?;
                }
            } else {
                for item in items {
                    count_strings(item, counts, alphabet, depth + 1)?;
                }
            }
        }
        Value::Object(map) => {
            for v in map.values() {
                count_strings(v, counts, alphabet, depth + 1)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn rewrite(
    value: &Value,
    remap: &HashMap<&str, String>,
    alphabet: AlphabetKind,
    depth: u32,
) -> Result<Value, CodecError> {
    if depth > MAX_DEPTH {
        return Err(CodecError::DepthExceeded { limit: MAX_DEPTH });
    }
    let sentinel = alphabet.tabular_sentinel();
    match value {
        Value::String(s) => {
            if let Some(sym) = remap.get(s.as_str()) {
                Ok(Value::String(sym.clone()))
            } else {
                Ok(Value::String(s.clone()))
            }
        }
        Value::Array(items) => {
            if items.first().and_then(|v| v.as_str()) == Some(sentinel) && items.len() == 3 {
                let rewritten_rows = rewrite(&items[2], remap, alphabet, depth + 1)?;
                Ok(Value::Array(vec![
                    items[0].clone(),
                    items[1].clone(),
                    rewritten_rows,
                ]))
            } else {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(rewrite(item, remap, alphabet, depth + 1)?);
                }
                Ok(Value::Array(out))
            }
        }
        Value::Object(map) => {
            let mut out = Map::new();
            for (k, v) in map {
                out.insert(k.clone(), rewrite(v, remap, alphabet, depth + 1)?);
            }
            Ok(Value::Object(out))
        }
        other => Ok(other.clone()),
    }
}

/// Inverse of [`apply_intern`] (control alphabet).
pub fn invert_intern(value: Value, table: &[String]) -> Result<Value, CodecError> {
    invert_intern_with(value, table, AlphabetKind::Control)
}

/// Inverse with an explicit alphabet.
pub fn invert_intern_with(
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
    let sentinel = alphabet.tabular_sentinel();
    match value {
        Value::String(s) => match alphabet.parse_value_symbol(&s) {
            Some(ord) => {
                let restored = table
                    .get(ord)
                    .cloned()
                    .ok_or(CodecError::DictionaryIndexOutOfRange)?;
                Ok(Value::String(restored))
            }
            None => Ok(Value::String(s)),
        },
        Value::Array(items) => {
            if items.first().and_then(|v| v.as_str()) == Some(sentinel) && items.len() == 3 {
                let inverted_rows = invert(items[2].clone(), table, alphabet, depth + 1)?;
                Ok(Value::Array(vec![
                    items[0].clone(),
                    items[1].clone(),
                    inverted_rows,
                ]))
            } else {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(invert(item, table, alphabet, depth + 1)?);
                }
                Ok(Value::Array(out))
            }
        }
        Value::Object(map) => {
            let mut out = Map::new();
            for (k, v) in map {
                out.insert(k, invert(v, table, alphabet, depth + 1)?);
            }
            Ok(Value::Object(out))
        }
        other => Ok(other),
    }
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
}
