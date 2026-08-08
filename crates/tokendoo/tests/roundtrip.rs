//! Property-based round-trip: decompress(compress(x)) == x over JSON values.

use proptest::prelude::*;
use serde_json::{json, Number, Value};
use tokendoo::{
    compress_value, compress_with_options, decompress_to_value, AlphabetKind, CompressOptions,
};

fn arb_number() -> impl Strategy<Value = Number> {
    prop_oneof![
        any::<i64>().prop_map(Number::from),
        any::<u64>().prop_map(Number::from),
        (-1.0e6f64..1.0e6f64)
            .prop_filter("finite", |f| f.is_finite())
            .prop_map(|f| Number::from_f64(f).expect("finite")),
        Just(Number::from(i64::MIN)),
        Just(Number::from(i64::MAX)),
        Just(Number::from(u64::MAX)),
        Just(Number::from_f64(f64::MIN_POSITIVE).expect("subnormal-ish")),
    ]
}

fn arb_string() -> impl Strategy<Value = String> {
    prop_oneof![
        ".*".prop_map(|s| s),
        Just(String::from("\u{0000}")),
        Just(String::from("😀")),
        Just(String::from("\\")),
        Just(String::from("\"")),
        Just(String::from("café")),
        Just(String::from("日本語")),
    ]
}

fn arb_leaf() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        arb_number().prop_map(Value::Number),
        arb_string().prop_map(Value::String),
    ]
}

fn arb_value(depth: u32) -> BoxedStrategy<Value> {
    if depth == 0 {
        return arb_leaf().boxed();
    }
    let leaf = arb_leaf();
    let arr = prop::collection::vec(arb_value(depth - 1), 0..6).prop_map(Value::Array);
    let obj = prop::collection::btree_map(arb_string(), arb_value(depth - 1), 0..6)
        .prop_map(|m| Value::Object(m.into_iter().collect()));
    prop_oneof![leaf, arr, obj].boxed()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn roundtrip_value_equality(value in arb_value(3)) {
        // Restrict to the JSON-representable subset: floats that do not survive
        // a serialize/parse cycle are outside the product guarantee.
        let Ok(canonical_text) = serde_json::to_string(&value) else {
            return Ok(());
        };
        let Ok(canonical) = serde_json::from_str::<Value>(&canonical_text) else {
            return Ok(());
        };
        let result = compress_value(canonical.clone()).expect("compress");
        let restored = decompress_to_value(&result.compressed, &result.dictionary)
            .expect("decompress");
        prop_assert_eq!(restored, canonical);
    }
}

/// Strings that are cheap to confuse with a symbol under one alphabet or another.
fn arb_symbol_like() -> BoxedStrategy<String> {
    prop_oneof![
        "[0-9a-z]{1,6}".boxed(),
        "[0-9A-Z]{1,6}".boxed(),
        "[0-9a-zA-Z_]{1,8}".boxed(),
        Just("__T".to_string()).boxed(),
        Just("~T".to_string()).boxed(),
        Just("~0".to_string()).boxed(),
        Just("^0".to_string()).boxed(),
        Just("\u{0001}0".to_string()).boxed(),
        Just("\u{0003}0".to_string()).boxed(),
        Just("\u{0002}T".to_string()).boxed(),
    ]
    .boxed()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Round trip over documents built to *reach* the dictionary transforms.
    ///
    /// `roundtrip_value_equality` above samples arbitrary values, which almost
    /// never repeat a string often enough for interning to be profitable, so it
    /// explores the identity path far more than the transform paths. This case
    /// forces repeated, profitable payloads to sit next to literals that look
    /// like symbols — the region where the collision detector and the decoder
    /// have to agree exactly.
    #[test]
    fn roundtrip_survives_symbol_like_literals(
        distinct in 1usize..8,
        repeats in 2usize..5,
        literals in prop::collection::vec(arb_symbol_like(), 1..5),
        extra_keys in prop::collection::vec(arb_symbol_like(), 0..4),
        alphabet_index in 0usize..4,
    ) {
        let alphabet = AlphabetKind::ALL[alphabet_index];
        let mut map = serde_json::Map::new();

        // Repeated long payloads, so interning is profitable and actually runs.
        for index in 0..distinct {
            let payload = format!("LONGVALUE_{index:04}_PADDING");
            for copy in 0..repeats {
                map.insert(format!("k{index}_{copy}"), Value::String(payload.clone()));
            }
        }

        // The literals as bare values.
        for (index, literal) in literals.iter().enumerate() {
            map.insert(format!("literal_{index}"), Value::String(literal.clone()));
        }

        // The literals as keys, for the key-dictionary path.
        for key in &extra_keys {
            map.insert(key.clone(), Value::String("LONGVALUE_0000_PADDING".to_string()));
        }

        // The literals inside a homogeneous array, for the tabular path. This is
        // the shipped combination: tabular collapse plus value interning.
        let mut rows = Vec::new();
        for index in 0..(distinct * repeats) {
            rows.push(json!({
                "row_id": index,
                "code": literals[index % literals.len()],
                "payload": format!("LONGVALUE_{:04}_PADDING", index % distinct),
            }));
        }
        map.insert("rows".to_string(), Value::Array(rows));

        let document = Value::Object(map);
        let text = serde_json::to_string(&document).expect("serialize");
        let options = CompressOptions {
            key_dictionary: true,
            tabular: true,
            intern_values: true,
            alphabet,
        };
        let result = compress_with_options(&text, options).expect("compress");
        let restored = decompress_to_value(&result.compressed, &result.dictionary)
            .expect("decompress");
        prop_assert_eq!(restored, document);
    }
}
