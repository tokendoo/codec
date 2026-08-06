//! Token estimation behind a replaceable trait boundary.
//!
//! The MVP ships a fast heuristic estimator. Exact BPE (`tiktoken-rs`) is
//! deferred to TOK-002 pending a Workers bundle-size study.

/// Estimates how many LLM tokens a UTF-8 byte sequence would consume.
pub trait TokenEstimator {
    /// Return an estimated token count for `text`.
    fn estimate(&self, text: &str) -> u64;
}

/// Heuristic estimator: roughly one token per ~4 UTF-8 bytes, floored at 1
/// for non-empty input. This deliberately overestimates slightly on dense
/// ASCII to avoid under-reporting savings.
#[derive(Debug, Default, Clone, Copy)]
pub struct HeuristicEstimator;

impl TokenEstimator for HeuristicEstimator {
    fn estimate(&self, text: &str) -> u64 {
        if text.is_empty() {
            return 0;
        }
        let bytes = text.len() as u64;
        std::cmp::max(1, bytes.div_ceil(4))
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
    fn scales_with_bytes() {
        assert_eq!(HeuristicEstimator.estimate("abcd"), 1);
        assert_eq!(HeuristicEstimator.estimate("abcdefgh"), 2);
    }
}
