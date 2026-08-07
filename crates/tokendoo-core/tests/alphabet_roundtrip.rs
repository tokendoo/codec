//! Alphabet collision and cross-alphabet round-trip coverage.

use serde_json::json;
use tokendoo_core::{
    compress_value, compress_with_options, decompress_to_value, AlphabetKind, CompressOptions,
};

fn opts(alphabet: AlphabetKind) -> CompressOptions {
    CompressOptions {
        key_dictionary: true,
        tabular: true,
        intern_values: true,
        alphabet,
    }
}

#[test]
fn all_alphabets_roundtrip_homogeneous() {
    let mut rows = Vec::new();
    for i in 0..12 {
        rows.push(json!({
            "rec_id": i,
            "ts": "2024-06-01T12:00:00Z",
            "status": "active",
            "role": "member",
        }));
    }
    let v = serde_json::Value::Array(rows);
    for alphabet in AlphabetKind::ALL {
        let text = serde_json::to_string(&v).unwrap();
        let result = compress_with_options(&text, opts(alphabet)).unwrap();
        let restored = decompress_to_value(&result.compressed, &result.dictionary).unwrap();
        assert_eq!(restored, v, "alphabet {}", alphabet.as_str());
    }
}

#[test]
fn collision_literals_do_not_break_roundtrip() {
    // Control collision
    let control = json!({ "\u{0001}0": 1, "x": "\u{0003}0", "arr": ["\u{0002}T"] });
    let r = compress_with_options(
        &serde_json::to_string(&control).unwrap(),
        opts(AlphabetKind::Control),
    )
    .unwrap();
    assert_eq!(
        decompress_to_value(&r.compressed, &r.dictionary).unwrap(),
        control
    );

    // Printable collision
    let printable = json!({ "~0": 1, "x": "^0", "arr": ["~T"] });
    let r = compress_with_options(
        &serde_json::to_string(&printable).unwrap(),
        opts(AlphabetKind::Printable),
    )
    .unwrap();
    assert_eq!(
        decompress_to_value(&r.compressed, &r.dictionary).unwrap(),
        printable
    );

    // Bare collision (short base36 keys/values)
    let bare = json!({ "0": 1, "1": 2, "x": "2", "arr": ["__T"] });
    let r = compress_with_options(&serde_json::to_string(&bare).unwrap(), opts(AlphabetKind::Bare))
        .unwrap();
    assert_eq!(
        decompress_to_value(&r.compressed, &r.dictionary).unwrap(),
        bare
    );

    // Common-word collision
    let cw = json!({ "a": 1, "b": 2, "arr": ["__T"] });
    let r = compress_with_options(
        &serde_json::to_string(&cw).unwrap(),
        opts(AlphabetKind::CommonWord),
    )
    .unwrap();
    assert_eq!(decompress_to_value(&r.compressed, &r.dictionary).unwrap(), cw);
}

#[test]
fn default_compress_uses_bare_and_roundtrips() {
    let v = json!([
        {"rec_id": 1, "ts": "t", "status": "active", "role": "member"},
        {"rec_id": 2, "ts": "t", "status": "active", "role": "member"},
        {"rec_id": 3, "ts": "t", "status": "active", "role": "member"},
        {"rec_id": 4, "ts": "t", "status": "active", "role": "member"},
    ]);
    let result = compress_value(v.clone()).unwrap();
    // Default alphabet is derived_cl100k_v1 → bits 5–7 == 4
    assert_eq!((result.flags & 0xE0) >> 5, 4);
    assert_eq!(result.dictionary.a.as_deref(), Some("c1"));
    assert!(result.compressed.starts_with("T3"));
    let restored = decompress_to_value(&result.compressed, &result.dictionary).unwrap();
    assert_eq!(restored, v);
}

#[test]
fn legacy_magics_are_rejected_after_t3_bump() {
    for payload in ["T108:{}", "T240:{}"] {
        let err = decompress_to_value(payload, &Default::default()).unwrap_err();
        assert!(
            matches!(err, tokendoo_core::CodecError::MalformedHeader),
            "{payload}"
        );
    }
}

#[test]
fn derived_alphabet_roundtrips_and_pins_dictionary_id() {
    let mut rows = Vec::new();
    for i in 0..40 {
        rows.push(json!({
            "rec_id": i,
            "ts": "2024-06-01T12:00:00Z",
            "status": "active",
            "role": "member",
        }));
    }
    let v = serde_json::Value::Array(rows);
    let text = serde_json::to_string(&v).unwrap();
    let result = compress_with_options(&text, opts(AlphabetKind::DerivedCl100kV1)).unwrap();
    assert_eq!(result.dictionary.a.as_deref(), Some("c1"));
    assert_eq!((result.flags & 0xE0) >> 5, 4);
    let restored = decompress_to_value(&result.compressed, &result.dictionary).unwrap();
    assert_eq!(restored, v);
}

#[test]
fn derived_symbol_literals_force_identity_not_silent_corruption() {
    // Pull adversarial literals from the frozen table itself.
    let table = AlphabetKind::DerivedCl100kV1
        .symbol_table()
        .expect("table");
    let probe = table[0];
    let v = json!({
        "safe": ["LONGVALUE_PAD", "LONGVALUE_PAD", "LONGVALUE_PAD"],
        "hit": probe,
    });
    let text = serde_json::to_string(&v).unwrap();
    let result = compress_with_options(&text, opts(AlphabetKind::DerivedCl100kV1)).unwrap();
    let restored = decompress_to_value(&result.compressed, &result.dictionary).unwrap();
    assert_eq!(restored, v);
}

#[test]
fn wrong_dictionary_alphabet_id_fails_loudly() {
    let mut rows = Vec::new();
    for i in 0..40 {
        rows.push(json!({
            "rec_id": i,
            "ts": "2024-06-01T12:00:00Z",
            "status": "active",
            "role": "member",
        }));
    }
    let v = serde_json::Value::Array(rows);
    let text = serde_json::to_string(&v).unwrap();
    let mut result = compress_with_options(&text, opts(AlphabetKind::DerivedCl100kV1)).unwrap();
    result.dictionary.a = Some("o1".into());
    let err = decompress_to_value(&result.compressed, &result.dictionary).unwrap_err();
    assert!(matches!(err, tokendoo_core::CodecError::AlphabetMismatch));
}
