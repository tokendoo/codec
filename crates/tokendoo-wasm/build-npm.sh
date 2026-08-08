#!/usr/bin/env bash
# Build the Node npm package into pkg/ and merge packaging metadata.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

wasm-pack build --target nodejs --out-dir pkg --release

cp -f index.js index.d.ts LICENSE NOTICE README.md pkg/

python3 - <<'PY'
import json
from pathlib import Path

pkg = Path("pkg/package.json")
data = json.loads(pkg.read_text())
data["name"] = "tokendoo"
data["version"] = "0.1.0"
data["description"] = (
    "Lossless structural JSON codec for Tokendoo (Node, WebAssembly)"
)
data["main"] = "index.js"
data["types"] = "index.d.ts"
data["license"] = "Apache-2.0"
data["files"] = [
    "index.js",
    "index.d.ts",
    "tokendoo_wasm_bg.wasm",
    "tokendoo_wasm.js",
    "tokendoo_wasm.d.ts",
    "tokendoo_wasm_bg.wasm.d.ts",
    "LICENSE",
    "NOTICE",
    "README.md",
]
data["sideEffects"] = ["./tokendoo_wasm.js", "./index.js"]
pkg.write_text(json.dumps(data, indent=2) + "\n")
PY

echo "WASM size (bytes): $(wc -c < pkg/tokendoo_wasm_bg.wasm)"
(cd pkg && npm pack --dry-run)
