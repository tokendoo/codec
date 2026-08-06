//! Bound HeuristicEstimator error against exact cl100k counts from TOK-002.

use std::fs;
use std::path::PathBuf;

use tokendoo_core::{HeuristicEstimator, TokenEstimator, HEURISTIC_MAX_RELATIVE_ERROR};

/// Exact `cl100k_base` token counts for committed fixture inputs
/// (from `docs/context/tok-002-measurements.json`, S0 / control).
const FIXTURE_EXACT: &[(&str, u64)] = &[
    ("homogeneous_8", 235),
    ("homogeneous_100", 2903),
    ("homogeneous_1000", 29003),
    ("heterogeneous_nested", 766),
    ("realistic_api", 3200),
    ("prose_heavy", 2965),
    ("size_1k", 380),
    ("size_10k", 3860),
    ("size_100k", 39983),
    ("size_250k", 98603),
];

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates")
        .join("tokendoo-bench/fixtures")
}

#[test]
fn heuristic_within_documented_bound_on_tok002_fixtures() {
    let dir = fixtures_dir();
    let est = HeuristicEstimator;
    let mut worst = 0.0_f64;
    let mut worst_name = "";
    for (name, exact) in FIXTURE_EXACT {
        let path = dir.join(format!("{name}.json"));
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("missing fixture {}: {e}", path.display()));
        let estimate = est.estimate(&text);
        let rel = (estimate as f64 - *exact as f64).abs() / (*exact as f64);
        if rel > worst {
            worst = rel;
            worst_name = name;
        }
        assert!(
            rel <= HEURISTIC_MAX_RELATIVE_ERROR,
            "fixture {name}: exact={exact} estimate={estimate} rel={rel:.4} exceeds bound {}",
            HEURISTIC_MAX_RELATIVE_ERROR
        );
    }
    eprintln!("worst relative error: {worst:.4} on {worst_name}");
}
