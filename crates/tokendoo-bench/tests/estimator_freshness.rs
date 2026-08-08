//! Guard: the committed TOK-002 measurement artefact must name the same
//! estimator the crate currently ships. Changing coefficients without
//! regenerating the artefact used to land silently in the same commit
//! (`7197c7d`); this test makes that a hard failure.

use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use tokendoo::{ESTIMATOR_COEFFICIENTS, ESTIMATOR_MODEL};

#[derive(Debug, Deserialize)]
struct Measurements {
    estimator: EstimatorBlock,
}

#[derive(Debug, Deserialize)]
struct EstimatorBlock {
    model: String,
    coefficients: Coeffs,
}

#[derive(Debug, Deserialize)]
struct Coeffs {
    w0: f64,
    w1: f64,
    p0: f64,
    p1: f64,
    s1: f64,
    n: f64,
}

#[test]
fn tok002_measurements_name_the_shipped_estimator() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace")
        .join("docs/context/tok-002-measurements.json");
    let raw = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "missing {}: {e}; run `cargo run -p tokendoo-bench --release`",
            path.display()
        )
    });
    let report: Measurements = serde_json::from_str(&raw).unwrap_or_else(|e| {
        panic!(
            "failed to parse {} ({e}); the artefact needs an `estimator` block — \
             regenerate with `cargo run -p tokendoo-bench --release`",
            path.display()
        )
    });

    assert_eq!(
        report.estimator.model, ESTIMATOR_MODEL,
        "tok-002-measurements.json names model {:?}, but the shipped estimator is {:?}; \
         regenerate the artefact",
        report.estimator.model, ESTIMATOR_MODEL
    );

    let c = &report.estimator.coefficients;
    let s = ESTIMATOR_COEFFICIENTS;
    for (name, got, want) in [
        ("w0", c.w0, s.w0),
        ("w1", c.w1, s.w1),
        ("p0", c.p0, s.p0),
        ("p1", c.p1, s.p1),
        ("s1", c.s1, s.s1),
        ("n", c.n, s.n),
    ] {
        assert!(
            (got - want).abs() < 1e-9,
            "tok-002-measurements.json coefficient {name}={got} != shipped {want}; regenerate"
        );
    }
}
