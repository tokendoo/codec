# tokendoo

Lossless structural compression for JSON that targets **tokens**, not bytes.
This package is the Node.js binding of the same Rust codec published on crates.io
as `tokendoo`.

## Install and compress

```bash
npm install tokendoo
```

```js
const { compress, decompress } = require("tokendoo");

const input = '{"users":[{"id":1,"name":"a"},{"id":2,"name":"b"}]}';
const result = compress(input);
const roundTrip = decompress(result.compressed, result.dictionary);
console.log(JSON.stringify(JSON.parse(input)) === JSON.stringify(JSON.parse(roundTrip)));
```

## Wire-format stability

The current wire format is **T3** (magic bytes in the compressed header). The wire
format is **not stable** before 1.0: it will change (table identity and further
format work are still ahead). npm treats a new `0.x` version as a distinct
release; lockstep with the crates.io and PyPI version is the compatibility
mechanism while the format is in motion.

Do **not** persist compressed payloads beyond the crate version that produced
them. Upgrading and reading old payloads through a different symbol table fails
silently or with decode errors; there is no migration path for stored T3 blobs
across breaking 0.x releases.

## Round-trip guarantee

Round-trip is defined over JSON value equality after parse, not over raw input
bytes. The same rule holds in Rust, Node, and Python: compress in one runtime
and decompress in another must recover equal JSON values, and compressed output
must be byte-identical across the three.

## What this package does not contain

- A browser build. The published artefact targets Node only.
- The token estimator used inside the Rust crate for calibration. Byte counts on
  `CompressResult` are exact; no estimated token figures are exposed here.
