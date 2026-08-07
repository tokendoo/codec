//! Frozen vocabulary-derived symbol tables (TOK-007).
//!
//! These lists are the **result** of bench-side derivation from a tokenizer
//! vocabulary. The vocabulary itself never enters this crate.

mod cl100k_v1;
mod llama_v1;
mod o200k_v1;

/// Published id written into `dictionary.a` and encoded in header bits 5–7.
pub const CL100K_V1_ID: &str = "c1";
/// Published id for the o200k-derived table.
pub const O200K_V1_ID: &str = "o1";
/// Published id for the Llama-derived table (non-OpenAI family).
pub const LLAMA_V1_ID: &str = "l1";

/// Frozen cl100k_base-derived symbol spellings, shortest-first.
pub use cl100k_v1::SYMBOLS as CL100K_V1;
/// Frozen o200k_base-derived symbol spellings, shortest-first.
pub use o200k_v1::SYMBOLS as O200K_V1;
/// Frozen Llama-tokenizer-derived symbol spellings, shortest-first.
pub use llama_v1::SYMBOLS as LLAMA_V1;
