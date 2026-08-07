//! Asserts transforms never beat the identity fallback when they would grow,
//! and that compressible fixtures actually shrink.

use tokendoo_core::json::header::{
    alphabet_flags, parse_header, FLAG_IDENTITY, FLAG_TABULAR, FLAG_VALUES, HEADER_LEN,
};
use tokendoo_core::json::legend::legend_bytes_for_flags;
use tokendoo_core::{compress, compress_value, AlphabetKind, Dictionary};

/// Framing overhead always present on the wire for the default alphabet
/// (header + dictionary carrying the derived alphabet id).
fn identity_framing_overhead() -> usize {
    let dict = Dictionary {
        a: AlphabetKind::default().wire_id().map(str::to_string),
        ..Dictionary::default()
    };
    let dict_len = serde_json::to_string(&dict).map(|s| s.len()).unwrap_or(0);
    HEADER_LEN + dict_len
}

#[test]
fn awkward_inputs_take_identity_and_do_not_exceed_identity_cost() {
    let corpus = [
        "{}",
        "[]",
        "null",
        "true",
        "false",
        "0",
        "\"x\"",
        "{\"a\":1}",
        "[{\"a\":1}]",
        "{\"a\":1,\"b\":2}",
        "[1,2,3]",
        "\"high-entropy-abcdefghijklmnopqrstuvwxyz0123456789\"",
    ];
    for input in corpus {
        let result = compress(input).expect("compress");
        let (flags, body) = parse_header(&result.compressed).unwrap();
        assert_eq!(
            flags & FLAG_IDENTITY,
            FLAG_IDENTITY,
            "expected identity for {input}"
        );
        // Authoritative guard: framed output equals framed identity, never worse.
        let identity_cost = HEADER_LEN + body.len() + {
            let d = serde_json::to_string(&result.dictionary).unwrap();
            d.len()
        };
        assert_eq!(result.bytes_after, identity_cost);
        // Framing may exceed raw input; that overhead is bounded.
        assert!(
            result.bytes_after <= input.len() + identity_framing_overhead(),
            "unexpected framing bloat for {input}"
        );
    }
}

#[test]
fn unique_keys_do_not_force_dictionary() {
    let v = serde_json::json!({
        "alpha": 1,
        "beta": 2,
        "gamma": 3,
    });
    let result = compress_value(v).unwrap();
    assert!(result.dictionary.k.is_empty());
    assert!(result.identity);
}

#[test]
fn repetitive_fixture_shrinks() {
    let mut rows = Vec::new();
    for i in 0..40 {
        rows.push(serde_json::json!({
            "userId": i,
            "createdAt": "2024-01-01T00:00:00Z",
            "status": "active",
            "role": "member",
        }));
    }
    let v = serde_json::Value::Array(rows);
    let canonical = serde_json::to_string(&v).unwrap();
    let result = compress_value(v).unwrap();
    assert!(
        result.bytes_after < canonical.len(),
        "expected shrink: before={} after={}",
        canonical.len(),
        result.bytes_after
    );
    assert!(!result.identity);
}

#[test]
fn wire_shrink_erased_by_legend_falls_back_to_identity() {
    // Small homogeneous array: tabular + intern shrink the wire a little, but
    // the transform legend is larger than that saving, so the prompt-level
    // guard must refuse the transform.
    let v = serde_json::json!([
        {"status": "active", "role": "member"},
        {"status": "active", "role": "member"},
        {"status": "active", "role": "member"},
    ]);
    let canonical = serde_json::to_string(&v).unwrap();
    let result = compress_value(v).unwrap();

    let alpha = alphabet_flags(AlphabetKind::default());
    let transform_flags = FLAG_TABULAR | FLAG_VALUES | alpha;
    let identity_flags = FLAG_IDENTITY | alpha;
    let legend_delta =
        legend_bytes_for_flags(transform_flags) - legend_bytes_for_flags(identity_flags);

    // Sanity: the fixture is in the regime the bug targeted — wire would save
    // less than the legend costs.
    assert!(
        legend_delta > 0,
        "transform legend must cost more bytes than the identity legend"
    );

    assert!(
        result.identity,
        "expected identity when legend erases the wire gain; bytes_after={} canonical={}",
        result.bytes_after,
        canonical.len()
    );
    assert_eq!(result.flags & FLAG_IDENTITY, FLAG_IDENTITY);
}
