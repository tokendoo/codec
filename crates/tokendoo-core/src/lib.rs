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
//! compressed = "T2" <flags:2 hex digits> ":" <body>
//! ```
//!
//! - `T2` — magic bytes and format version (TOK-002; T1 used the control alphabet).
//! - `flags` — an 8-bit bitmask encoded as two lowercase hex digits:
//!   - `0x01` — key dictionary applied (`dictionary.k` non-empty)
//!   - `0x02` — at least one tabular collapse applied
//!   - `0x04` — value interning applied (`dictionary.v` non-empty)
//!   - `0x08` — identity form: body is the canonical JSON of the input value;
//!     no other transform bit may be set
//!   - `0x10` — encapsulated JSON-string payload
//!   - `0x20|0x40` — alphabet id: 0=control, 1=printable, 2=bare (default), 3=common_word
//!   - `0x80` — reserved; must be zero
//!
//! ## Symbol alphabet (default: bare)
//!
//! TOK-002 measured four alphabets against `cl100k_base`. The **bare** alphabet
//! (base36 ordinals, tabular sentinel `__T`) minimised median token ratio on
//! shipping fixtures. The TOK-001 control alphabet shrinks bytes while often
//! leaving tokens flat or worse on small/realistic payloads.
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
pub use tokenizer::{HeuristicEstimator, TokenEstimator, HEURISTIC_MAX_RELATIVE_ERROR};
