//! Node.js WebAssembly bindings for the Tokendoo codec.
//!
//! This crate is a build input for the npm package `tokendoo`. It is not
//! published to crates.io (`publish = false`).

#![deny(missing_docs)]

use std::panic::{catch_unwind, AssertUnwindSafe};

use js_sys::{Error, Object, Reflect};
use serde::Deserialize;
use serde_json::Value;
use tokendoo::json::header::{
    alphabet_from_flags as core_alphabet_from_flags, FLAG_ALPHABET_MASK as CORE_FLAG_ALPHABET_MASK,
    FLAG_IDENTITY as CORE_FLAG_IDENTITY, FLAG_KEYS as CORE_FLAG_KEYS,
    FLAG_TABULAR as CORE_FLAG_TABULAR, FLAG_VALUES as CORE_FLAG_VALUES,
    FLAG_WRAPPED_STRING as CORE_FLAG_WRAPPED_STRING,
};
use tokendoo::{
    compress as core_compress, compress_value as core_compress_value,
    compress_with_options as core_compress_with_options, decompress as core_decompress,
    decompress_to_value as core_decompress_to_value, AlphabetKind, CodecError, CompressOptions,
    CompressResult, Dictionary, MAX_DEPTH as CORE_MAX_DEPTH,
    MAX_INPUT_BYTES as CORE_MAX_INPUT_BYTES,
};
use wasm_bindgen::prelude::*;

/// Hard upper bound on input payload size in bytes.
#[wasm_bindgen]
pub fn max_input_bytes() -> u32 {
    CORE_MAX_INPUT_BYTES as u32
}

/// Maximum nesting depth walked by any recursive transform.
#[wasm_bindgen]
pub fn max_depth() -> u32 {
    CORE_MAX_DEPTH
}

/// Key dictionary applied (`0x01`).
#[wasm_bindgen]
pub fn flag_keys() -> u8 {
    CORE_FLAG_KEYS
}

/// Tabular collapse applied at least once (`0x02`).
#[wasm_bindgen]
pub fn flag_tabular() -> u8 {
    CORE_FLAG_TABULAR
}

/// Value interning applied (`0x04`).
#[wasm_bindgen]
pub fn flag_values() -> u8 {
    CORE_FLAG_VALUES
}

/// Identity fallback; exclusive with other transform bits (`0x08`).
#[wasm_bindgen]
pub fn flag_identity() -> u8 {
    CORE_FLAG_IDENTITY
}

/// Original payload was a JSON string wrapping JSON text (`0x10`).
#[wasm_bindgen]
pub fn flag_wrapped_string() -> u8 {
    CORE_FLAG_WRAPPED_STRING
}

/// Alphabet id occupies bits 5–7 (`0xE0`).
#[wasm_bindgen]
pub fn flag_alphabet_mask() -> u8 {
    CORE_FLAG_ALPHABET_MASK
}

#[derive(Debug, Deserialize)]
struct OptionsIn {
    #[serde(default = "default_false")]
    key_dictionary: bool,
    #[serde(default = "default_true")]
    tabular: bool,
    #[serde(default = "default_true")]
    intern_values: bool,
    #[serde(default)]
    alphabet: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn parse_alphabet(name: &str) -> Result<AlphabetKind, CodecError> {
    match name {
        "control" => Ok(AlphabetKind::Control),
        "printable" => Ok(AlphabetKind::Printable),
        "bare" => Ok(AlphabetKind::Bare),
        "common_word" => Ok(AlphabetKind::CommonWord),
        "derived_cl100k_v1" => Ok(AlphabetKind::DerivedCl100kV1),
        "derived_o200k_v1" => Ok(AlphabetKind::DerivedO200kV1),
        "derived_llama_v1" => Ok(AlphabetKind::DerivedLlamaV1),
        _ => Err(CodecError::UnsupportedMode),
    }
}

fn options_from_js(value: JsValue) -> Result<CompressOptions, JsValue> {
    if value.is_undefined() || value.is_null() {
        return Ok(CompressOptions::default());
    }
    let parsed: OptionsIn =
        serde_wasm_bindgen::from_value(value).map_err(|_| type_error("invalid compress options"))?;
    let mut options = CompressOptions {
        key_dictionary: parsed.key_dictionary,
        tabular: parsed.tabular,
        intern_values: parsed.intern_values,
        alphabet: AlphabetKind::default(),
    };
    if let Some(name) = parsed.alphabet.as_deref() {
        options.alphabet = parse_alphabet(name).map_err(codec_error_to_js)?;
    }
    Ok(options)
}

fn dictionary_from_js(value: JsValue) -> Result<Dictionary, JsValue> {
    serde_wasm_bindgen::from_value(value).map_err(|_| type_error("invalid dictionary"))
}

fn result_to_js(result: CompressResult) -> Result<JsValue, JsValue> {
    let dict = serde_wasm_bindgen::to_value(&result.dictionary)
        .map_err(|_| internal_error("dictionary encode failed"))?;
    let obj = Object::new();
    set_str(&obj, "compressed", &result.compressed)?;
    Reflect::set(&obj, &"dictionary".into(), &dict).map_err(reflect_err)?;
    set_u32(&obj, "bytes_before", result.bytes_before as u32)?;
    set_u32(&obj, "bytes_after", result.bytes_after as u32)?;
    Reflect::set(&obj, &"identity".into(), &JsValue::from_bool(result.identity))
        .map_err(reflect_err)?;
    set_u32(&obj, "flags", u32::from(result.flags))?;
    Ok(obj.into())
}

fn set_str(obj: &Object, key: &str, value: &str) -> Result<(), JsValue> {
    Reflect::set(obj, &key.into(), &JsValue::from_str(value)).map_err(reflect_err)?;
    Ok(())
}

fn set_u32(obj: &Object, key: &str, value: u32) -> Result<(), JsValue> {
    Reflect::set(obj, &key.into(), &JsValue::from(value)).map_err(reflect_err)?;
    Ok(())
}

fn reflect_err(_err: JsValue) -> JsValue {
    internal_error("failed to build result object")
}

fn type_error(message: &str) -> JsValue {
    let err = Error::new(message);
    let _ = Reflect::set(&err, &"name".into(), &"TypeError".into());
    err.into()
}

fn internal_error(message: &str) -> JsValue {
    let err = Error::new(message);
    let _ = Reflect::set(&err, &"name".into(), &"InternalError".into());
    let _ = Reflect::set(&err, &"kind".into(), &"InternalError".into());
    err.into()
}

fn codec_error_to_js(err: CodecError) -> JsValue {
    let err_obj = Error::new(&err.to_string());
    let _ = Reflect::set(&err_obj, &"name".into(), &"CodecError".into());
    match err {
        CodecError::ParseFailure { line, column } => {
            let _ = Reflect::set(&err_obj, &"kind".into(), &"ParseFailure".into());
            let _ = Reflect::set(&err_obj, &"line".into(), &JsValue::from(line));
            let _ = Reflect::set(&err_obj, &"column".into(), &JsValue::from(column));
        }
        CodecError::DepthExceeded { limit } => {
            let _ = Reflect::set(&err_obj, &"kind".into(), &"DepthExceeded".into());
            let _ = Reflect::set(&err_obj, &"limit".into(), &JsValue::from(limit));
        }
        CodecError::InputTooLarge { limit, actual } => {
            let _ = Reflect::set(&err_obj, &"kind".into(), &"InputTooLarge".into());
            let _ = Reflect::set(&err_obj, &"limit".into(), &JsValue::from(limit as u32));
            let _ = Reflect::set(&err_obj, &"actual".into(), &JsValue::from(actual as u32));
        }
        CodecError::MalformedHeader => {
            let _ = Reflect::set(&err_obj, &"kind".into(), &"MalformedHeader".into());
        }
        CodecError::DictionaryMismatch => {
            let _ = Reflect::set(&err_obj, &"kind".into(), &"DictionaryMismatch".into());
        }
        CodecError::SchemaArityMismatch { expected, actual } => {
            let _ = Reflect::set(&err_obj, &"kind".into(), &"SchemaArityMismatch".into());
            let _ = Reflect::set(&err_obj, &"expected".into(), &JsValue::from(expected as u32));
            let _ = Reflect::set(&err_obj, &"actual".into(), &JsValue::from(actual as u32));
        }
        CodecError::DictionaryIndexOutOfRange => {
            let _ = Reflect::set(&err_obj, &"kind".into(), &"DictionaryIndexOutOfRange".into());
        }
        CodecError::UnknownAlphabet => {
            let _ = Reflect::set(&err_obj, &"kind".into(), &"UnknownAlphabet".into());
        }
        CodecError::AlphabetMismatch => {
            let _ = Reflect::set(&err_obj, &"kind".into(), &"AlphabetMismatch".into());
        }
        CodecError::UnsupportedMode => {
            let _ = Reflect::set(&err_obj, &"kind".into(), &"UnsupportedMode".into());
        }
    }
    err_obj.into()
}

fn run_catch<T>(f: impl FnOnce() -> Result<T, CodecError>) -> Result<T, JsValue> {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(codec_error_to_js(e)),
        Err(_) => Err(internal_error("internal error")),
    }
}

/// Decode alphabet kind name from header flag bits 5–7.
#[wasm_bindgen]
pub fn alphabet_from_flags(flags: u8) -> Result<String, JsValue> {
    run_catch(|| core_alphabet_from_flags(flags).map(|k| k.as_str().to_string()))
}

/// Compress a JSON text payload with the default options.
#[wasm_bindgen]
pub fn compress(input: &str) -> Result<JsValue, JsValue> {
    let result = run_catch(|| core_compress(input))?;
    result_to_js(result)
}

/// Compress a JSON text payload with explicit options.
#[wasm_bindgen]
pub fn compress_with_options(input: &str, options: JsValue) -> Result<JsValue, JsValue> {
    let options = options_from_js(options)?;
    let result = run_catch(|| core_compress_with_options(input, options))?;
    result_to_js(result)
}

/// Compress an already-parsed JSON value.
#[wasm_bindgen]
pub fn compress_value(value: JsValue) -> Result<JsValue, JsValue> {
    let value: Value =
        serde_wasm_bindgen::from_value(value).map_err(|_| type_error("invalid JSON value"))?;
    let result = run_catch(|| core_compress_value(value))?;
    result_to_js(result)
}

/// Decompress a `(compressed, dictionary)` pair into canonical JSON text.
#[wasm_bindgen]
pub fn decompress(compressed: &str, dictionary: JsValue) -> Result<String, JsValue> {
    let dictionary = dictionary_from_js(dictionary)?;
    run_catch(|| core_decompress(compressed, &dictionary))
}

/// Decompress into a JSON value.
#[wasm_bindgen]
pub fn decompress_to_value(compressed: &str, dictionary: JsValue) -> Result<JsValue, JsValue> {
    let dictionary = dictionary_from_js(dictionary)?;
    let value = run_catch(|| core_decompress_to_value(compressed, &dictionary))?;
    serde_wasm_bindgen::to_value(&value).map_err(|_| internal_error("value encode failed"))
}
