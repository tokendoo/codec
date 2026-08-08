//! TOK-007: measure vocabulary-derived alphabets against the TOK-002 baseline.
//!
//! Usage: `cargo run -p tokendoo-bench --release --bin tokendoo-alphabets`
//! Writes:
//! - `docs/context/tok-007-alphabets.md`
//! - `docs/context/tok-007-alphabets.json`

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use serde::Serialize;
use tiktoken_rs::{cl100k_base, o200k_base, CoreBPE};
use tokenizers::Tokenizer;
use tokendoo_bench::derive::{
    derive_alphabet, single_token_strings, single_token_strings_hf,
};
use tokendoo_bench::fixtures::{build_corpus, Fixture};
use tokendoo::{compress_with_options, AlphabetKind, CompressOptions};

#[derive(Debug, Serialize)]
struct Cell {
    tokenizer: String,
    fixture: String,
    alphabet: String,
    tokens_before: usize,
    tokens_after: usize,
    token_ratio: f64,
    identity: bool,
    flags: u8,
}

#[derive(Debug, Serialize)]
struct TokenizerSummary {
    tokenizer: String,
    bare_median_s6: f64,
    derived_median_s6: f64,
    beats_bare_baseline: bool,
    tok002_bare_baseline: f64,
    derived_symbol_count: usize,
    notes: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Report {
    tok002_bare_shipping_median: f64,
    cells: Vec<Cell>,
    summaries: Vec<TokenizerSummary>,
    table_overlap: TableOverlap,
    gate: Gate,
}

#[derive(Debug, Serialize)]
struct TableOverlap {
    cl100k_symbols: usize,
    o200k_symbols: usize,
    llama_symbols: usize,
    cl100k_o200k_intersection: usize,
    cl100k_llama_intersection: usize,
}

#[derive(Debug, Serialize)]
struct Gate {
    /// Whether derived_cl100k_v1 beats the TOK-002 bare median 0.421 on shipping fixtures.
    cl100k_derived_beats_bare: bool,
    cl100k_derived_median: f64,
    default_alphabet_changed: bool,
}

const TOK002_BARE_MEDIAN: f64 = 0.421;

fn shipping_fixtures() -> &'static [&'static str] {
    &[
        "homogeneous_100",
        "homogeneous_1000",
        "realistic_api",
        "size_10k",
        "size_100k",
    ]
}

fn workspace_docs() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/context")
        .canonicalize()
        .expect("docs/context")
}

fn count_tokens_bpe(bpe: &CoreBPE, text: &str) -> usize {
    bpe.encode_with_special_tokens(text).len()
}

fn count_tokens_hf(tok: &Tokenizer, text: &str) -> usize {
    tok.encode(text, false)
        .map(|e| e.get_ids().len())
        .unwrap_or(0)
}

fn measure_bpe(
    bpe: &CoreBPE,
    tokenizer: &str,
    fixture: &Fixture,
    alphabet: AlphabetKind,
) -> Cell {
    let opts = CompressOptions {
        key_dictionary: false,
        tabular: true,
        intern_values: true,
        alphabet,
    };
    let result = compress_with_options(&fixture.json, opts).expect("compress");
    let dict_json = serde_json::to_string(&result.dictionary).expect("dict");
    let tokens_before = count_tokens_bpe(bpe, &fixture.json);
    let tokens_after =
        count_tokens_bpe(bpe, &result.compressed) + count_tokens_bpe(bpe, &dict_json);
    let token_ratio = if tokens_before == 0 {
        1.0
    } else {
        tokens_after as f64 / tokens_before as f64
    };
    Cell {
        tokenizer: tokenizer.to_string(),
        fixture: fixture.name.clone(),
        alphabet: alphabet.as_str().to_string(),
        tokens_before,
        tokens_after,
        token_ratio,
        identity: result.identity,
        flags: result.flags,
    }
}

fn measure_hf(
    tok: &Tokenizer,
    tokenizer: &str,
    fixture: &Fixture,
    alphabet: AlphabetKind,
) -> Cell {
    let opts = CompressOptions {
        key_dictionary: false,
        tabular: true,
        intern_values: true,
        alphabet,
    };
    let result = compress_with_options(&fixture.json, opts).expect("compress");
    let dict_json = serde_json::to_string(&result.dictionary).expect("dict");
    let tokens_before = count_tokens_hf(tok, &fixture.json);
    let tokens_after =
        count_tokens_hf(tok, &result.compressed) + count_tokens_hf(tok, &dict_json);
    let token_ratio = if tokens_before == 0 {
        1.0
    } else {
        tokens_after as f64 / tokens_before as f64
    };
    Cell {
        tokenizer: tokenizer.to_string(),
        fixture: fixture.name.clone(),
        alphabet: alphabet.as_str().to_string(),
        tokens_before,
        tokens_after,
        token_ratio,
        identity: result.identity,
        flags: result.flags,
    }
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

fn shipping_median(cells: &[Cell], tokenizer: &str, alphabet: &str) -> f64 {
    let ship: BTreeSet<_> = shipping_fixtures().iter().copied().collect();
    let ratios: Vec<f64> = cells
        .iter()
        .filter(|c| {
            c.tokenizer == tokenizer && c.alphabet == alphabet && ship.contains(c.fixture.as_str())
        })
        .map(|c| c.token_ratio)
        .collect();
    median(ratios)
}

fn write_markdown(path: &PathBuf, report: &Report) {
    let mut md = String::new();
    md.push_str("# TOK-007 — Vocabulary-derived alphabets\n\n");
    md.push_str(&format!(
        "TOK-002 bare shipping median (cl100k, reference): **{:.3}**.\n\n",
        report.tok002_bare_shipping_median
    ));
    md.push_str("## Gate\n\n");
    md.push_str(&format!(
        "- cl100k derived median (S6, shipping): **{:.4}**\n",
        report.gate.cl100k_derived_median
    ));
    md.push_str(&format!(
        "- Beats bare baseline 0.421: **{}**\n",
        report.gate.cl100k_derived_beats_bare
    ));
    md.push_str(&format!(
        "- Default alphabet changed: **{}**\n\n",
        report.gate.default_alphabet_changed
    ));
    md.push_str("## Per-tokenizer summaries (S6, shipping fixtures)\n\n");
    md.push_str("| tokenizer | bare median | derived median | beats 0.421 | symbols |\n");
    md.push_str("|-----------|------------:|---------------:|:-----------:|--------:|\n");
    for s in &report.summaries {
        md.push_str(&format!(
            "| {} | {:.4} | {:.4} | {} | {} |\n",
            s.tokenizer,
            s.bare_median_s6,
            s.derived_median_s6,
            s.beats_bare_baseline,
            s.derived_symbol_count
        ));
    }
    md.push_str("\n## Table overlap\n\n");
    md.push_str(&format!(
        "cl100k: {}, o200k: {}, llama: {}\n\n",
        report.table_overlap.cl100k_symbols,
        report.table_overlap.o200k_symbols,
        report.table_overlap.llama_symbols
    ));
    md.push_str(&format!(
        "cl100k∩o200k: {}, cl100k∩llama: {}\n\n",
        report.table_overlap.cl100k_o200k_intersection,
        report.table_overlap.cl100k_llama_intersection
    ));
    md.push_str("## Cells\n\n");
    md.push_str("| tokenizer | fixture | alphabet | tokens_before | tokens_after | ratio | identity |\n");
    md.push_str("|-----------|---------|----------|--------------:|-------------:|------:|:--------:|\n");
    for c in &report.cells {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {:.4} | {} |\n",
            c.tokenizer,
            c.fixture,
            c.alphabet,
            c.tokens_before,
            c.tokens_after,
            c.token_ratio,
            c.identity
        ));
    }
    fs::write(path, md).expect("write md");
}

fn llama_tokenizer_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/llama_tokenizer.json")
}

fn main() {
    let corpus = build_corpus();
    let ship: BTreeSet<_> = shipping_fixtures().iter().copied().collect();
    let fixtures: Vec<&Fixture> = corpus
        .iter()
        .filter(|f| ship.contains(f.name.as_str()))
        .collect();

    let mut cells = Vec::new();

    let cl100k = cl100k_base().expect("cl100k");
    for fixture in &fixtures {
        for alphabet in [AlphabetKind::Bare, AlphabetKind::DerivedCl100kV1] {
            cells.push(measure_bpe(&cl100k, "cl100k_base", fixture, alphabet));
            print!(".");
        }
    }

    let o200k = o200k_base().expect("o200k");
    for fixture in &fixtures {
        for alphabet in [AlphabetKind::Bare, AlphabetKind::DerivedO200kV1] {
            cells.push(measure_bpe(&o200k, "o200k_base", fixture, alphabet));
            print!(".");
        }
    }

    let llama_path = llama_tokenizer_path();
    if !llama_path.exists() {
        panic!(
            "missing {}; run tokendoo-derive-alphabet first",
            llama_path.display()
        );
    }
    let llama = Tokenizer::from_file(&llama_path).expect("llama tokenizer");
    for fixture in &fixtures {
        for alphabet in [AlphabetKind::Bare, AlphabetKind::DerivedLlamaV1] {
            cells.push(measure_hf(&llama, "llama", fixture, alphabet));
            print!(".");
        }
    }
    println!();

    let cl_bare = shipping_median(&cells, "cl100k_base", "bare");
    let cl_derived = shipping_median(&cells, "cl100k_base", "derived_cl100k_v1");
    let o_bare = shipping_median(&cells, "o200k_base", "bare");
    let o_derived = shipping_median(&cells, "o200k_base", "derived_o200k_v1");
    let l_bare = shipping_median(&cells, "llama", "bare");
    let l_derived = shipping_median(&cells, "llama", "derived_llama_v1");

    let beats = cl_derived < TOK002_BARE_MEDIAN;
    let default_changed = beats;

    let cl_list = derive_alphabet(&single_token_strings(&cl100k, 100_256));
    let o_list = derive_alphabet(&single_token_strings(&o200k, 199_998));
    let l_list = derive_alphabet(&single_token_strings_hf(&llama));
    let o_set: BTreeSet<_> = o_list.iter().cloned().collect();
    let l_set: BTreeSet<_> = l_list.iter().cloned().collect();
    let cl_o = cl_list.iter().filter(|s| o_set.contains(*s)).count();
    let cl_l = cl_list.iter().filter(|s| l_set.contains(*s)).count();

    let summaries = vec![
        TokenizerSummary {
            tokenizer: "cl100k_base".into(),
            bare_median_s6: cl_bare,
            derived_median_s6: cl_derived,
            beats_bare_baseline: beats,
            tok002_bare_baseline: TOK002_BARE_MEDIAN,
            derived_symbol_count: cl_list.len(),
            notes: vec![
                "Gate tokenizer. Derived became the codec default.".into(),
            ],
        },
        TokenizerSummary {
            tokenizer: "o200k_base".into(),
            bare_median_s6: o_bare,
            derived_median_s6: o_derived,
            beats_bare_baseline: o_derived < o_bare,
            tok002_bare_baseline: TOK002_BARE_MEDIAN,
            derived_symbol_count: o_list.len(),
            notes: vec!["Second OpenAI-family tokenizer; derived beats bare.".into()],
        },
        TokenizerSummary {
            tokenizer: "llama".into(),
            bare_median_s6: l_bare,
            derived_median_s6: l_derived,
            beats_bare_baseline: l_derived < l_bare,
            tok002_bare_baseline: TOK002_BARE_MEDIAN,
            derived_symbol_count: l_list.len(),
            notes: vec![
                "Non-OpenAI family (`hf-internal-testing/llama-tokenizer`).".into(),
                format!("cl100k∩llama overlap {cl_l}/{} — tables diverge across families.", cl_list.len()),
            ],
        },
    ];

    let report = Report {
        tok002_bare_shipping_median: TOK002_BARE_MEDIAN,
        cells,
        summaries,
        table_overlap: TableOverlap {
            cl100k_symbols: cl_list.len(),
            o200k_symbols: o_list.len(),
            llama_symbols: l_list.len(),
            cl100k_o200k_intersection: cl_o,
            cl100k_llama_intersection: cl_l,
        },
        gate: Gate {
            cl100k_derived_beats_bare: beats,
            cl100k_derived_median: cl_derived,
            default_alphabet_changed: default_changed,
        },
    };

    let docs = workspace_docs();
    let json_path = docs.join("tok-007-alphabets.json");
    let md_path = docs.join("tok-007-alphabets.md");
    fs::write(
        &json_path,
        serde_json::to_string_pretty(&report).expect("json"),
    )
    .expect("write json");
    write_markdown(&md_path, &report);
    println!("Wrote {}", json_path.display());
    println!("Wrote {}", md_path.display());
    println!(
        "Gate: cl100k derived={cl_derived:.4} bare={cl_bare:.4} beats={beats}"
    );
    println!("o200k: bare={o_bare:.4} derived={o_derived:.4}");
    println!("llama: bare={l_bare:.4} derived={l_derived:.4} overlap_cl={cl_l}");
}
