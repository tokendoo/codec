//! Tokendoo benchmark library: synthetic fixtures and measurement harnesses.
//!
//! - [`fixtures`] — the synthetic corpus. No network, no environment, no PII.
//! - [`eval`] — the TOK-003 comprehension harness. Reaches the network only
//!   from its own binary, and only with fixtures from [`fixtures`].

#![forbid(unsafe_code)]

/// The comprehension harness documents every public item; the corpus module
/// predates that convention and is left untouched.
#[deny(missing_docs)]
pub mod eval;

pub mod fixtures;
