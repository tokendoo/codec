//! TOK-002 estimator calibration: fit the shipped HeuristicEstimator against
//! exact `cl100k_base` counts on the full transform × alphabet × fixture grid.
//!
//! Usage: `cargo run -p tokendoo-bench --release --bin tokendoo-calibrate`
//!
//! Writes `docs/context/estimator-calibration.json`.
//!
//! The production Worker computes
//! `tokens_after = estimate(compressed) + estimate(dict_json)` — two separate
//! rounded estimates. This binary fits that quantity, not the estimate of the
//! concatenation. The objective is the worst-case absolute relative error on
//! the **ratio**; per-side errors are diagnostics.

use std::fs;
use std::path::PathBuf;

use serde::Serialize;
use sha2::{Digest, Sha256};
use tiktoken_rs::cl100k_base;
use tokendoo_bench::fixtures::build_corpus;
use tokendoo::{
    compress_with_options, AlphabetKind, CompressOptions, HEURISTIC_COMPRESSED_MAX_RELATIVE_ERROR,
    HEURISTIC_INPUT_MAX_RELATIVE_ERROR, HEURISTIC_MAX_RATIO_ERROR,
};

#[derive(Debug, Clone, Copy)]
struct Subset {
    name: &'static str,
    key_dictionary: bool,
    tabular: bool,
    intern_values: bool,
}

const SUBSETS: &[Subset] = &[
    Subset {
        name: "S0_none",
        key_dictionary: false,
        tabular: false,
        intern_values: false,
    },
    Subset {
        name: "S1_keys",
        key_dictionary: true,
        tabular: false,
        intern_values: false,
    },
    Subset {
        name: "S2_tabular",
        key_dictionary: false,
        tabular: true,
        intern_values: false,
    },
    Subset {
        name: "S3_intern",
        key_dictionary: false,
        tabular: false,
        intern_values: true,
    },
    Subset {
        name: "S4_keys_tabular",
        key_dictionary: true,
        tabular: true,
        intern_values: false,
    },
    Subset {
        name: "S5_keys_intern",
        key_dictionary: true,
        tabular: false,
        intern_values: true,
    },
    Subset {
        name: "S6_tabular_intern",
        key_dictionary: false,
        tabular: true,
        intern_values: true,
    },
    Subset {
        name: "S7_full",
        key_dictionary: true,
        tabular: true,
        intern_values: true,
    },
];

/// Flat character-class features matching the TOK-002 estimator shape.
#[derive(Debug, Clone, Copy, Default)]
struct FlatFeatures {
    alnum_space: f64,
    other_ascii: f64,
    non_ascii: f64,
}

impl FlatFeatures {
    fn extract(text: &str) -> Self {
        let mut f = Self::default();
        for c in text.chars() {
            if c.is_ascii_alphanumeric() || c == '_' || c == ' ' {
                f.alnum_space += 1.0;
            } else if c.is_ascii() {
                f.other_ascii += 1.0;
            } else {
                f.non_ascii += 1.0;
            }
        }
        f
    }

    fn estimate(self, coeffs: &[f64; 3]) -> u64 {
        if self.alnum_space + self.other_ascii + self.non_ascii == 0.0 {
            return 0;
        }
        let units = coeffs[0] * self.alnum_space
            + coeffs[1] * self.other_ascii
            + coeffs[2] * self.non_ascii;
        std::cmp::max(1, units.round() as u64)
    }
}

/// Burst / run-length features. Digit runs contribute a fixed `ceil(L/3)`.
#[derive(Debug, Clone, Copy, Default)]
struct BurstFeatures {
    /// Fixed digit contribution: sum of ceil(len/3) over digit runs.
    digit_units: f64,
    /// Number of word runs (ASCII letters + `_`).
    word_runs: f64,
    /// Sum of (L-1) over word runs.
    word_extra: f64,
    /// Number of ASCII-punctuation runs.
    punct_runs: f64,
    /// Sum of (L-1) over punctuation runs.
    punct_extra: f64,
    /// Total space characters.
    spaces: f64,
    /// Total non-ASCII characters.
    non_ascii: f64,
}

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

impl BurstFeatures {
    fn extract(text: &str) -> Self {
        let mut f = Self::default();
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            let kind = classify(c);
            let mut len = 1_usize;
            while chars.peek().copied().is_some_and(|n| classify(n) == kind) {
                chars.next();
                len += 1;
            }
            match kind {
                RunKind::Digit => {
                    f.digit_units += len.div_ceil(3) as f64;
                }
                RunKind::Word => {
                    f.word_runs += 1.0;
                    f.word_extra += (len - 1) as f64;
                }
                RunKind::Punct => {
                    f.punct_runs += 1.0;
                    f.punct_extra += (len - 1) as f64;
                }
                RunKind::Space => f.spaces += len as f64,
                RunKind::NonAscii => f.non_ascii += len as f64,
            }
        }
        f
    }

    /// Coefficients: [w0, w1, p0, p1, s1, n].
    fn estimate(self, coeffs: &[f64; 6]) -> u64 {
        if self.digit_units == 0.0
            && self.word_runs == 0.0
            && self.punct_runs == 0.0
            && self.spaces == 0.0
            && self.non_ascii == 0.0
        {
            return 0;
        }
        let units = self.digit_units
            + coeffs[0] * self.word_runs
            + coeffs[1] * self.word_extra
            + coeffs[2] * self.punct_runs
            + coeffs[3] * self.punct_extra
            + coeffs[4] * self.spaces
            + coeffs[5] * self.non_ascii;
        std::cmp::max(1, units.round() as u64)
    }
}

#[derive(Debug, Clone)]
struct Sample {
    fixture: String,
    alphabet: String,
    subset: String,
    input: FlatFeatures,
    compressed: FlatFeatures,
    dict: FlatFeatures,
    input_burst: BurstFeatures,
    compressed_burst: BurstFeatures,
    dict_burst: BurstFeatures,
    tokens_before: u64,
    tokens_after: u64,
    compressed_sha256: String,
    dict_sha256: String,
}

#[derive(Debug, Clone, Serialize)]
struct SideSummary {
    worst_signed_rel_error: f64,
    worst_abs_rel_error: f64,
    mean_signed_bias: f64,
    worst_cell: String,
}

#[derive(Debug, Clone, Serialize)]
struct CellRecord {
    fixture: String,
    alphabet: String,
    subset: String,
    tokens_before_exact: u64,
    tokens_after_exact: u64,
    tokens_before_est: u64,
    tokens_after_est: u64,
    ratio_exact: f64,
    ratio_est: f64,
    compressed_sha256: String,
    dict_sha256: String,
}

#[derive(Debug, Clone, Serialize)]
struct CalibrationReport {
    model: String,
    coefficients: serde_json::Value,
    summaries: Summaries,
    cells: Vec<CellRecord>,
    decision: String,
}

#[derive(Debug, Clone, Serialize)]
struct Summaries {
    input: SideSummary,
    compressed: SideSummary,
    ratio: SideSummary,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn count_tokens(text: &str) -> u64 {
    let bpe = cl100k_base().expect("cl100k_base");
    bpe.encode_with_special_tokens(text).len() as u64
}

fn collect_samples() -> Vec<Sample> {
    let corpus = build_corpus();
    let mut samples = Vec::with_capacity(corpus.len() * AlphabetKind::ALL.len() * SUBSETS.len());
    eprint!(
        "Collecting {}×{}×{} cells",
        corpus.len(),
        AlphabetKind::ALL.len(),
        SUBSETS.len()
    );
    for fixture in &corpus {
        for alphabet in AlphabetKind::ALL {
            for subset in SUBSETS {
                eprint!(".");
                let opts = CompressOptions {
                    key_dictionary: subset.key_dictionary,
                    tabular: subset.tabular,
                    intern_values: subset.intern_values,
                    alphabet,
                };
                let result = compress_with_options(&fixture.json, opts).expect("compress");
                let dict_json = serde_json::to_string(&result.dictionary).expect("dict");
                let tokens_before = count_tokens(&fixture.json);
                let tokens_after = count_tokens(&result.compressed) + count_tokens(&dict_json);
                samples.push(Sample {
                    fixture: fixture.name.clone(),
                    alphabet: alphabet.as_str().to_string(),
                    subset: subset.name.to_string(),
                    input: FlatFeatures::extract(&fixture.json),
                    compressed: FlatFeatures::extract(&result.compressed),
                    dict: FlatFeatures::extract(&dict_json),
                    input_burst: BurstFeatures::extract(&fixture.json),
                    compressed_burst: BurstFeatures::extract(&result.compressed),
                    dict_burst: BurstFeatures::extract(&dict_json),
                    tokens_before,
                    tokens_after,
                    compressed_sha256: sha256_hex(result.compressed.as_bytes()),
                    dict_sha256: sha256_hex(dict_json.as_bytes()),
                });
            }
        }
    }
    eprintln!();
    samples
}

fn rel_signed(est: f64, exact: f64) -> f64 {
    if exact == 0.0 {
        0.0
    } else {
        (est - exact) / exact
    }
}

fn evaluate_flat(samples: &[Sample], coeffs: &[f64; 3]) -> (Summaries, Vec<CellRecord>, f64) {
    let mut input_signed = Vec::new();
    let mut compressed_signed = Vec::new();
    let mut ratio_signed = Vec::new();
    let mut cells = Vec::with_capacity(samples.len());

    let mut worst_input = (0.0_f64, String::new());
    let mut worst_comp = (0.0_f64, String::new());
    let mut worst_ratio = (0.0_f64, String::new());

    for s in samples {
        let before_est = s.input.estimate(coeffs);
        let after_est = s.compressed.estimate(coeffs) + s.dict.estimate(coeffs);
        let key = format!("{}|{}|{}", s.fixture, s.alphabet, s.subset);

        let in_err = rel_signed(before_est as f64, s.tokens_before as f64);
        input_signed.push(in_err);
        if in_err.abs() > worst_input.0.abs() {
            worst_input = (in_err, key.clone());
        }

        let out_err = rel_signed(after_est as f64, s.tokens_after as f64);
        compressed_signed.push(out_err);
        if out_err.abs() > worst_comp.0.abs() {
            worst_comp = (out_err, key.clone());
        }

        let ratio_exact = if s.tokens_before == 0 {
            1.0
        } else {
            s.tokens_after as f64 / s.tokens_before as f64
        };
        let ratio_est = if before_est == 0 {
            1.0
        } else {
            after_est as f64 / before_est as f64
        };
        let r_err = rel_signed(ratio_est, ratio_exact);
        ratio_signed.push(r_err);
        if r_err.abs() > worst_ratio.0.abs() {
            worst_ratio = (r_err, key);
        }

        cells.push(CellRecord {
            fixture: s.fixture.clone(),
            alphabet: s.alphabet.clone(),
            subset: s.subset.clone(),
            tokens_before_exact: s.tokens_before,
            tokens_after_exact: s.tokens_after,
            tokens_before_est: before_est,
            tokens_after_est: after_est,
            ratio_exact,
            ratio_est,
            compressed_sha256: s.compressed_sha256.clone(),
            dict_sha256: s.dict_sha256.clone(),
        });
    }

    let summaries = Summaries {
        input: side_summary(&input_signed, &worst_input),
        compressed: side_summary(&compressed_signed, &worst_comp),
        ratio: side_summary(&ratio_signed, &worst_ratio),
    };
    let objective = summaries.ratio.worst_abs_rel_error;
    (summaries, cells, objective)
}

fn evaluate_burst(samples: &[Sample], coeffs: &[f64; 6]) -> (Summaries, Vec<CellRecord>, f64) {
    let mut input_signed = Vec::new();
    let mut compressed_signed = Vec::new();
    let mut ratio_signed = Vec::new();
    let mut cells = Vec::with_capacity(samples.len());

    let mut worst_input = (0.0_f64, String::new());
    let mut worst_comp = (0.0_f64, String::new());
    let mut worst_ratio = (0.0_f64, String::new());

    for s in samples {
        let before_est = s.input_burst.estimate(coeffs);
        let after_est = s.compressed_burst.estimate(coeffs) + s.dict_burst.estimate(coeffs);
        let key = format!("{}|{}|{}", s.fixture, s.alphabet, s.subset);

        let in_err = rel_signed(before_est as f64, s.tokens_before as f64);
        input_signed.push(in_err);
        if in_err.abs() > worst_input.0.abs() {
            worst_input = (in_err, key.clone());
        }

        let out_err = rel_signed(after_est as f64, s.tokens_after as f64);
        compressed_signed.push(out_err);
        if out_err.abs() > worst_comp.0.abs() {
            worst_comp = (out_err, key.clone());
        }

        let ratio_exact = if s.tokens_before == 0 {
            1.0
        } else {
            s.tokens_after as f64 / s.tokens_before as f64
        };
        let ratio_est = if before_est == 0 {
            1.0
        } else {
            after_est as f64 / before_est as f64
        };
        let r_err = rel_signed(ratio_est, ratio_exact);
        ratio_signed.push(r_err);
        if r_err.abs() > worst_ratio.0.abs() {
            worst_ratio = (r_err, key);
        }

        cells.push(CellRecord {
            fixture: s.fixture.clone(),
            alphabet: s.alphabet.clone(),
            subset: s.subset.clone(),
            tokens_before_exact: s.tokens_before,
            tokens_after_exact: s.tokens_after,
            tokens_before_est: before_est,
            tokens_after_est: after_est,
            ratio_exact,
            ratio_est,
            compressed_sha256: s.compressed_sha256.clone(),
            dict_sha256: s.dict_sha256.clone(),
        });
    }

    let summaries = Summaries {
        input: side_summary(&input_signed, &worst_input),
        compressed: side_summary(&compressed_signed, &worst_comp),
        ratio: side_summary(&ratio_signed, &worst_ratio),
    };
    let objective = summaries.ratio.worst_abs_rel_error;
    (summaries, cells, objective)
}

fn side_summary(signed: &[f64], worst: &(f64, String)) -> SideSummary {
    let mean = if signed.is_empty() {
        0.0
    } else {
        signed.iter().sum::<f64>() / signed.len() as f64
    };
    SideSummary {
        worst_signed_rel_error: worst.0,
        worst_abs_rel_error: worst.0.abs(),
        mean_signed_bias: mean,
        worst_cell: worst.1.clone(),
    }
}

/// Deterministic coordinate descent: coarse grid then three refine passes ×10.
fn fit_flat(samples: &[Sample]) -> ([f64; 3], Summaries, Vec<CellRecord>) {
    let mut best = [0.18_f64, 0.73, 1.0];
    let (_, _, mut best_obj) = evaluate_flat(samples, &best);

    // Coarse grid around the TOK-002 seed.
    for a in grid(0.05, 0.40, 0.02) {
        for b in grid(0.30, 1.20, 0.05) {
            for c in grid(0.50, 1.50, 0.10) {
                let cand = [a, b, c];
                let (_, _, obj) = evaluate_flat(samples, &cand);
                if obj < best_obj - 1e-12 || ((obj - best_obj).abs() < 1e-12 && cand < best) {
                    best = cand;
                    best_obj = obj;
                }
            }
        }
    }

    // Three refinement passes, factor-of-ten narrower each time.
    let mut step = 0.02;
    for _ in 0..3 {
        step /= 10.0;
        let center = best;
        for i in 0..3 {
            for delta in grid(-10.0 * step, 10.0 * step, step) {
                let mut cand = center;
                cand[i] = (center[i] + delta).max(0.01);
                let (_, _, obj) = evaluate_flat(samples, &cand);
                if obj < best_obj - 1e-12 || ((obj - best_obj).abs() < 1e-12 && cand < best) {
                    best = cand;
                    best_obj = obj;
                }
            }
        }
        // Full local neighbourhood of the new centre.
        let center = best;
        for da in grid(-2.0 * step, 2.0 * step, step) {
            for db in grid(-2.0 * step, 2.0 * step, step) {
                for dc in grid(-2.0 * step, 2.0 * step, step) {
                    let cand = [
                        (center[0] + da).max(0.01),
                        (center[1] + db).max(0.01),
                        (center[2] + dc).max(0.01),
                    ];
                    let (_, _, obj) = evaluate_flat(samples, &cand);
                    if obj < best_obj - 1e-12 || ((obj - best_obj).abs() < 1e-12 && cand < best) {
                        best = cand;
                        best_obj = obj;
                    }
                }
            }
        }
    }

    let (summaries, cells, _) = evaluate_flat(samples, &best);
    (best, summaries, cells)
}

fn fit_burst(samples: &[Sample]) -> ([f64; 6], Summaries, Vec<CellRecord>) {
    // Seed: word like alnum (0.18), punct cheaper in bursts.
    let mut best = [0.80_f64, 0.18, 0.90, 0.35, 0.18, 1.0];
    let (_, _, mut best_obj) = evaluate_burst(samples, &best);

    // Coarse one-at-a-time sweep from the seed.
    let ranges: [(f64, f64, f64); 6] = [
        (0.10, 1.50, 0.05), // w0
        (0.02, 0.50, 0.02), // w1
        (0.20, 1.50, 0.05), // p0
        (0.02, 0.80, 0.02), // p1
        (0.01, 0.50, 0.02), // s1
        (0.20, 1.50, 0.05), // n
    ];
    for i in 0..6 {
        let (lo, hi, step) = ranges[i];
        let center = best;
        for v in grid(lo, hi, step) {
            let mut cand = center;
            cand[i] = v;
            let (_, _, obj) = evaluate_burst(samples, &cand);
            if obj < best_obj - 1e-12 || ((obj - best_obj).abs() < 1e-12 && cand < best) {
                best = cand;
                best_obj = obj;
            }
        }
    }

    // Pairwise refinement on (w0,w1) and (p0,p1), then the rest.
    for &(i, j) in &[(0usize, 1usize), (2usize, 3usize)] {
        let center = best;
        let (lo_i, hi_i, step_i) = ranges[i];
        let (lo_j, hi_j, step_j) = ranges[j];
        for vi in grid(lo_i, hi_i, step_i) {
            for vj in grid(lo_j, hi_j, step_j) {
                let mut cand = center;
                cand[i] = vi;
                cand[j] = vj;
                let (_, _, obj) = evaluate_burst(samples, &cand);
                if obj < best_obj - 1e-12 || ((obj - best_obj).abs() < 1e-12 && cand < best) {
                    best = cand;
                    best_obj = obj;
                }
            }
        }
    }

    // Three refine passes.
    let mut step = 0.05;
    for _ in 0..3 {
        step /= 10.0;
        let center = best;
        for i in 0..6 {
            for delta in grid(-10.0 * step, 10.0 * step, step) {
                let mut cand = center;
                cand[i] = (center[i] + delta).max(0.01);
                let (_, _, obj) = evaluate_burst(samples, &cand);
                if obj < best_obj - 1e-12 || ((obj - best_obj).abs() < 1e-12 && cand < best) {
                    best = cand;
                    best_obj = obj;
                }
            }
        }
    }

    let (summaries, cells, _) = evaluate_burst(samples, &best);
    (best, summaries, cells)
}

fn grid(lo: f64, hi: f64, step: f64) -> Vec<f64> {
    let mut out = Vec::new();
    let mut v = lo;
    // Inclusive upper bound with float-safe termination.
    while v <= hi + step * 0.5 {
        out.push((v * 1e9).round() / 1e9);
        v += step;
    }
    out
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn main() {
    let samples = collect_samples();
    eprintln!("Fitting flat model…");
    let (flat_coeffs, flat_sum, flat_cells) = fit_flat(&samples);
    eprintln!(
        "  flat coeffs=[{:.4}, {:.4}, {:.4}]  ratio_worst_abs={:.4}  at {}",
        flat_coeffs[0],
        flat_coeffs[1],
        flat_coeffs[2],
        flat_sum.ratio.worst_abs_rel_error,
        flat_sum.ratio.worst_cell
    );
    eprintln!(
        "  input_worst_abs={:.4}  compressed_worst_abs={:.4}  ratio_bias={:.4}",
        flat_sum.input.worst_abs_rel_error,
        flat_sum.compressed.worst_abs_rel_error,
        flat_sum.ratio.mean_signed_bias
    );

    eprintln!("Fitting burst model…");
    let (burst_coeffs, burst_sum, burst_cells) = fit_burst(&samples);
    eprintln!(
        "  burst coeffs=[w0={:.4}, w1={:.4}, p0={:.4}, p1={:.4}, s1={:.4}, n={:.4}]",
        burst_coeffs[0],
        burst_coeffs[1],
        burst_coeffs[2],
        burst_coeffs[3],
        burst_coeffs[4],
        burst_coeffs[5]
    );
    eprintln!(
        "  ratio_worst_abs={:.4}  at {}",
        burst_sum.ratio.worst_abs_rel_error, burst_sum.ratio.worst_cell
    );
    eprintln!(
        "  input_worst_abs={:.4}  compressed_worst_abs={:.4}  ratio_bias={:.4}",
        burst_sum.input.worst_abs_rel_error,
        burst_sum.compressed.worst_abs_rel_error,
        burst_sum.ratio.mean_signed_bias
    );

    // Target for keeping flat: ratio worst abs ≤ 0.05 (action threshold band).
    // Prefer flat when it meets the product guarantee band; otherwise burst.
    const FLAT_KEEP_TARGET: f64 = 0.05;
    let flat_ratio_worst = flat_sum.ratio.worst_abs_rel_error;
    let burst_ratio_worst = burst_sum.ratio.worst_abs_rel_error;
    let (model, coeffs_json, fitted_summaries, _fitted_cells, decision) = if flat_ratio_worst
        <= FLAT_KEEP_TARGET
    {
        (
                "flat".to_string(),
                serde_json::json!({
                    "alnum_space": round4(flat_coeffs[0]),
                    "other_ascii": round4(flat_coeffs[1]),
                    "non_ascii": round4(flat_coeffs[2]),
                }),
                flat_sum,
                flat_cells,
                format!(
                    "Kept flat model: ratio worst abs {flat_ratio_worst:.4} ≤ {FLAT_KEEP_TARGET}. Smallest change that meets the product-guarantee band."
                ),
            )
    } else if burst_ratio_worst < flat_ratio_worst {
        (
                "burst".to_string(),
                serde_json::json!({
                    "w0": round4(burst_coeffs[0]),
                    "w1": round4(burst_coeffs[1]),
                    "p0": round4(burst_coeffs[2]),
                    "p1": round4(burst_coeffs[3]),
                    "s1": round4(burst_coeffs[4]),
                    "n": round4(burst_coeffs[5]),
                    "digit_rule": "ceil(L/3) per digit run",
                }),
                burst_sum,
                burst_cells,
                format!(
                    "Chose burst model: flat ratio worst abs {flat_ratio_worst:.4} exceeded {FLAT_KEEP_TARGET}; burst achieves {burst_ratio_worst:.4}."
                ),
            )
    } else {
        (
                "flat".to_string(),
                serde_json::json!({
                    "alnum_space": round4(flat_coeffs[0]),
                    "other_ascii": round4(flat_coeffs[1]),
                    "non_ascii": round4(flat_coeffs[2]),
                }),
                flat_sum,
                flat_cells,
                format!(
                    "Kept flat model despite missing {FLAT_KEEP_TARGET}: flat ratio worst abs {flat_ratio_worst:.4} ≤ burst {burst_ratio_worst:.4}. Action thresholds in ADR-014 apply."
                ),
            )
    };

    // Re-evaluate with rounded coefficients so the artefact matches what ships.
    let (final_summaries, final_cells) = if model == "flat" {
        let c = [
            coeffs_json["alnum_space"].as_f64().unwrap(),
            coeffs_json["other_ascii"].as_f64().unwrap(),
            coeffs_json["non_ascii"].as_f64().unwrap(),
        ];
        let (s, cells, _) = evaluate_flat(&samples, &c);
        (s, cells)
    } else {
        let c = [
            coeffs_json["w0"].as_f64().unwrap(),
            coeffs_json["w1"].as_f64().unwrap(),
            coeffs_json["p0"].as_f64().unwrap(),
            coeffs_json["p1"].as_f64().unwrap(),
            coeffs_json["s1"].as_f64().unwrap(),
            coeffs_json["n"].as_f64().unwrap(),
        ];
        let (s, cells, _) = evaluate_burst(&samples, &c);
        (s, cells)
    };
    let _ = fitted_summaries; // fitted (pre-round) summaries already printed
    let report = CalibrationReport {
        model: model.clone(),
        coefficients: coeffs_json,
        summaries: final_summaries,
        cells: final_cells,
        decision: decision.clone(),
    };

    let path = repo_root().join("docs/context/estimator-calibration.json");
    fs::create_dir_all(path.parent().unwrap()).expect("docs");
    fs::write(&path, serde_json::to_string_pretty(&report).expect("json")).expect("write");
    println!("Wrote {}", path.display());
    println!("model={model}");
    println!("{decision}");
    println!(
        "ratio: worst_abs={:.4} worst_signed={:.4} bias={:.4} cell={}",
        report.summaries.ratio.worst_abs_rel_error,
        report.summaries.ratio.worst_signed_rel_error,
        report.summaries.ratio.mean_signed_bias,
        report.summaries.ratio.worst_cell
    );
    println!(
        "input: worst_abs={:.4} bias={:.4}",
        report.summaries.input.worst_abs_rel_error, report.summaries.input.mean_signed_bias
    );
    println!(
        "compressed: worst_abs={:.4} bias={:.4}",
        report.summaries.compressed.worst_abs_rel_error,
        report.summaries.compressed.mean_signed_bias
    );

    let mut failed = false;
    if report.summaries.ratio.worst_abs_rel_error > HEURISTIC_MAX_RATIO_ERROR {
        eprintln!(
            "FAIL: ratio product guarantee {:.4} > HEURISTIC_MAX_RATIO_ERROR {}",
            report.summaries.ratio.worst_abs_rel_error, HEURISTIC_MAX_RATIO_ERROR
        );
        failed = true;
    }
    if report.summaries.input.worst_abs_rel_error > HEURISTIC_INPUT_MAX_RELATIVE_ERROR {
        eprintln!(
            "FAIL: input diagnostic {:.4} > HEURISTIC_INPUT_MAX_RELATIVE_ERROR {}",
            report.summaries.input.worst_abs_rel_error, HEURISTIC_INPUT_MAX_RELATIVE_ERROR
        );
        failed = true;
    }
    if report.summaries.compressed.worst_abs_rel_error > HEURISTIC_COMPRESSED_MAX_RELATIVE_ERROR {
        eprintln!(
            "FAIL: compressed diagnostic {:.4} > HEURISTIC_COMPRESSED_MAX_RELATIVE_ERROR {}",
            report.summaries.compressed.worst_abs_rel_error,
            HEURISTIC_COMPRESSED_MAX_RELATIVE_ERROR
        );
        failed = true;
    }
    if failed {
        std::process::exit(1);
    }
}
