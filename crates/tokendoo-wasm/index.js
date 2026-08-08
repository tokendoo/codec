"use strict";

const wasm = require("./tokendoo_wasm.js");

module.exports = {
  compress: wasm.compress,
  compress_with_options: wasm.compress_with_options,
  compress_value: wasm.compress_value,
  decompress: wasm.decompress,
  decompress_to_value: wasm.decompress_to_value,
  alphabet_from_flags: wasm.alphabet_from_flags,
  MAX_INPUT_BYTES: wasm.max_input_bytes(),
  MAX_DEPTH: wasm.max_depth(),
  FLAG_KEYS: wasm.flag_keys(),
  FLAG_TABULAR: wasm.flag_tabular(),
  FLAG_VALUES: wasm.flag_values(),
  FLAG_IDENTITY: wasm.flag_identity(),
  FLAG_WRAPPED_STRING: wasm.flag_wrapped_string(),
  FLAG_ALPHABET_MASK: wasm.flag_alphabet_mask(),
};
