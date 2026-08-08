//! Host-side helper for cross-language round-trip checks.
//!
//! Commands:
//! - `compress <input.json>` prints `{compressed, dictionary}` JSON on stdout
//! - `decompress <pair.json>` prints the recovered JSON text on stdout

use std::env;
use std::fs;
use std::process::ExitCode;

use serde::{Deserialize, Serialize};
use tokendoo::{compress, decompress, Dictionary};

#[derive(Serialize, Deserialize)]
struct Pair {
    compressed: String,
    dictionary: Dictionary,
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(cmd) = args.next() else {
        eprintln!("usage: cross-lang compress|decompress <path>");
        return ExitCode::from(2);
    };
    let Some(path) = args.next() else {
        eprintln!("usage: cross-lang compress|decompress <path>");
        return ExitCode::from(2);
    };
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => {
            eprintln!("failed to read input path");
            return ExitCode::from(1);
        }
    };
    match cmd.as_str() {
        "compress" => match compress(&text) {
            Ok(result) => {
                let pair = Pair {
                    compressed: result.compressed,
                    dictionary: result.dictionary,
                };
                match serde_json::to_string(&pair) {
                    Ok(s) => {
                        println!("{s}");
                        ExitCode::SUCCESS
                    }
                    Err(_) => {
                        eprintln!("failed to encode pair");
                        ExitCode::from(1)
                    }
                }
            }
            Err(err) => {
                eprintln!("{err}");
                ExitCode::from(1)
            }
        },
        "decompress" => {
            let pair: Pair = match serde_json::from_str(&text) {
                Ok(p) => p,
                Err(_) => {
                    eprintln!("failed to parse pair JSON");
                    return ExitCode::from(1);
                }
            };
            match decompress(&pair.compressed, &pair.dictionary) {
                Ok(out) => {
                    println!("{out}");
                    ExitCode::SUCCESS
                }
                Err(err) => {
                    eprintln!("{err}");
                    ExitCode::from(1)
                }
            }
        }
        _ => {
            eprintln!("unknown command");
            ExitCode::from(2)
        }
    }
}
