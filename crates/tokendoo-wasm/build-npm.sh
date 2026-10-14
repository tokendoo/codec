#!/usr/bin/env bash
# Build the Node npm package into pkg/ and merge packaging metadata.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

wasm-pack build --target nodejs --out-dir pkg --release

cp -f index.js index.d.ts LICENSE NOTICE README.md pkg/

python3 - <<'PY'
import json
import tomllib
from pathlib import Path

pkg = Path("pkg/package.json")
data = json.loads(pkg.read_text())
metadata = json.loads(Path("package.json").read_text())
version = tomllib.loads(Path("../../Cargo.toml").read_text())["workspace"]["package"]["version"]
if metadata["version"] != version:
    raise SystemExit("npm metadata version does not match the Rust workspace")
data.update(metadata)
pkg.write_text(json.dumps(data, indent=2) + "\n")
PY

echo "WASM size (bytes): $(wc -c < pkg/tokendoo_wasm_bg.wasm)"
(cd pkg && npm pack --dry-run)
