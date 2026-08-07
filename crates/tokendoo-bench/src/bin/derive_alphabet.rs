//! Generate frozen derived symbol tables into `tokendoo-core`.
//!
//! Usage: `cargo run -p tokendoo-bench --release --bin tokendoo-derive-alphabet`

use std::fs;
use std::path::PathBuf;

use tiktoken_rs::{cl100k_base, o200k_base};
use tokenizers::Tokenizer;
use tokendoo_bench::derive::{
    derive_alphabet, render_symbols_rs, single_token_strings, single_token_strings_hf,
};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn write_table(name: &str, doc: &str, symbols: &[String]) {
    let path = workspace_root()
        .join("crates/tokendoo-core/src/json/derived_tables")
        .join(format!("{name}.rs"));
    let src = render_symbols_rs(doc, symbols);
    fs::write(&path, src).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
    println!("Wrote {} symbols → {}", symbols.len(), path.display());
}

fn ensure_llama_tokenizer() -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/llama_tokenizer.json");
    if path.exists() {
        return path;
    }
    fs::create_dir_all(path.parent().expect("parent")).expect("mkdir assets");
    let url = "https://huggingface.co/hf-internal-testing/llama-tokenizer/resolve/main/tokenizer.json";
    println!("Downloading Llama tokenizer → {}", path.display());
    let response = ureq::get(url).call().expect("download llama tokenizer");
    let bytes = response
        .into_body()
        .read_to_vec()
        .expect("read tokenizer body");
    fs::write(&path, bytes).expect("write tokenizer");
    path
}

fn main() {
    println!("Deriving cl100k_base alphabet…");
    let cl100k = cl100k_base().expect("cl100k_base");
    let cl_syms = derive_alphabet(&single_token_strings(&cl100k, 100_256));
    write_table(
        "cl100k_v1",
        "Symbol table derived from `cl100k_base` (TOK-007).",
        &cl_syms,
    );

    println!("Deriving o200k_base alphabet…");
    let o200k = o200k_base().expect("o200k_base");
    let o_syms = derive_alphabet(&single_token_strings(&o200k, 199_998));
    write_table(
        "o200k_v1",
        "Symbol table derived from `o200k_base` (TOK-007).",
        &o_syms,
    );

    println!("Deriving Llama alphabet…");
    let llama_path = ensure_llama_tokenizer();
    let llama = Tokenizer::from_file(&llama_path).expect("load llama tokenizer");
    let llama_syms = derive_alphabet(&single_token_strings_hf(&llama));
    write_table(
        "llama_v1",
        "Symbol table derived from `hf-internal-testing/llama-tokenizer` (TOK-007).",
        &llama_syms,
    );

    let o_set: std::collections::BTreeSet<_> = o_syms.iter().cloned().collect();
    let l_set: std::collections::BTreeSet<_> = llama_syms.iter().cloned().collect();
    let cl_o = cl_syms.iter().filter(|s| o_set.contains(*s)).count();
    let cl_l = cl_syms.iter().filter(|s| l_set.contains(*s)).count();
    println!(
        "Overlap cl100k∩o200k = {cl_o} / {}; cl100k∩llama = {cl_l} / {}",
        cl_syms.len(),
        cl_syms.len()
    );
}
