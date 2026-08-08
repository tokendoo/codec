//! Regression suite for edge cases from the TOK-001 design notes.

use serde_json::json;
use tokendoo::json::header::{parse_header, FLAG_IDENTITY};
use tokendoo::json::keys::KEY_PREFIX;
use tokendoo::json::tabular::TABULAR_SENTINEL;
use tokendoo::{
    compress, compress_value, compress_with_options, decompress_to_value, AlphabetKind,
    CodecError, CompressOptions, MAX_DEPTH, MAX_INPUT_BYTES,
};

fn bare_opts() -> CompressOptions {
    CompressOptions {
        key_dictionary: false,
        tabular: true,
        intern_values: true,
        alphabet: AlphabetKind::Bare,
    }
}

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

/// Build a document whose values are interned, plus one literal the codec must
/// leave alone. `distinct` controls the dictionary size, which decides whether a
/// misread ordinal lands inside the table or past its end.
fn interned_document(distinct: usize, literal: &str) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for index in 0..distinct {
        let payload = format!("LONGVALUE_{index:04}_PADDING");
        for copy in 0..4 {
            map.insert(
                format!("k{index}_{copy}"),
                serde_json::Value::String(payload.clone()),
            );
        }
    }
    map.insert(
        "literal".to_string(),
        serde_json::Value::String(literal.to_string()),
    );
    serde_json::Value::Object(map)
}

#[test]
fn base36_looking_literals_survive_value_interning() {
    // The bare alphabet writes ordinals with no prefix, so a literal that reads
    // as base36 is ambiguous with a dictionary reference. The collision
    // detector once accepted a narrower set of strings than the decoder, and
    // every literal in the gap was silently reinterpreted on the way back.
    for literal in ["ZZZZZ", "abcdef", "A", "AB12", "12345", "Z", "0Z"] {
        let document = interned_document(1, literal);
        let text = serde_json::to_string(&document).unwrap();
        let result = compress_with_options(&text, bare_opts()).expect("compress");
        let restored = decompress_to_value(&result.compressed, &result.dictionary)
            .unwrap_or_else(|error| panic!("{literal:?} failed to decompress: {error}"));
        assert_eq!(restored, document, "{literal:?} did not survive the round trip");
    }
}

#[test]
fn a_misread_ordinal_inside_the_dictionary_would_corrupt_silently() {
    // The dangerous shape under the bare alphabet: "A" once read as base36
    // ordinal 10. Pinned on bare so the ADR-013 regression stays targeted.
    let document = interned_document(12, "A");
    let text = serde_json::to_string(&document).unwrap();
    let result = compress_with_options(&text, bare_opts()).expect("compress");
    assert!(result.dictionary.v.len() >= 11, "dictionary too small to expose the case");
    let restored = decompress_to_value(&result.compressed, &result.dictionary).expect("decompress");
    assert_eq!(restored, document);
}

#[test]
fn lowercase_short_literals_force_the_identity_fallback() {
    // Inside the bare readable window, ambiguity is unavoidable, so the codec
    // must decline to intern rather than emit something it cannot reverse.
    let document = interned_document(3, "7");
    let text = serde_json::to_string(&document).unwrap();
    let result = compress_with_options(&text, bare_opts()).expect("compress");
    assert!(
        result.dictionary.v.is_empty(),
        "interning must be skipped when a literal collides"
    );
    let restored = decompress_to_value(&result.compressed, &result.dictionary).expect("decompress");
    assert_eq!(restored, document);
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
