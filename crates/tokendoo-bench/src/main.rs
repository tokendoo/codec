//! TOK-002 benchmark binary: exact cl100k_base token ratios across alphabets,
//! fixtures, and transform subsets.
//!
//! Usage: `cargo run -p tokendoo-bench --bin tokendoo-bench --release`
//! Writes:
//! - `docs/context/tok-002-measurements.md`
//! - `docs/context/tok-002-measurements.json`

use std::fs;
use std::path::PathBuf;

use serde::Serialize;
use tiktoken_rs::cl100k_base;
use tokendoo_bench::fixtures::{build_corpus, Fixture};
use tokendoo_core::{
    compress_with_options, AlphabetKind, CompressOptions, HeuristicEstimator, TokenEstimator,
    ESTIMATOR_COEFFICIENTS, ESTIMATOR_MODEL,
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

#[derive(Debug, Serialize, Clone)]
struct Cell {
    fixture: String,
    alphabet: String,
    subset: String,
    bytes_before: usize,
    bytes_after: usize,
    tokens_before: usize,
    tokens_after: usize,
    byte_ratio: f64,
    token_ratio: f64,
    divergence: f64,
    identity: bool,
    flags: u8,
    heuristic_tokens_before: u64,
    heuristic_tokens_after: u64,
}

#[derive(Debug, Serialize)]
struct Report {
    baseline_wasm_bytes: u64,
    estimator: EstimatorMeta,
    cells: Vec<Cell>,
    decision: DecisionSummary,
}

#[derive(Debug, Serialize, Clone)]
struct EstimatorMeta {
    model: String,
    coefficients: EstimatorCoeffsJson,
}

#[derive(Debug, Serialize, Clone)]
struct EstimatorCoeffsJson {
    w0: f64,
    w1: f64,
    p0: f64,
    p1: f64,
    s1: f64,
    n: f64,
    digit_rule: &'static str,
}

#[derive(Debug, Serialize, Clone)]
struct DecisionSummary {
    winning_alphabet: String,
    /// Median token ratio on shipping fixtures under [`SHIPPED_SUBSET`].
    median_token_ratio_s6: f64,
    ship_token_claim: bool,
    transforms_default: TransformsDefault,
    notes: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
struct TransformsDefault {
    key_dictionary: bool,
    tabular: bool,
    intern_values: bool,
}

fn count_tokens(text: &str) -> usize {
    let bpe = cl100k_base().expect("cl100k_base");
    bpe.encode_with_special_tokens(text).len()
}

fn measure_cell(fixture: &Fixture, alphabet: AlphabetKind, subset: &Subset) -> Cell {
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
    let byte_ratio = if result.bytes_before == 0 {
        1.0
    } else {
        result.bytes_after as f64 / result.bytes_before as f64
    };
    let token_ratio = if tokens_before == 0 {
        1.0
    } else {
        tokens_after as f64 / tokens_before as f64
    };
    let est = HeuristicEstimator;
    Cell {
        fixture: fixture.name.clone(),
        alphabet: alphabet.as_str().to_string(),
        subset: subset.name.to_string(),
        bytes_before: result.bytes_before,
        bytes_after: result.bytes_after,
        tokens_before,
        tokens_after,
        byte_ratio,
        token_ratio,
        divergence: token_ratio - byte_ratio,
        identity: result.identity,
        flags: result.flags,
        heuristic_tokens_before: est.estimate(&fixture.json),
        heuristic_tokens_after: est.estimate(&result.compressed) + est.estimate(&dict_json),
    }
}

fn shipping_fixtures() -> &'static [&'static str] {
    &[
        "homogeneous_100",
        "homogeneous_1000",
        "realistic_api",
        "size_10k",
        "size_100k",
    ]
}

fn median(mut xs: Vec<f64>) -> f64 {
    if xs.is_empty() {
        return f64::NAN;
    }
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = xs.len() / 2;
    if xs.len().is_multiple_of(2) {
        (xs[mid - 1] + xs[mid]) / 2.0
    } else {
        xs[mid]
    }
}

/// Shipped transform subset: tabular + intern, key dictionary off (ADR-011).
/// Alphabet ranking uses this row, not `S7_full` — on bare the two coincide, on
/// derived they need not, and the landing median is the shipped configuration.
const SHIPPED_SUBSET: &str = "S6_tabular_intern";

fn decide(cells: &[Cell]) -> DecisionSummary {
    let mut notes = Vec::new();
    let ship = shipping_fixtures();

    let mut alphabet_scores: Vec<(AlphabetKind, f64, f64)> = Vec::new();
    for alphabet in AlphabetKind::MEASURED {
        let ratios: Vec<f64> = cells
            .iter()
            .filter(|c| {
                c.alphabet == alphabet.as_str()
                    && c.subset == SHIPPED_SUBSET
                    && ship.contains(&c.fixture.as_str())
            })
            .map(|c| c.token_ratio)
            .collect();
        let med = median(ratios.clone());
        let n = ratios.len().max(1) as f64;
        let mean_abs_div: f64 = cells
            .iter()
            .filter(|c| {
                c.alphabet == alphabet.as_str()
                    && c.subset == SHIPPED_SUBSET
                    && ship.contains(&c.fixture.as_str())
            })
            .map(|c| c.divergence.abs())
            .sum::<f64>()
            / n;
        alphabet_scores.push((alphabet, med, mean_abs_div));
        notes.push(format!(
            "alphabet {}: median_token_ratio_S6={med:.4}, mean_|divergence|={mean_abs_div:.4}",
            alphabet.as_str()
        ));
    }

    alphabet_scores.sort_by(|a, b| {
        a.1.partial_cmp(&b.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
    });
    let (winner, win_med, _) = alphabet_scores[0];
    let ship_token_claim = win_med <= 0.90;

    // Isolation diagnostics only. Transform defaults are pinned by ADR-011
    // (keys off, tabular + intern on) and were not reopened when TOK-007 moved
    // the alphabet; regenerating the matrix must not flip them in the artefact.
    let _ = transform_worth(cells, winner, "S1_keys", ship, &mut notes, "key_dictionary");
    let _ = transform_worth(cells, winner, "S2_tabular", ship, &mut notes, "tabular");
    let _ = transform_worth(
        cells,
        winner,
        "S3_intern",
        ship,
        &mut notes,
        "intern_values",
    );
    notes.push(
        "transforms_default pinned to ADR-011 shipped values (key_dictionary=false, tabular=true, intern_values=true); isolation hits above are diagnostic only"
            .into(),
    );
    let key_dictionary = false;
    let tabular = true;
    let intern_values = true;

    if let Some(p) = cells.iter().find(|c| {
        c.alphabet == winner.as_str()
            && c.subset == SHIPPED_SUBSET
            && c.fixture == "prose_heavy"
    }) {
        notes.push(format!(
            "prose_heavy S6 token_ratio={:.4} (must be <=1.05)",
            p.token_ratio
        ));
        if p.token_ratio > 1.05 {
            notes.push(
                "WARNING: shipped pipeline worsens prose_heavy tokens; size guard may need token-aware revision"
                    .into(),
            );
        }
    }

    if !ship_token_claim {
        notes.push(format!(
            "No alphabet reached median token_ratio <= 0.90 (best {} = {:.4}); do not claim token savings for v1 structural JSON",
            winner.as_str(),
            win_med
        ));
    }

    DecisionSummary {
        winning_alphabet: winner.as_str().to_string(),
        median_token_ratio_s6: win_med,
        ship_token_claim,
        transforms_default: TransformsDefault {
            key_dictionary,
            tabular,
            intern_values,
        },
        notes,
    }
}

fn transform_worth(
    cells: &[Cell],
    winner: AlphabetKind,
    subset_name: &str,
    ship: &[&str],
    notes: &mut Vec<String>,
    flag_name: &str,
) -> bool {
    let hits = cells
        .iter()
        .filter(|c| {
            c.alphabet == winner.as_str()
                && c.subset == subset_name
                && ship.contains(&c.fixture.as_str())
                && c.token_ratio <= 0.90
        })
        .count();
    let prose_ok = cells
        .iter()
        .filter(|c| {
            c.alphabet == winner.as_str() && c.subset == subset_name && c.fixture == "prose_heavy"
        })
        .all(|c| c.token_ratio <= 1.05);
    let worth = hits >= 2 && prose_ok;
    notes.push(format!(
        "transform {flag_name} on {}: hits={hits}/shipping with ratio<=0.90, prose_ok={prose_ok} => default_on={worth}",
        winner.as_str()
    ));
    worth
}

fn write_markdown(path: &PathBuf, report: &Report) {
    let mut md = String::new();
    md.push_str("# TOK-002 Measurements\n\n");
    md.push_str(&format!(
        "Baseline Worker WASM size (pre-change reference): **{}** bytes.\n\n",
        report.baseline_wasm_bytes
    ));
    md.push_str("## Decision\n\n");
    md.push_str(&format!(
        "- Winning alphabet: **{}**\n",
        report.decision.winning_alphabet
    ));
    md.push_str(&format!(
        "- Median S6 token ratio (shipping fixtures, shipped transforms): **{:.4}**\n",
        report.decision.median_token_ratio_s6
    ));
    md.push_str(&format!(
        "- Ship token claim: **{}**\n",
        report.decision.ship_token_claim
    ));
    md.push_str(&format!(
        "- Default transforms: keys={}, tabular={}, intern={}\n\n",
        report.decision.transforms_default.key_dictionary,
        report.decision.transforms_default.tabular,
        report.decision.transforms_default.intern_values
    ));
    md.push_str("### Notes\n\n");
    for n in &report.decision.notes {
        md.push_str(&format!("- {n}\n"));
    }
    md.push_str("\n## Full pipeline (S7) — primary table\n\n");
    md.push_str(
        "| fixture | alphabet | bytes_before | bytes_after | tokens_before | tokens_after | byte_ratio | token_ratio | divergence | identity |\n",
    );
    md.push_str(
        "|---------|----------|-------------:|------------:|--------------:|-------------:|-----------:|------------:|-----------:|---------|\n",
    );
    for c in report.cells.iter().filter(|c| c.subset == "S7_full") {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {:.4} | {:.4} | {:+.4} | {} |\n",
            c.fixture,
            c.alphabet,
            c.bytes_before,
            c.bytes_after,
            c.tokens_before,
            c.tokens_after,
            c.byte_ratio,
            c.token_ratio,
            c.divergence,
            c.identity
        ));
    }
    md.push_str("\n## Isolation appendix (all subsets)\n\n");
    md.push_str(
        "| fixture | alphabet | subset | token_ratio | byte_ratio | divergence | identity |\n",
    );
    md.push_str(
        "|---------|----------|--------|------------:|-----------:|-----------:|---------|\n",
    );
    for c in &report.cells {
        md.push_str(&format!(
            "| {} | {} | {} | {:.4} | {:.4} | {:+.4} | {} |\n",
            c.fixture, c.alphabet, c.subset, c.token_ratio, c.byte_ratio, c.divergence, c.identity
        ));
    }
    fs::write(path, md).expect("write md");
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn main() {
    let root = repo_root();
    let baseline_wasm = 601_320u64; // captured before TOK-002 code changes

    // Persist fixtures for reproducibility.
    let fixtures_dir = root.join("crates/tokendoo-bench/fixtures");
    fs::create_dir_all(&fixtures_dir).expect("fixtures dir");
    let corpus = build_corpus();
    for f in &corpus {
        fs::write(fixtures_dir.join(format!("{}.json", f.name)), &f.json).expect("write fixture");
    }

    eprintln!(
        "Running matrix: {} fixtures × {} alphabets × {} subsets",
        corpus.len(),
        AlphabetKind::MEASURED.len(),
        SUBSETS.len()
    );

    let mut cells = Vec::new();
    for fixture in &corpus {
        for alphabet in AlphabetKind::MEASURED {
            for subset in SUBSETS {
                eprint!(".");
                cells.push(measure_cell(fixture, alphabet, subset));
            }
        }
    }
    eprintln!();

    let decision = decide(&cells);
    let report = Report {
        baseline_wasm_bytes: baseline_wasm,
        estimator: EstimatorMeta {
            model: ESTIMATOR_MODEL.to_string(),
            coefficients: EstimatorCoeffsJson {
                w0: ESTIMATOR_COEFFICIENTS.w0,
                w1: ESTIMATOR_COEFFICIENTS.w1,
                p0: ESTIMATOR_COEFFICIENTS.p0,
                p1: ESTIMATOR_COEFFICIENTS.p1,
                s1: ESTIMATOR_COEFFICIENTS.s1,
                n: ESTIMATOR_COEFFICIENTS.n,
                digit_rule: "ceil(L/3) per digit run",
            },
        },
        cells,
        decision,
    };

    let docs = root.join("docs/context");
    fs::create_dir_all(&docs).expect("docs");
    let json_path = docs.join("tok-002-measurements.json");
    let md_path = docs.join("tok-002-measurements.md");
    fs::write(
        &json_path,
        serde_json::to_string_pretty(&report).expect("json"),
    )
    .expect("write json");
    write_markdown(&md_path, &report);

    println!("Wrote {}", json_path.display());
    println!("Wrote {}", md_path.display());
    println!(
        "Winner={} median_S6={:.4} ship_claim={}",
        report.decision.winning_alphabet,
        report.decision.median_token_ratio_s6,
        report.decision.ship_token_claim
    );
}
