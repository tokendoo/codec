#!/usr/bin/env node
"use strict";

const assert = require("assert");
const path = require("path");
const tokendoo = require(process.env.TOKENDOO_NPM || path.join(__dirname, "../pkg"));

function expectKind(fn, kind) {
  let err;
  try {
    fn();
  } catch (e) {
    err = e;
  }
  assert.ok(err, "expected error");
  assert.strictEqual(err.name, "CodecError");
  assert.strictEqual(err.kind, kind);
  return err;
}

const input = '{"users":[{"id":1,"name":"a"},{"id":2,"name":"b"}]}';
const result = tokendoo.compress(input);
assert.strictEqual(typeof result.compressed, "string");
assert.strictEqual(typeof result.flags, "number");
assert.strictEqual(result.flags & tokendoo.FLAG_ALPHABET_MASK, 4 << 5);
const round = tokendoo.decompress(result.compressed, result.dictionary);
assert.deepStrictEqual(JSON.parse(input), JSON.parse(round));

const tooLarge = expectKind(
  () => tokendoo.compress("x".repeat(tokendoo.MAX_INPUT_BYTES + 1)),
  "InputTooLarge",
);
assert.strictEqual(tooLarge.limit, tokendoo.MAX_INPUT_BYTES);
assert.strictEqual(tooLarge.actual, tokendoo.MAX_INPUT_BYTES + 1);

expectKind(() => tokendoo.decompress("not-a-header", { k: [], v: [] }), "MalformedHeader");
expectKind(
  () => tokendoo.compress_with_options("{}", { alphabet: "nope" }),
  "UnsupportedMode",
);

assert.strictEqual(tokendoo.FLAG_KEYS, 1);
assert.strictEqual(tokendoo.FLAG_TABULAR, 2);
assert.strictEqual(tokendoo.FLAG_VALUES, 4);
assert.strictEqual(tokendoo.FLAG_IDENTITY, 8);
assert.strictEqual(tokendoo.FLAG_WRAPPED_STRING, 16);
assert.strictEqual(tokendoo.FLAG_ALPHABET_MASK, 0xe0);
assert.strictEqual(tokendoo.MAX_DEPTH, 64);

console.log("tokendoo-wasm package tests: ok");
