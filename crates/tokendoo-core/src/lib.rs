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
//! compressed = "T1" <flags:2 hex digits> ":" <body>
//! ```
//!
//! - `T1` — magic bytes and format version.
//! - `flags` — an 8-bit bitmask encoded as two lowercase hex digits:
//!   - `0x01` — key dictionary applied (`dictionary.k` non-empty)
//!   - `0x02` — at least one tabular collapse applied
//!   - `0x04` — value interning applied (`dictionary.v` non-empty)
//!   - `0x08` — identity form: body is the canonical JSON of the input value;
//!     no other transform bit may be set
//!   - `0x10` — the original request `payload` was a JSON string whose content
//!     is itself JSON text (encapsulated form); decompress returns that string
//!     value rather than the parsed inner document
//!   - bits `0x20`–`0x80` — reserved; must be zero
//! - `:` — separator; never the first character of a JSON value
//! - `body` — compact JSON serialization of the (possibly transformed) tree
//!
//! ## Symbol alphabet
//!
//! Transform symbols are prefixed with a control character so they cannot be
//! confused with ordinary keys or values:
//!
//! - key dictionary symbol: `U+0001` followed by base-36 ordinal
//! - tabular sentinel: `U+0002` followed by `'T'`
//! - value intern symbol: `U+0003` followed by base-36 ordinal
//!
//! If a document already contains a string that would collide with a sentinel,
//! the corresponding transform is skipped.
//!
//! ## Round-trip guarantee
//!
//! Round-trip is defined over [`serde_json::Value`] equality after parse, not
//! over raw input bytes. Canonicalization (whitespace stripping, key sorting,
//! number normalization) is intentionally not byte-invertible.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod json;
pub mod tokenizer;

pub use json::{
    compress, compress_value, compress_with_options, decompress, decompress_to_value, CodecError,
    CompressOptions, CompressResult, Dictionary, MAX_DEPTH, MAX_INPUT_BYTES,
};
pub use tokenizer::{HeuristicEstimator, TokenEstimator};
