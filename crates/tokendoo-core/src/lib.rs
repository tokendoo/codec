//! Tokendoo core: lossless structural JSON compression.
//!
//! # Wire format
//!
//! A compressed document is the pair `(compressed, dictionary)` returned by
//! [`compress`] and consumed by [`decompress`].
//!
//! ## `compressed` string
//!
//! ```text
//! compressed = "T3" <flags:2 hex digits> ":" <body>
//! ```
//!
//! - `T3` — magic bytes and format version (TOK-007; T2 was bare-default, T1 control).
//! - `flags` — an 8-bit bitmask encoded as two lowercase hex digits:
//!   - `0x01` — key dictionary applied (`dictionary.k` non-empty)
//!   - `0x02` — at least one tabular collapse applied
//!   - `0x04` — value interning applied (`dictionary.v` non-empty)
//!   - `0x08` — identity form: body is the canonical JSON of the input value;
//!     no other transform bit may be set
//!   - `0x10` — encapsulated JSON-string payload
//!   - `0x20|0x40|0x80` — alphabet id bits 5–7: 0=control, 1=printable,
//!     2=bare, 3=common_word, 4=derived_cl100k_v1 (default), 5=derived_o200k_v1,
//!     6=derived_llama_v1; 7 unknown → [`CodecError::UnknownAlphabet`]
//!
//! Derived alphabets also write a short id into `dictionary.a` (`c1` / `o1` / `l1`).
//! Header and dictionary ids must agree or decompress fails with
//! [`CodecError::AlphabetMismatch`].
//!
//! ## Symbol alphabet (default: derived_cl100k_v1)
//!
//! TOK-007 derives symbol spellings from tokenizer vocabularies so each symbol
//! costs exactly one token by construction. The cl100k-derived table beat the
//! TOK-002 bare median (0.387 vs 0.421) and became the default. Vocabularies
//! never enter this crate — only frozen result lists.
//!
//! ## Round-trip guarantee
//!
//! Round-trip is defined over [`serde_json::Value`] equality after parse, not
//! over raw input bytes.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod json;
pub mod tokenizer;

pub use json::alphabet::AlphabetKind;
pub use json::{
    compress, compress_value, compress_with_options, decompress, decompress_to_value, CodecError,
    CompressOptions, CompressResult, Dictionary, MAX_DEPTH, MAX_INPUT_BYTES,
};
pub use tokenizer::{
    EstimatorCoefficients, HeuristicEstimator, TokenEstimator, ESTIMATOR_COEFFICIENTS,
    ESTIMATOR_MODEL, HEURISTIC_BIAS_TOLERANCE, HEURISTIC_COMPRESSED_BIAS,
    HEURISTIC_COMPRESSED_MAX_RELATIVE_ERROR, HEURISTIC_INPUT_BIAS,
    HEURISTIC_INPUT_MAX_RELATIVE_ERROR, HEURISTIC_MAX_RATIO_ERROR, HEURISTIC_RATIO_BIAS,
};
