//! Python bindings for the Tokendoo codec.
//!
//! This crate is a build input for the PyPI package `tokendoo`. It is not
//! published to crates.io (`publish = false`).

use std::panic::{catch_unwind, AssertUnwindSafe};

use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyRuntimeError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyAnyMethods, PyModule, PyModuleMethods};
use serde_json::Value;
use ::tokendoo::json::header::{
    alphabet_from_flags as core_alphabet_from_flags, FLAG_ALPHABET_MASK as CORE_FLAG_ALPHABET_MASK,
    FLAG_IDENTITY as CORE_FLAG_IDENTITY, FLAG_KEYS as CORE_FLAG_KEYS,
    FLAG_TABULAR as CORE_FLAG_TABULAR, FLAG_VALUES as CORE_FLAG_VALUES,
    FLAG_WRAPPED_STRING as CORE_FLAG_WRAPPED_STRING,
};
use ::tokendoo::{
    compress as core_compress, compress_value as core_compress_value,
    compress_with_options as core_compress_with_options, decompress as core_decompress,
    decompress_to_value as core_decompress_to_value, AlphabetKind, CodecError as CoreCodecError,
    CompressOptions, CompressResult as CoreCompressResult, Dictionary as CoreDictionary,
    MAX_DEPTH as CORE_MAX_DEPTH, MAX_INPUT_BYTES as CORE_MAX_INPUT_BYTES,
};

create_exception!(tokendoo, CodecError, PyException);
create_exception!(tokendoo, ParseFailure, CodecError);
create_exception!(tokendoo, DepthExceeded, CodecError);
create_exception!(tokendoo, InputTooLarge, CodecError);
create_exception!(tokendoo, MalformedHeader, CodecError);
create_exception!(tokendoo, DictionaryMismatch, CodecError);
create_exception!(tokendoo, SchemaArityMismatch, CodecError);
create_exception!(tokendoo, DictionaryIndexOutOfRange, CodecError);
create_exception!(tokendoo, UnknownAlphabet, CodecError);
create_exception!(tokendoo, AlphabetMismatch, CodecError);
create_exception!(tokendoo, UnsupportedMode, CodecError);

/// Symbol tables shipped alongside a compressed body.
#[pyclass(name = "Dictionary")]
#[derive(Clone, Debug, Default)]
struct Dictionary {
    /// Key dictionary: index `i` maps to symbol ordinal `i`.
    #[pyo3(get, set)]
    k: Vec<String>,
    /// Value dictionary: index `i` maps to intern ordinal `i`.
    #[pyo3(get, set)]
    v: Vec<String>,
    /// Short alphabet id for vocabulary-derived tables.
    #[pyo3(get, set)]
    a: Option<String>,
}

#[pymethods]
impl Dictionary {
    #[new]
    #[pyo3(signature = (k=None, v=None, a=None))]
    fn new(k: Option<Vec<String>>, v: Option<Vec<String>>, a: Option<String>) -> Self {
        Self {
            k: k.unwrap_or_default(),
            v: v.unwrap_or_default(),
            a,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Dictionary(k_len={}, v_len={}, a={:?})",
            self.k.len(),
            self.v.len(),
            self.a
        )
    }
}

impl From<CoreDictionary> for Dictionary {
    fn from(value: CoreDictionary) -> Self {
        Self {
            k: value.k,
            v: value.v,
            a: value.a,
        }
    }
}

impl From<&Dictionary> for CoreDictionary {
    fn from(value: &Dictionary) -> Self {
        Self {
            k: value.k.clone(),
            v: value.v.clone(),
            a: value.a.clone(),
        }
    }
}

/// Successful compression output.
#[pyclass(name = "CompressResult")]
#[derive(Clone, Debug)]
struct CompressResult {
    /// Header-prefixed compact body.
    #[pyo3(get)]
    compressed: String,
    /// Accompanying symbol tables.
    #[pyo3(get)]
    dictionary: Dictionary,
    /// Byte length of the input as received.
    #[pyo3(get)]
    bytes_before: usize,
    /// Byte length of `compressed` plus the serialized dictionary.
    #[pyo3(get)]
    bytes_after: usize,
    /// Whether the identity fallback was chosen.
    #[pyo3(get)]
    identity: bool,
    /// Header flags actually written.
    #[pyo3(get)]
    flags: u8,
}

#[pymethods]
impl CompressResult {
    fn __repr__(&self) -> String {
        format!(
            "CompressResult(bytes_before={}, bytes_after={}, identity={}, flags={})",
            self.bytes_before, self.bytes_after, self.identity, self.flags
        )
    }
}

fn parse_alphabet(name: &str) -> Result<AlphabetKind, CoreCodecError> {
    match name {
        "control" => Ok(AlphabetKind::Control),
        "printable" => Ok(AlphabetKind::Printable),
        "bare" => Ok(AlphabetKind::Bare),
        "common_word" => Ok(AlphabetKind::CommonWord),
        "derived_cl100k_v1" => Ok(AlphabetKind::DerivedCl100kV1),
        "derived_o200k_v1" => Ok(AlphabetKind::DerivedO200kV1),
        "derived_llama_v1" => Ok(AlphabetKind::DerivedLlamaV1),
        _ => Err(CoreCodecError::UnsupportedMode),
    }
}

fn options_from_args(
    key_dictionary: Option<bool>,
    tabular: Option<bool>,
    intern_values: Option<bool>,
    alphabet: Option<&str>,
) -> Result<CompressOptions, CoreCodecError> {
    let mut options = CompressOptions::default();
    if let Some(v) = key_dictionary {
        options.key_dictionary = v;
    }
    if let Some(v) = tabular {
        options.tabular = v;
    }
    if let Some(v) = intern_values {
        options.intern_values = v;
    }
    if let Some(name) = alphabet {
        options.alphabet = parse_alphabet(name)?;
    }
    Ok(options)
}

fn codec_to_pyerr(py: Python<'_>, err: CoreCodecError) -> PyErr {
    let message = err.to_string();
    let py_err = match err {
        CoreCodecError::ParseFailure { line, column } => {
            let e = ParseFailure::new_err(message);
            let _ = e.value(py).setattr("line", line);
            let _ = e.value(py).setattr("column", column);
            let _ = e.value(py).setattr("kind", "ParseFailure");
            e
        }
        CoreCodecError::DepthExceeded { limit } => {
            let e = DepthExceeded::new_err(message);
            let _ = e.value(py).setattr("limit", limit);
            let _ = e.value(py).setattr("kind", "DepthExceeded");
            e
        }
        CoreCodecError::InputTooLarge { limit, actual } => {
            let e = InputTooLarge::new_err(message);
            let _ = e.value(py).setattr("limit", limit);
            let _ = e.value(py).setattr("actual", actual);
            let _ = e.value(py).setattr("kind", "InputTooLarge");
            e
        }
        CoreCodecError::MalformedHeader => {
            let e = MalformedHeader::new_err(message);
            let _ = e.value(py).setattr("kind", "MalformedHeader");
            e
        }
        CoreCodecError::DictionaryMismatch => {
            let e = DictionaryMismatch::new_err(message);
            let _ = e.value(py).setattr("kind", "DictionaryMismatch");
            e
        }
        CoreCodecError::SchemaArityMismatch { expected, actual } => {
            let e = SchemaArityMismatch::new_err(message);
            let _ = e.value(py).setattr("expected", expected);
            let _ = e.value(py).setattr("actual", actual);
            let _ = e.value(py).setattr("kind", "SchemaArityMismatch");
            e
        }
        CoreCodecError::DictionaryIndexOutOfRange => {
            let e = DictionaryIndexOutOfRange::new_err(message);
            let _ = e.value(py).setattr("kind", "DictionaryIndexOutOfRange");
            e
        }
        CoreCodecError::UnknownAlphabet => {
            let e = UnknownAlphabet::new_err(message);
            let _ = e.value(py).setattr("kind", "UnknownAlphabet");
            e
        }
        CoreCodecError::AlphabetMismatch => {
            let e = AlphabetMismatch::new_err(message);
            let _ = e.value(py).setattr("kind", "AlphabetMismatch");
            e
        }
        CoreCodecError::UnsupportedMode => {
            let e = UnsupportedMode::new_err(message);
            let _ = e.value(py).setattr("kind", "UnsupportedMode");
            e
        }
    };
    py_err
}

fn run_catch<T>(py: Python<'_>, f: impl FnOnce() -> Result<T, CoreCodecError>) -> PyResult<T> {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(codec_to_pyerr(py, e)),
        Err(_) => Err(PyRuntimeError::new_err("internal error")),
    }
}

fn to_py_result(result: CoreCompressResult) -> CompressResult {
    CompressResult {
        compressed: result.compressed,
        dictionary: Dictionary::from(result.dictionary),
        bytes_before: result.bytes_before,
        bytes_after: result.bytes_after,
        identity: result.identity,
        flags: result.flags,
    }
}

/// Decode alphabet kind name from header flag bits 5–7.
#[pyfunction]
fn alphabet_from_flags(py: Python<'_>, flags: u8) -> PyResult<String> {
    run_catch(py, || core_alphabet_from_flags(flags).map(|k| k.as_str().to_string()))
}

/// Compress a JSON text payload with the default options.
#[pyfunction]
fn compress(py: Python<'_>, input: &str) -> PyResult<CompressResult> {
    run_catch(py, || core_compress(input)).map(to_py_result)
}

/// Compress a JSON text payload with explicit options.
#[pyfunction]
#[pyo3(signature = (input, *, key_dictionary=None, tabular=None, intern_values=None, alphabet=None))]
fn compress_with_options(
    py: Python<'_>,
    input: &str,
    key_dictionary: Option<bool>,
    tabular: Option<bool>,
    intern_values: Option<bool>,
    alphabet: Option<&str>,
) -> PyResult<CompressResult> {
    let options = options_from_args(key_dictionary, tabular, intern_values, alphabet)
        .map_err(|e| codec_to_pyerr(py, e))?;
    run_catch(py, || core_compress_with_options(input, options)).map(to_py_result)
}

/// Compress an already-parsed JSON value.
#[pyfunction]
fn compress_value(py: Python<'_>, value: Bound<'_, PyAny>) -> PyResult<CompressResult> {
    let json = py.import("json")?;
    let text: String = json
        .call_method1("dumps", (value,))?
        .extract()
        .map_err(|_| PyTypeError::new_err("invalid JSON value"))?;
    let parsed: Value = serde_json::from_str(&text)
        .map_err(|_| PyTypeError::new_err("invalid JSON value"))?;
    run_catch(py, || core_compress_value(parsed)).map(to_py_result)
}

/// Decompress a `(compressed, dictionary)` pair into canonical JSON text.
#[pyfunction]
fn decompress(py: Python<'_>, compressed: &str, dictionary: PyRef<'_, Dictionary>) -> PyResult<String> {
    let dict = CoreDictionary::from(&*dictionary);
    run_catch(py, || core_decompress(compressed, &dict))
}

/// Decompress into a JSON value (Python object).
#[pyfunction]
fn decompress_to_value(
    py: Python<'_>,
    compressed: &str,
    dictionary: PyRef<'_, Dictionary>,
) -> PyResult<Py<PyAny>> {
    let dict = CoreDictionary::from(&*dictionary);
    let value = run_catch(py, || core_decompress_to_value(compressed, &dict))?;
    let text =
        serde_json::to_string(&value).map_err(|_| PyRuntimeError::new_err("value encode failed"))?;
    let json = py.import("json")?;
    Ok(json.call_method1("loads", (text,))?.unbind())
}

/// Python module `tokendoo`.
#[pymodule]
#[pyo3(name = "tokendoo")]
fn tokendoo_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Dictionary>()?;
    m.add_class::<CompressResult>()?;
    m.add("MAX_INPUT_BYTES", CORE_MAX_INPUT_BYTES)?;
    m.add("MAX_DEPTH", CORE_MAX_DEPTH)?;
    m.add("FLAG_KEYS", CORE_FLAG_KEYS)?;
    m.add("FLAG_TABULAR", CORE_FLAG_TABULAR)?;
    m.add("FLAG_VALUES", CORE_FLAG_VALUES)?;
    m.add("FLAG_IDENTITY", CORE_FLAG_IDENTITY)?;
    m.add("FLAG_WRAPPED_STRING", CORE_FLAG_WRAPPED_STRING)?;
    m.add("FLAG_ALPHABET_MASK", CORE_FLAG_ALPHABET_MASK)?;
    m.add("CodecError", m.py().get_type::<CodecError>())?;
    m.add("ParseFailure", m.py().get_type::<ParseFailure>())?;
    m.add("DepthExceeded", m.py().get_type::<DepthExceeded>())?;
    m.add("InputTooLarge", m.py().get_type::<InputTooLarge>())?;
    m.add("MalformedHeader", m.py().get_type::<MalformedHeader>())?;
    m.add("DictionaryMismatch", m.py().get_type::<DictionaryMismatch>())?;
    m.add(
        "SchemaArityMismatch",
        m.py().get_type::<SchemaArityMismatch>(),
    )?;
    m.add(
        "DictionaryIndexOutOfRange",
        m.py().get_type::<DictionaryIndexOutOfRange>(),
    )?;
    m.add("UnknownAlphabet", m.py().get_type::<UnknownAlphabet>())?;
    m.add("AlphabetMismatch", m.py().get_type::<AlphabetMismatch>())?;
    m.add("UnsupportedMode", m.py().get_type::<UnsupportedMode>())?;
    m.add_function(wrap_pyfunction!(alphabet_from_flags, m)?)?;
    m.add_function(wrap_pyfunction!(compress, m)?)?;
    m.add_function(wrap_pyfunction!(compress_with_options, m)?)?;
    m.add_function(wrap_pyfunction!(compress_value, m)?)?;
    m.add_function(wrap_pyfunction!(decompress, m)?)?;
    m.add_function(wrap_pyfunction!(decompress_to_value, m)?)?;
    Ok(())
}
