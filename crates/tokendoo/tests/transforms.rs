//! Per-transform forward and inverse unit tests.

use serde_json::json;
use tokendoo::json::canonical::canonicalize_text;
use tokendoo::json::intern::{apply_intern, invert_intern};
use tokendoo::json::keys::{apply_keys, invert_keys};
use tokendoo::json::tabular::{apply_tabular, invert_tabular, TABULAR_SENTINEL};

#[test]
fn canonicalization_strips_whitespace_and_sorts_keys() {
    let input = r#"{ "b": 1, "a": 2 }"#;
    let out = canonicalize_text(input).expect("canon");
    assert_eq!(out, r#"{"a":2,"b":1}"#);
}

#[test]
fn key_dictionary_forward_and_inverse() {
    let v = json!([
        {"userId": 1, "createdAt": "a"},
        {"userId": 2, "createdAt": "b"},
        {"userId": 3, "createdAt": "c"},
        {"userId": 4, "createdAt": "d"},
        {"userId": 5, "createdAt": "e"},
        {"userId": 6, "createdAt": "f"},
    ]);
    let (transformed, table) = apply_keys(&v).unwrap().expect("keys should apply");
    assert!(!table.is_empty());
    let restored = invert_keys(transformed, &table).unwrap();
    assert_eq!(restored, v);
}

#[test]
fn tabular_collapse_forward_and_inverse() {
    let v = json!([
        {"a": 1, "b": "x"},
        {"a": 2, "b": "y"},
        {"a": 3, "b": "z"},
    ]);
    let (transformed, applied) = apply_tabular(v.clone()).unwrap();
    assert!(applied);
    match &transformed {
        serde_json::Value::Array(items) => {
            assert_eq!(items[0], json!(TABULAR_SENTINEL));
        }
        other => panic!("expected array, got {other:?}"),
    }
    let restored = invert_tabular(transformed).unwrap();
    assert_eq!(restored, v);
}

#[test]
fn value_interning_forward_and_inverse() {
    let long = "repeated_active_status_token";
    let v = json!({
        "a": long, "b": long, "c": long, "d": long, "e": long, "f": long,
    });
    let (transformed, table) = apply_intern(&v).unwrap().expect("intern should apply");
    assert_eq!(table, vec![long.to_string()]);
    let restored = invert_intern(transformed, &table).unwrap();
    assert_eq!(restored, v);
}

#[test]
fn value_interning_skips_tabular_schema_keys() {
    let schema_key = "very_long_schema_key_name_xyz";
    let v = json!([
        TABULAR_SENTINEL,
        [schema_key, schema_key],
        [
            ["v1", "v2"],
            ["v3", "v4"],
            ["v5", "v6"],
            ["v7", "v8"],
            ["v9", "v0"],
        ]
    ]);
    // Even if schema_key is repeated, interning must leave schema untouched.
    if let Some((transformed, _)) = apply_intern(&v).unwrap() {
        assert_eq!(
            transformed.as_array().unwrap()[1],
            json!([schema_key, schema_key])
        );
    }
}
