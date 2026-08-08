export interface Dictionary {
  k?: string[];
  v?: string[];
  a?: string;
}

export interface CompressOptions {
  key_dictionary?: boolean;
  tabular?: boolean;
  intern_values?: boolean;
  alphabet?: string;
}

export interface CompressResult {
  compressed: string;
  dictionary: Dictionary;
  bytes_before: number;
  bytes_after: number;
  identity: boolean;
  flags: number;
}

export class CodecError extends Error {
  name: "CodecError";
  kind: string;
  line?: number;
  column?: number;
  limit?: number;
  actual?: number;
  expected?: number;
}

export const MAX_INPUT_BYTES: number;
export const MAX_DEPTH: number;
export const FLAG_KEYS: number;
export const FLAG_TABULAR: number;
export const FLAG_VALUES: number;
export const FLAG_IDENTITY: number;
export const FLAG_WRAPPED_STRING: number;
export const FLAG_ALPHABET_MASK: number;

export function alphabet_from_flags(flags: number): string;
export function compress(input: string): CompressResult;
export function compress_with_options(
  input: string,
  options?: CompressOptions,
): CompressResult;
export function compress_value(value: unknown): CompressResult;
export function decompress(compressed: string, dictionary: Dictionary): string;
export function decompress_to_value(
  compressed: string,
  dictionary: Dictionary,
): unknown;
