//! Tokendoo benchmark library: synthetic fixtures and measurement helpers.
//!
//! - [`fixtures`] — the synthetic corpus. No network, no environment, no PII.
//! - [`derive`] — vocabulary-derived alphabet construction (TOK-007). Bench-only.

#![forbid(unsafe_code)]

/// Vocabulary-derived alphabet construction (TOK-007). Bench-only.
pub mod derive;

pub mod fixtures;
