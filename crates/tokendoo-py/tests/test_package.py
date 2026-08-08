#!/usr/bin/env python3
"""Package-level tests run against an installed wheel."""

from __future__ import annotations

import json
import unittest

import tokendoo
from tokendoo import (
    FLAG_ALPHABET_MASK,
    FLAG_IDENTITY,
    FLAG_KEYS,
    FLAG_TABULAR,
    FLAG_VALUES,
    FLAG_WRAPPED_STRING,
    MAX_DEPTH,
    MAX_INPUT_BYTES,
    InputTooLarge,
    MalformedHeader,
    UnsupportedMode,
    compress,
    compress_with_options,
    decompress,
)


class PackageTests(unittest.TestCase):
    def test_round_trip(self) -> None:
        raw = '{"users":[{"id":1,"name":"a"},{"id":2,"name":"b"}]}'
        result = compress(raw)
        out = decompress(result.compressed, result.dictionary)
        self.assertEqual(json.loads(raw), json.loads(out))
        self.assertIsInstance(result.flags, int)
        self.assertIsInstance(result.bytes_before, int)
        self.assertIsInstance(result.bytes_after, int)

    def test_input_too_large(self) -> None:
        with self.assertRaises(InputTooLarge) as ctx:
            compress("x" * (MAX_INPUT_BYTES + 1))
        err = ctx.exception
        self.assertEqual(err.kind, "InputTooLarge")
        self.assertEqual(err.limit, MAX_INPUT_BYTES)
        self.assertEqual(err.actual, MAX_INPUT_BYTES + 1)

    def test_malformed_header(self) -> None:
        with self.assertRaises(MalformedHeader) as ctx:
            decompress("not-a-header", tokendoo.Dictionary())
        self.assertEqual(ctx.exception.kind, "MalformedHeader")

    def test_unsupported_alphabet(self) -> None:
        with self.assertRaises(UnsupportedMode) as ctx:
            compress_with_options("{}", alphabet="nope")
        self.assertEqual(ctx.exception.kind, "UnsupportedMode")

    def test_flag_constants(self) -> None:
        self.assertEqual(FLAG_KEYS, 0x01)
        self.assertEqual(FLAG_TABULAR, 0x02)
        self.assertEqual(FLAG_VALUES, 0x04)
        self.assertEqual(FLAG_IDENTITY, 0x08)
        self.assertEqual(FLAG_WRAPPED_STRING, 0x10)
        self.assertEqual(FLAG_ALPHABET_MASK, 0xE0)
        self.assertEqual(MAX_DEPTH, 64)
        self.assertEqual(MAX_INPUT_BYTES, 256 * 1024)


if __name__ == "__main__":
    unittest.main()
