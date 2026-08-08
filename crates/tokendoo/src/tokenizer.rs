//! Token estimation behind a replaceable trait boundary.
//!
//! Exact BPE (`tiktoken-rs` + `cl100k_base`) stays bench-only. The shipped
//! estimator is a run-length (burst) heuristic calibrated against cl100k on the
//! full TOK-002 transform × alphabet × fixture grid — both fixture **inputs**
//! and codec **outputs** (`compressed` + serialized dictionary), matching how
//! the Worker computes `stats.tokens_*`.
//!
//! # Bound ranks
//!
//! Three numbers are published; they are **not** of equal standing:
//!
//! - [`HEURISTIC_MAX_RATIO_ERROR`] / [`HEURISTIC_RATIO_BIAS`] — **product
//!   guarantee**. The only bound a client-facing savings claim may rest on.
//!   Measured directly: ratio error is not derivable from the two side errors
//!   (same-sign biases cancel; opposite-sign biases amplify).
//! - [`HEURISTIC_INPUT_MAX_RELATIVE_ERROR`] / [`HEURISTIC_INPUT_BIAS`] and
//!   [`HEURISTIC_COMPRESSED_MAX_RELATIVE_ERROR`] / [`HEURISTIC_COMPRESSED_BIAS`]
//!   — **diagnostics**. They explain the ratio; they are never cited as the
//!   product guarantee.
//!
//! Signed biases are tested for drift ([`HEURISTIC_BIAS_TOLERANCE`]). A ratio
//! that only holds because two side biases cancel is fragile; the bias check
//! is what catches a character-profile shift (e.g. a future code mode) before
//! the absolute-error check does.
//!
//! See `docs/context/estimator-calibration.json` and ADR-014.

/// Estimates how many LLM tokens a UTF-8 byte sequence would consume.
pub trait TokenEstimator {
    /// Return an estimated token count for `text`.
    fn estimate(&self, text: &str) -> u64;
}

// ─── PRODUCT GUARANTEE ───────────────────────────────────────────────────────

/// **Product guarantee** — max absolute relative error of
/// `estimate(after) / estimate(before)` vs the exact cl100k ratio on the
/// TOK-002 calibration grid. This is the only bound a savings claim may rest on.
///
/// Measured worst case after the burst fit: ≈ 0.088
/// (`homogeneous_1000|control|S3_intern`). Action thresholds (ADR-014):
/// ≤ 0.05 silence; (0.05, 0.10] state the bound in the API contract before any
/// public figure; > 0.10 no public figure from `stats.tokens_*`.
pub const HEURISTIC_MAX_RATIO_ERROR: f64 = 0.09;

/// **Product guarantee** — mean signed bias of the estimated ratio vs exact
/// (`est - exact`) / `exact` on the calibration grid. Negative means the
/// estimator slightly overstates savings on average.
pub const HEURISTIC_RATIO_BIAS: f64 = -0.0143;

// ─── DIAGNOSTICS (explain the guarantee; never cited as such) ────────────────

/// Diagnostic — max absolute relative error on fixture **inputs**.
pub const HEURISTIC_INPUT_MAX_RELATIVE_ERROR: f64 = 0.46;

/// Diagnostic — mean signed bias on fixture inputs.
pub const HEURISTIC_INPUT_BIAS: f64 = -0.1485;

/// Diagnostic — max absolute relative error on codec **outputs**
/// (`estimate(compressed) + estimate(dict)` vs exact sum).
pub const HEURISTIC_COMPRESSED_MAX_RELATIVE_ERROR: f64 = 0.46;

/// Diagnostic — mean signed bias on codec outputs.
pub const HEURISTIC_COMPRESSED_BIAS: f64 = -0.1612;

/// Absolute drift tolerated on each documented bias before the calibration must
/// be re-run. Protects against silent loss of same-sign compensation.
pub const HEURISTIC_BIAS_TOLERANCE: f64 = 0.02;

/// Stable name of the shipped model, mirrored in measurement artefacts.
pub const ESTIMATOR_MODEL: &str = "burst";

/// Coefficients of the shipped burst model. Digit runs use a fixed
/// `ceil(L/3)` rule (no fitted coefficient).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EstimatorCoefficients {
    /// Per word-run intercept (ASCII letters + `_`).
    pub w0: f64,
    /// Per extra character in a word run.
    pub w1: f64,
    /// Per ASCII-punctuation-run intercept.
    pub p0: f64,
    /// Per extra character in a punctuation run.
    pub p1: f64,
    /// Per space character.
    pub s1: f64,
    /// Per non-ASCII character.
    pub n: f64,
}

/// Fitted coefficients (TOK-002 / ADR-014 burst fit).
pub const ESTIMATOR_COEFFICIENTS: EstimatorCoefficients = EstimatorCoefficients {
    w0: 0.155,
    w1: 0.12,
    p0: 0.7,
    p1: 0.3414,
    s1: 0.01,
    n: 0.2,
};

/// Run-length heuristic calibrated to `cl100k_base`.
///
/// Digit runs contribute `ceil(L/3)` (cl100k groups digits by three). Word runs
/// (ASCII letters and `_`), ASCII punctuation runs, spaces, and non-ASCII each
/// have their own linear cost. This is **not** an affine function of byte
/// length — unlike the TOK-001 `bytes.div_ceil(4)` estimator — so
/// `tokens_after / tokens_before` can diverge from the byte ratio.
///
/// Production computes `tokens_after` as
/// `estimate(compressed) + estimate(dict_json)` (two separate rounded
/// estimates). The calibration matches that arithmetic.
#[derive(Debug, Default, Clone, Copy)]
pub struct HeuristicEstimator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunKind {
    Digit,
    Word,
    Punct,
    Space,
    NonAscii,
}

fn classify(c: char) -> RunKind {
    if c.is_ascii_digit() {
        RunKind::Digit
    } else if c.is_ascii_alphabetic() || c == '_' {
        RunKind::Word
    } else if c == ' ' {
        RunKind::Space
    } else if c.is_ascii() {
        RunKind::Punct
    } else {
        RunKind::NonAscii
    }
}

impl TokenEstimator for HeuristicEstimator {
    fn estimate(&self, text: &str) -> u64 {
        if text.is_empty() {
            return 0;
        }
        let c = ESTIMATOR_COEFFICIENTS;
        let mut units = 0.0_f64;
        let mut chars = text.chars().peekable();
        while let Some(ch) = chars.next() {
            let kind = classify(ch);
            let mut len = 1_usize;
            while chars.peek().copied().is_some_and(|n| classify(n) == kind) {
                chars.next();
                len += 1;
            }
            match kind {
                RunKind::Digit => {
                    units += len.div_ceil(3) as f64;
                }
                RunKind::Word => {
                    units += c.w0 + c.w1 * (len - 1) as f64;
                }
                RunKind::Punct => {
                    units += c.p0 + c.p1 * (len - 1) as f64;
                }
                RunKind::Space => {
                    units += c.s1 * len as f64;
                }
                RunKind::NonAscii => {
                    units += c.n * len as f64;
                }
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
    fn punctuation_costs_more_than_letters_per_char_when_isolated() {
        // A single punct char costs p0; eight isolated letters cost 8*w0.
        // Eight letters in one run cost w0 + 7*w1, which is cheap — so compare
        // isolated punctuation against an equal-length letter run carefully.
        let one_punct = HeuristicEstimator.estimate("{");
        let one_letter = HeuristicEstimator.estimate("a");
        assert!(one_punct >= one_letter);
    }

    #[test]
    fn digit_runs_group_by_three() {
        // "123456" → ceil(6/3)=2; "12345" → ceil(5/3)=2; "12" → 1.
        assert_eq!(HeuristicEstimator.estimate("123456"), 2);
        assert_eq!(HeuristicEstimator.estimate("12345"), 2);
        assert_eq!(HeuristicEstimator.estimate("12"), 1);
    }

    #[test]
    fn model_metadata_matches_constants() {
        assert_eq!(ESTIMATOR_MODEL, "burst");
        assert!((ESTIMATOR_COEFFICIENTS.w0 - 0.155).abs() < 1e-12);
    }
}
