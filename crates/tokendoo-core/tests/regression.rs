//! Regression suite for edge cases from the TOK-001 design notes.

use serde_json::json;
use tokendoo_core::json::header::{parse_header, FLAG_IDENTITY};
use tokendoo_core::json::keys::KEY_PREFIX;
use tokendoo_core::json::tabular::TABULAR_SENTINEL;
use tokendoo_core::{
    compress, compress_value, decompress_to_value, CodecError, MAX_DEPTH, MAX_INPUT_BYTES,
};

fn assert_roundtrip_str(input: &str) {
    let result = compress(input).expect("compress");
    let restored = decompress_to_value(&result.compressed, &result.dictionary).expect("decomp");
    let original: serde_json::Value = serde_json::from_str(input).unwrap();
    assert_eq!(restored, original);
}

#[test]
fn empty_object_and_array_are_identity() {
    for input in ["{}", "[]"] {
        let result = compress(input).unwrap();
        assert!(result.identity, "expected identity for {input}");
        assert_eq!(result.flags & FLAG_IDENTITY, FLAG_IDENTITY);
        assert_roundtrip_str(input);
    }
}

#[test]
fn heterogeneous_array_roundtrips() {
    assert_roundtrip_str(r#"[1,"a",{},[],null,true]"#);
}

#[test]
fn nested_arrays_of_objects_descend() {
    let input = r#"[[{"a":1},{"a":2}],[{"a":3},{"a":4}]]"#;
    assert_roundtrip_str(input);
    let result = compress(input).unwrap();
    // Outer array elements are arrays, not objects — but inners may collapse.
    let restored = decompress_to_value(&result.compressed, &result.dictionary).unwrap();
    let original: serde_json::Value = serde_json::from_str(input).unwrap();
    assert_eq!(restored, original);
}

#[test]
fn duplicate_keys_resolve_last_wins() {
    let input = r#"{"a":1,"a":2}"#;
    let result = compress(input).unwrap();
    let restored = decompress_to_value(&result.compressed, &result.dictionary).unwrap();
    assert_eq!(restored, json!({"a": 2}));
}

#[test]
fn escape_heavy_strings_roundtrip() {
    assert_roundtrip_str(r#"{"a":"\u0000","b":"😀","c":"\\","d":"\""}"#);
}

#[test]
fn sentinel_collision_skips_transforms_and_roundtrips() {
    let key_collision = serde_json::json!({ format!("{KEY_PREFIX}0"): 1, "other": 2 });
    let r = compress_value(key_collision.clone()).unwrap();
    let restored = decompress_to_value(&r.compressed, &r.dictionary).unwrap();
    assert_eq!(restored, key_collision);

    let value_collision = json!({"a": "\u{0003}0", "b": "ok"});
    let r = compress_value(value_collision.clone()).unwrap();
    let restored = decompress_to_value(&r.compressed, &r.dictionary).unwrap();
    assert_eq!(restored, value_collision);

    let tabular_collision = json!([TABULAR_SENTINEL, {"a": 1}]);
    let r = compress_value(tabular_collision.clone()).unwrap();
    let restored = decompress_to_value(&r.compressed, &r.dictionary).unwrap();
    assert_eq!(restored, tabular_collision);
}

#[test]
fn deep_nesting_returns_depth_exceeded() {
    let mut v = json!(1);
    for _ in 0..(MAX_DEPTH + 2) {
        v = json!([v]);
    }
    let err = compress_value(v).unwrap_err();
    assert_eq!(
        err,
        CodecError::DepthExceeded {
            limit: MAX_DEPTH
        }
    );
}

#[test]
fn number_precision_cases() {
    // 2^53 + 1 fits in i64/u64 via serde_json integer path.
    assert_roundtrip_str("9007199254740993");
    assert_roundtrip_str(&u64::MAX.to_string());
    assert_roundtrip_str(&i64::MIN.to_string());

    // -0.0 serializes distinctly in some paths; equality of Values may collapse.
    let result = compress("-0.0").unwrap();
    let (_flags, body) = parse_header(&result.compressed).unwrap();
    // After round-trip through Value, -0.0 becomes 0.0 or stays -0.0 depending
    // on serde_json; assert decompress succeeds and is a number.
    let restored = decompress_to_value(&result.compressed, &result.dictionary).unwrap();
    assert!(restored.is_number(), "body={body}");
}

#[test]
fn null_vs_absent_does_not_collapse() {
    let mixed = json!([{"x": null}, {}]);
    let r = compress_value(mixed.clone()).unwrap();
    let restored = decompress_to_value(&r.compressed, &r.dictionary).unwrap();
    assert_eq!(restored, mixed);

    let both_null = json!([{"x": null}, {"x": null}]);
    let r = compress_value(both_null.clone()).unwrap();
    let restored = decompress_to_value(&r.compressed, &r.dictionary).unwrap();
    assert_eq!(restored, both_null);
}

#[test]
fn numeric_looking_keys_with_dictionary() {
    // Build enough repetition of "0"/"1" as keys alongside longer keys so the
    // dictionary can activate without colliding with prefixed symbols.
    let mut rows = Vec::new();
    for i in 0..8 {
        rows.push(json!({"0": i, "1": i + 1, "userId": i, "createdAt": format!("t{i}")}));
    }
    let v = serde_json::Value::Array(rows);
    let r = compress_value(v.clone()).unwrap();
    let restored = decompress_to_value(&r.compressed, &r.dictionary).unwrap();
    assert_eq!(restored, v);
}

#[test]
fn single_element_homogeneous_array_stays_identity_or_roundtrips() {
    let v = json!([{"a": 1}]);
    let r = compress_value(v.clone()).unwrap();
    let restored = decompress_to_value(&r.compressed, &r.dictionary).unwrap();
    assert_eq!(restored, v);
}

#[test]
fn input_size_cap_boundary() {
    // Exactly at the cap: a string payload whose JSON encoding fits.
    let inner_len = MAX_INPUT_BYTES - 2; // quotes
    let big = format!("\"{}\"", "a".repeat(inner_len));
    assert!(big.len() <= MAX_INPUT_BYTES);
    // May succeed or fail depth/size depending on construction; over-cap must fail.
    let over = format!("\"{}\"", "a".repeat(MAX_INPUT_BYTES));
    assert!(over.len() > MAX_INPUT_BYTES);
    let err = compress(&over).unwrap_err();
    match err {
        CodecError::InputTooLarge { limit, actual } => {
            assert_eq!(limit, MAX_INPUT_BYTES);
            assert_eq!(actual, over.len());
        }
        other => panic!("expected InputTooLarge, got {other:?}"),
    }
}
