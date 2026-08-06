//! Token estimation behind a replaceable trait boundary.
//!
//! Exact BPE (`tiktoken-rs` + `cl100k_base`) stays bench-only. The shipped
//! estimator is a character-class heuristic calibrated against cl100k on the
//! TOK-002 fixture corpus (see `docs/context/tok-002-measurements.md`).

/// Estimates how many LLM tokens a UTF-8 byte sequence would consume.
pub trait TokenEstimator {
    /// Return an estimated token count for `text`.
    fn estimate(&self, text: &str) -> u64;
}

/// Documented max relative error of [`HeuristicEstimator`] vs cl100k_base on the
/// TOK-002 fixture corpus (inputs). Shipping structural fixtures stay well
/// under this; the worst case in the corpus is prose-heavy (~0.12).
pub const HEURISTIC_MAX_RELATIVE_ERROR: f64 = 0.15;

/// Character-class heuristic calibrated to `cl100k_base`.
///
/// Coefficients (TOK-002 fit): alphanumeric / `_` / space contribute `0.18`
/// token-units each; other ASCII (JSON punctuation) contribute `0.73`;
/// non-ASCII contributes `1.0`. This is **not** an affine function of byte
/// length — unlike the TOK-001 `bytes.div_ceil(4)` estimator — so
/// `tokens_after / tokens_before` can diverge from the byte ratio.
#[derive(Debug, Default, Clone, Copy)]
pub struct HeuristicEstimator;

impl TokenEstimator for HeuristicEstimator {
    fn estimate(&self, text: &str) -> u64 {
        if text.is_empty() {
            return 0;
        }
        let mut units = 0.0_f64;
        for c in text.chars() {
            if c.is_ascii_alphanumeric() || c == '_' || c == ' ' {
                units += 0.18;
            } else if c.is_ascii() {
                units += 0.73;
            } else {
                units += 1.0;
            }
        }
        std::cmp::max(1, units.round() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_zero() {
        assert_eq!(HeuristicEstimator.estimate(""), 0);
    }

    #[test]
    fn short_is_at_least_one() {
        assert_eq!(HeuristicEstimator.estimate("a"), 1);
    }

    #[test]
    fn punctuation_costs_more_than_letters() {
        let letters = HeuristicEstimator.estimate("aaaaaaaa");
        let punct = HeuristicEstimator.estimate("{}{}{}{}");
        assert!(punct > letters);
    }
}
