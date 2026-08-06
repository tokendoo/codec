//! Property-based round-trip: decompress(compress(x)) == x over JSON values.

use proptest::prelude::*;
use serde_json::{Number, Value};
use tokendoo_core::{compress_value, decompress_to_value};

fn arb_number() -> impl Strategy<Value = Number> {
    prop_oneof![
        any::<i64>().prop_map(|n| Number::from(n)),
        any::<u64>().prop_map(|n| Number::from(n)),
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
