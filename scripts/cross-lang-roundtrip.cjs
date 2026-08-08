#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");

const root = path.resolve(__dirname, "..");
const fixturesDir = path.join(root, "crates/tokendoo-bench/fixtures");
const tokendoo = require(process.env.TOKENDOO_NPM || path.join(root, "crates/tokendoo-wasm/pkg"));
const rustBin = process.env.CROSS_LANG_BIN || path.join(root, "target/release/cross-lang");
const python = process.env.TOKENDOO_PYTHON || "python3";

function run(cmd, args, input) {
  const res = spawnSync(cmd, args, {
    input,
    encoding: "utf8",
    maxBuffer: 32 * 1024 * 1024,
  });
  if (res.status !== 0) {
    throw new Error(`${cmd} ${args.join(" ")} failed: ${res.stderr || res.stdout}`);
  }
  return res.stdout;
}

function rustCompress(fixturePath) {
  return JSON.parse(run(rustBin, ["compress", fixturePath]).trim());
}

function rustDecompress(pair) {
  const tmp = path.join(root, "target/cross-lang-pair-rust.json");
  fs.writeFileSync(tmp, JSON.stringify(pair));
  return run(rustBin, ["decompress", tmp]).trim();
}

function pyCompress(text) {
  const script = `
import json, sys
from tokendoo import compress
text = sys.stdin.read()
r = compress(text)
print(json.dumps({"compressed": r.compressed, "dictionary": {"k": r.dictionary.k, "v": r.dictionary.v, "a": r.dictionary.a}}))
`;
  return JSON.parse(run(python, ["-c", script], text).trim());
}

function pyDecompress(pair) {
  const script = `
import json, sys
from tokendoo import decompress, Dictionary
pair = json.loads(sys.stdin.read())
d = Dictionary(k=pair["dictionary"].get("k") or [], v=pair["dictionary"].get("v") or [], a=pair["dictionary"].get("a"))
print(decompress(pair["compressed"], d))
`;
  return run(python, ["-c", script], JSON.stringify(pair)).trim();
}

function valuesEqual(a, b) {
  return JSON.stringify(JSON.parse(a)) === JSON.stringify(JSON.parse(b));
}

function normalizeDict(dict) {
  return {
    k: dict.k || [],
    v: dict.v || [],
    a: dict.a === undefined ? null : dict.a,
  };
}

function dictsEqual(a, b) {
  return JSON.stringify(normalizeDict(a)) === JSON.stringify(normalizeDict(b));
}

const fixtures = fs
  .readdirSync(fixturesDir)
  .filter((name) => name.endsWith(".json"))
  .sort();

let failed = 0;
for (const name of fixtures) {
  const fixturePath = path.join(fixturesDir, name);
  const input = fs.readFileSync(fixturePath, "utf8");
  if (Buffer.byteLength(input, "utf8") > tokendoo.MAX_INPUT_BYTES) {
    process.stdout.write(`skip ${name} (over MAX_INPUT_BYTES)\n`);
    continue;
  }

  try {
    const rustPair = rustCompress(fixturePath);
    const nodePair = (() => {
      const r = tokendoo.compress(input);
      return { compressed: r.compressed, dictionary: r.dictionary };
    })();
    const pyPair = pyCompress(input);

    if (rustPair.compressed !== nodePair.compressed || rustPair.compressed !== pyPair.compressed) {
      throw new Error("compressed output differs across runtimes");
    }
    if (!dictsEqual(rustPair.dictionary, nodePair.dictionary)
      || !dictsEqual(rustPair.dictionary, pyPair.dictionary)) {
      throw new Error("dictionary differs across runtimes");
    }

    const fromRust = {
      node: tokendoo.decompress(rustPair.compressed, rustPair.dictionary),
      py: pyDecompress(rustPair),
      rust: rustDecompress(rustPair),
    };
    const fromNode = {
      rust: rustDecompress(nodePair),
      py: pyDecompress(nodePair),
      node: tokendoo.decompress(nodePair.compressed, nodePair.dictionary),
    };
    const fromPy = {
      rust: rustDecompress(pyPair),
      node: tokendoo.decompress(pyPair.compressed, pyPair.dictionary),
      py: pyDecompress(pyPair),
    };

    for (const [label, out] of Object.entries({ ...prefix("rust", fromRust), ...prefix("node", fromNode), ...prefix("py", fromPy) })) {
      if (!valuesEqual(input, out)) {
        throw new Error(`value mismatch for ${label}`);
      }
    }
    process.stdout.write(`ok ${name}\n`);
  } catch (err) {
    failed += 1;
    process.stderr.write(`FAIL ${name}: ${err.message}\n`);
  }
}

function prefix(source, map) {
  const out = {};
  for (const [k, v] of Object.entries(map)) {
    out[`${source}->${k}`] = v;
  }
  return out;
}

if (failed > 0) {
  process.exit(1);
}
process.stdout.write("cross-language round-trip: all fixtures passed\n");
