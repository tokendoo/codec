//! Bound HeuristicEstimator error against the TOK-002 calibration artefact.
//!
//! Exact `cl100k_base` counts live in `docs/context/estimator-calibration.json`
//! (produced by `tokendoo-calibrate`). This test recompresses the committed
//! fixtures and:
//!
//! 1. Checks SHA-256 of `compressed` and the dictionary against the artefact —
//!    a mismatch means the **codec** changed and the artefact must be
//!    regenerated, not that the estimator is out of spec.
//! 2. Asserts the product-guarantee ratio bound and the two diagnostic side
//!    bounds.
//! 3. Asserts each documented signed bias stays within
//!    [`HEURISTIC_BIAS_TOLERANCE`] of the published value — the check that
//!    catches silent loss of same-sign compensation.
//!
//! Default scope excludes fixtures larger than 100 KiB so `cargo test` stays
//! interactive; the full grid is enforced by `tokendoo-calibrate`, which exits
//! non-zero on any violation.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokendoo_core::{
    compress_with_options, AlphabetKind, CompressOptions, EstimatorCoefficients,
    HeuristicEstimator, TokenEstimator, ESTIMATOR_COEFFICIENTS, ESTIMATOR_MODEL,
    HEURISTIC_BIAS_TOLERANCE, HEURISTIC_COMPRESSED_BIAS, HEURISTIC_COMPRESSED_MAX_RELATIVE_ERROR,
    HEURISTIC_INPUT_BIAS, HEURISTIC_INPUT_MAX_RELATIVE_ERROR, HEURISTIC_MAX_RATIO_ERROR,
    HEURISTIC_RATIO_BIAS,
};

#[derive(Debug, Deserialize)]
struct CalibrationReport {
    model: String,
    coefficients: CoeffsJson,
    cells: Vec<CellRecord>,
}

#[derive(Debug, Deserialize)]
struct CoeffsJson {
    w0: f64,
    w1: f64,
    p0: f64,
    p1: f64,
    s1: f64,
    n: f64,
}

#[derive(Debug, Deserialize)]
struct CellRecord {
    fixture: String,
    alphabet: String,
    subset: String,
    tokens_before_exact: u64,
    tokens_after_exact: u64,
    compressed_sha256: String,
    dict_sha256: String,
}

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

/// Fixtures larger than this are left to `tokendoo-calibrate`.
const MAX_FIXTURE_BYTES_FOR_DEFAULT_TEST: u64 = 100 * 1024;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates")
        .parent()
        .expect("workspace")
        .to_path_buf()
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates")
        .join("tokendoo-bench/fixtures")
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn parse_alphabet(name: &str) -> AlphabetKind {
    match name {
        "control" => AlphabetKind::Control,
        "printable" => AlphabetKind::Printable,
        "bare" => AlphabetKind::Bare,
        "common_word" => AlphabetKind::CommonWord,
        other => panic!("unknown alphabet {other}"),
    }
}

fn subset_opts(name: &str, alphabet: AlphabetKind) -> CompressOptions {
    let subset = SUBSETS
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("unknown subset {name}"));
    CompressOptions {
        key_dictionary: subset.key_dictionary,
        tabular: subset.tabular,
        intern_values: subset.intern_values,
        alphabet,
    }
}

fn rel_signed(est: f64, exact: f64) -> f64 {
    if exact == 0.0 {
        0.0
    } else {
        (est - exact) / exact
    }
}

fn coeffs_match(a: &CoeffsJson, b: EstimatorCoefficients) -> bool {
    (a.w0 - b.w0).abs() < 1e-9
        && (a.w1 - b.w1).abs() < 1e-9
        && (a.p0 - b.p0).abs() < 1e-9
        && (a.p1 - b.p1).abs() < 1e-9
        && (a.s1 - b.s1).abs() < 1e-9
        && (a.n - b.n).abs() < 1e-9
}

#[test]
fn heuristic_bounds_and_biases_on_calibration_grid() {
    let root = repo_root();
    let report_path = root.join("docs/context/estimator-calibration.json");
    let raw = fs::read_to_string(&report_path).unwrap_or_else(|e| {
        panic!(
            "missing calibration artefact {}: {e}; run `cargo run -p tokendoo-bench --release --bin tokendoo-calibrate`",
            report_path.display()
        )
    });
    let report: CalibrationReport = serde_json::from_str(&raw).expect("parse calibration");

    assert_eq!(
        report.model, ESTIMATOR_MODEL,
        "artefact model {:?} != shipped ESTIMATOR_MODEL {:?}",
        report.model, ESTIMATOR_MODEL
    );
    assert!(
        coeffs_match(&report.coefficients, ESTIMATOR_COEFFICIENTS),
        "artefact coefficients do not match ESTIMATOR_COEFFICIENTS; regenerate or update tokenizer.rs"
    );

    let dir = fixtures_dir();
    let est = HeuristicEstimator;

    // Cache fixture texts and skip oversized ones for the default test.
    let mut fixture_text: BTreeMap<String, String> = BTreeMap::new();
    let mut skipped = BTreeMap::new();
    for cell in &report.cells {
        if fixture_text.contains_key(&cell.fixture) || skipped.contains_key(&cell.fixture) {
            continue;
        }
        let path = dir.join(format!("{}.json", cell.fixture));
        let meta = fs::metadata(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        if meta.len() > MAX_FIXTURE_BYTES_FOR_DEFAULT_TEST {
            skipped.insert(cell.fixture.clone(), meta.len());
            continue;
        }
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        fixture_text.insert(cell.fixture.clone(), text);
    }

    let mut input_signed = Vec::new();
    let mut compressed_signed = Vec::new();
    let mut ratio_signed = Vec::new();
    let mut checked = 0_usize;

    for cell in &report.cells {
        if skipped.contains_key(&cell.fixture) {
            continue;
        }
        let input = fixture_text
            .get(&cell.fixture)
            .unwrap_or_else(|| panic!("fixture {} not loaded", cell.fixture));
        let alphabet = parse_alphabet(&cell.alphabet);
        let opts = subset_opts(&cell.subset, alphabet);
        let result = compress_with_options(input, opts).expect("compress");
        let dict_json = serde_json::to_string(&result.dictionary).expect("dict");

        let comp_hash = sha256_hex(result.compressed.as_bytes());
        let dict_hash = sha256_hex(dict_json.as_bytes());
        assert_eq!(
            comp_hash, cell.compressed_sha256,
            "codec output hash mismatch for {}|{}|{} — the codec changed; \
             regenerate with `cargo run -p tokendoo-bench --release --bin tokendoo-calibrate` \
             (this is not an estimator failure)",
            cell.fixture, cell.alphabet, cell.subset
        );
        assert_eq!(
            dict_hash, cell.dict_sha256,
            "dictionary hash mismatch for {}|{}|{} — the codec changed; \
             regenerate with `cargo run -p tokendoo-bench --release --bin tokendoo-calibrate` \
             (this is not an estimator failure)",
            cell.fixture, cell.alphabet, cell.subset
        );

        let before_est = est.estimate(input);
        let after_est = est.estimate(&result.compressed) + est.estimate(&dict_json);

        let in_err = rel_signed(before_est as f64, cell.tokens_before_exact as f64);
        let out_err = rel_signed(after_est as f64, cell.tokens_after_exact as f64);
        let ratio_exact = if cell.tokens_before_exact == 0 {
            1.0
        } else {
            cell.tokens_after_exact as f64 / cell.tokens_before_exact as f64
        };
        let ratio_est = if before_est == 0 {
            1.0
        } else {
            after_est as f64 / before_est as f64
        };
        let r_err = rel_signed(ratio_est, ratio_exact);

        assert!(
            in_err.abs() <= HEURISTIC_INPUT_MAX_RELATIVE_ERROR,
            "input diagnostic exceeded on {}|{}|{}: signed={in_err:.4} bound={}",
            cell.fixture,
            cell.alphabet,
            cell.subset,
            HEURISTIC_INPUT_MAX_RELATIVE_ERROR
        );
        assert!(
            out_err.abs() <= HEURISTIC_COMPRESSED_MAX_RELATIVE_ERROR,
            "compressed diagnostic exceeded on {}|{}|{}: signed={out_err:.4} bound={}",
            cell.fixture,
            cell.alphabet,
            cell.subset,
            HEURISTIC_COMPRESSED_MAX_RELATIVE_ERROR
        );
        assert!(
            r_err.abs() <= HEURISTIC_MAX_RATIO_ERROR,
            "PRODUCT GUARANTEE exceeded on {}|{}|{}: signed={r_err:.4} bound={}",
            cell.fixture,
            cell.alphabet,
            cell.subset,
            HEURISTIC_MAX_RATIO_ERROR
        );

        input_signed.push(in_err);
        compressed_signed.push(out_err);
        ratio_signed.push(r_err);
        checked += 1;
    }

    assert!(
        checked > 0,
        "no cells checked; fixture filter may be too aggressive"
    );

    let mean = |xs: &[f64]| xs.iter().sum::<f64>() / xs.len() as f64;
    let input_bias = mean(&input_signed);
    let compressed_bias = mean(&compressed_signed);
    let ratio_bias = mean(&ratio_signed);

    assert!(
        (input_bias - HEURISTIC_INPUT_BIAS).abs() <= HEURISTIC_BIAS_TOLERANCE,
        "input bias drifted: measured={input_bias:.4} documented={} tolerance={}",
        HEURISTIC_INPUT_BIAS,
        HEURISTIC_BIAS_TOLERANCE
    );
    assert!(
        (compressed_bias - HEURISTIC_COMPRESSED_BIAS).abs() <= HEURISTIC_BIAS_TOLERANCE,
        "compressed bias drifted: measured={compressed_bias:.4} documented={} tolerance={}",
        HEURISTIC_COMPRESSED_BIAS,
        HEURISTIC_BIAS_TOLERANCE
    );
    assert!(
        (ratio_bias - HEURISTIC_RATIO_BIAS).abs() <= HEURISTIC_BIAS_TOLERANCE,
        "ratio bias drifted: measured={ratio_bias:.4} documented={} tolerance={}",
        HEURISTIC_RATIO_BIAS,
        HEURISTIC_BIAS_TOLERANCE
    );

    eprintln!(
        "checked {checked} cells (skipped oversized: {:?}); \
         input_bias={input_bias:.4} compressed_bias={compressed_bias:.4} ratio_bias={ratio_bias:.4}",
        skipped.keys().collect::<Vec<_>>()
    );
}
