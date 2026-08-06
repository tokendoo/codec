//! Tabular collapse of homogeneous object arrays, and its inverse.

use serde_json::{Map, Value};

use crate::json::{CodecError, MAX_DEPTH};

/// Sentinel marking a collapsed tabular array (`U+0002` + `'T'`).
pub const TABULAR_SENTINEL: &str = "\u{0002}T";

/// Return `true` if any string in the tree equals [`TABULAR_SENTINEL`].
pub fn scan_tabular_sentinel(value: &Value) -> bool {
    scan(value, 0)
}

fn scan(value: &Value, depth: u32) -> bool {
    if depth > MAX_DEPTH {
        return false;
    }
    match value {
        Value::String(s) => s == TABULAR_SENTINEL,
        Value::Array(items) => items.iter().any(|v| scan(v, depth + 1)),
        Value::Object(map) => map.values().any(|v| scan(v, depth + 1)),
        _ => false,
    }
}

/// Recursively collapse homogeneous object arrays.
///
/// Returns the transformed tree and whether any collapse occurred.
pub fn apply_tabular(value: Value) -> Result<(Value, bool), CodecError> {
    apply(value, 0)
}

fn apply(value: Value, depth: u32) -> Result<(Value, bool), CodecError> {
    if depth > MAX_DEPTH {
        return Err(CodecError::DepthExceeded { limit: MAX_DEPTH });
    }
    match value {
        Value::Array(items) => {
            let mut applied = false;
            let mut children = Vec::with_capacity(items.len());
            for item in items {
                let (child, child_applied) = apply(item, depth + 1)?;
                applied |= child_applied;
                children.push(child);
            }

            if let Some(collapsed) = try_collapse(&children) {
                Ok((collapsed, true))
            } else {
                Ok((Value::Array(children), applied))
            }
        }
        Value::Object(map) => {
            let mut applied = false;
            let mut out = Map::new();
            for (k, v) in map {
                let (child, child_applied) = apply(v, depth + 1)?;
                applied |= child_applied;
                out.insert(k, child);
            }
            Ok((Value::Object(out), applied))
        }
        other => Ok((other, false)),
    }
}

fn try_collapse(items: &[Value]) -> Option<Value> {
    if items.len() < 2 {
        return None;
    }
    let first = items.first()?.as_object()?;
    let schema: Vec<String> = first.keys().cloned().collect();
    if schema.is_empty() {
        // Empty objects: collapsing is not profitable.
        return None;
    }

    for item in items.iter().skip(1) {
        let obj = item.as_object()?;
        if obj.len() != schema.len() {
            return None;
        }
        for key in &schema {
            if !obj.contains_key(key) {
                return None;
            }
        }
    }

    // Size heuristic: collapse only when estimated after < before.
    let before = estimate_before(items, &schema);
    let after = estimate_after(items.len(), &schema);
    if after >= before {
        return None;
    }

    let mut rows = Vec::with_capacity(items.len());
    for item in items {
        let obj = item.as_object()?;
        let mut row = Vec::with_capacity(schema.len());
        for key in &schema {
            row.push(obj.get(key)?.clone());
        }
        rows.push(Value::Array(row));
    }

    Some(Value::Array(vec![
        Value::String(TABULAR_SENTINEL.to_string()),
        Value::Array(schema.into_iter().map(Value::String).collect()),
        Value::Array(rows),
    ]))
}

fn estimate_before(items: &[Value], schema: &[String]) -> usize {
    let key_overhead: usize = schema.iter().map(|k| k.len() + 3).sum();
    items.len() * key_overhead
}

fn estimate_after(row_count: usize, schema: &[String]) -> usize {
    let schema_cost: usize = schema.iter().map(|k| k.len() + 3).sum();
    // sentinel + brackets + per-row brackets
    schema_cost + 8 + row_count * 2
}

/// Inverse of [`apply_tabular`]: expand every sentinel-tagged array.
pub fn invert_tabular(value: Value) -> Result<Value, CodecError> {
    invert(value, 0)
}

fn invert(value: Value, depth: u32) -> Result<Value, CodecError> {
    if depth > MAX_DEPTH {
        return Err(CodecError::DepthExceeded { limit: MAX_DEPTH });
    }
    match value {
        Value::Array(items) => {
            if items.len() == 3 {
                if let Some(Value::String(s)) = items.first() {
                    if s == TABULAR_SENTINEL {
                        return expand_tabular(items, depth);
                    }
                }
            }
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(invert(item, depth + 1)?);
            }
            Ok(Value::Array(out))
        }
        Value::Object(map) => {
            let mut out = Map::new();
            for (k, v) in map {
                out.insert(k, invert(v, depth + 1)?);
            }
            Ok(Value::Object(out))
        }
        other => Ok(other),
    }
}

fn expand_tabular(items: Vec<Value>, depth: u32) -> Result<Value, CodecError> {
    let mut iter = items.into_iter();
    let _sentinel = iter.next();
    let schema_val = iter.next().ok_or(CodecError::MalformedHeader)?;
    let rows_val = iter.next().ok_or(CodecError::MalformedHeader)?;

    let schema_arr = schema_val.as_array().ok_or(CodecError::MalformedHeader)?;
    let mut schema: Vec<String> = Vec::with_capacity(schema_arr.len());
    for key in schema_arr {
        let s = key.as_str().ok_or(CodecError::MalformedHeader)?;
        schema.push(s.to_string());
    }

    let rows_arr = rows_val.as_array().ok_or(CodecError::MalformedHeader)?;
    let mut objects = Vec::with_capacity(rows_arr.len());
    for row in rows_arr {
        let cells = row.as_array().ok_or(CodecError::MalformedHeader)?;
        if cells.len() != schema.len() {
            return Err(CodecError::SchemaArityMismatch {
                expected: schema.len(),
                actual: cells.len(),
            });
        }
        let mut obj = Map::new();
        for (i, key) in schema.iter().enumerate() {
            let cell = cells
                .get(i)
                .cloned()
                .ok_or(CodecError::SchemaArityMismatch {
                    expected: schema.len(),
                    actual: cells.len(),
                })?;
            obj.insert(key.clone(), invert(cell, depth + 1)?);
        }
        objects.push(Value::Object(obj));
    }
    Ok(Value::Array(objects))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn collapse_and_expand() {
        let v = json!([
            {"a": 1, "b": 2},
            {"a": 3, "b": 4},
            {"a": 5, "b": 6},
        ]);
        let (transformed, applied) = apply_tabular(v.clone()).unwrap();
        assert!(applied);
        let restored = invert_tabular(transformed).unwrap();
        assert_eq!(restored, v);
    }

    #[test]
    fn null_vs_absent_does_not_collapse() {
        let v = json!([{"x": null}, {}]);
        let (transformed, applied) = apply_tabular(v.clone()).unwrap();
        assert!(!applied || transformed == v);
        // Homogeneous check fails, so structure stays an array of objects.
        assert_eq!(transformed, v);
    }
}
