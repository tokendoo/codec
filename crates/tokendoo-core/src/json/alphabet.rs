//! Symbol alphabets for structural dictionary encoding.
//!
//! Each alphabet defines how ordinals are rendered as key/value symbols and
//! how the tabular sentinel is spelled. Collision detection is alphabet-specific.

use serde_json::Value;

use crate::json::MAX_DEPTH;

/// Identifies which symbol alphabet the codec uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlphabetKind {
    /// `U+0001` / `U+0002T` / `U+0003` — TOK-001 T1 control alphabet (benchmark baseline).
    Control,
    /// Printable rare ASCII: `~` + base36 keys, `~T` tabular, `^` + base36 values.
    Printable,
    /// Bare base36 ordinals; tabular sentinel `__T`. Selected by TOK-002 (default).
    #[default]
    Bare,
    /// Short static tokens curated for cheap BPE cost; tabular `__T`.
    CommonWord,
}

impl AlphabetKind {
    /// Stable name for benchmarks and docs.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Control => "control",
            Self::Printable => "printable",
            Self::Bare => "bare",
            Self::CommonWord => "common_word",
        }
    }

    /// All candidates measured in TOK-002.
    pub const ALL: [Self; 4] = [
        Self::Control,
        Self::Printable,
        Self::Bare,
        Self::CommonWord,
    ];
}

/// Encode `n` as lowercase base-36.
pub fn to_base36(mut n: usize) -> String {
    const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    if n == 0 {
        return "0".to_string();
    }
    let mut buf = Vec::new();
    while n > 0 {
        buf.push(DIGITS[n % 36]);
        n /= 36;
    }
    buf.reverse();
    String::from_utf8(buf).unwrap_or_else(|_| "0".to_string())
}

/// Parse a lowercase/uppercase base-36 string.
pub fn from_base36(s: &str) -> Option<usize> {
    if s.is_empty() {
        return None;
    }
    let mut n: usize = 0;
    for c in s.chars() {
        let digit = match c {
            '0'..='9' => (c as u8 - b'0') as usize,
            'a'..='z' => (c as u8 - b'a') as usize + 10,
            'A'..='Z' => (c as u8 - b'A') as usize + 10,
            _ => return None,
        };
        n = n.checked_mul(36)?.checked_add(digit)?;
    }
    Some(n)
}

/// Curated short symbols intended to be single cl100k tokens.
/// Indexed ordinals wrap with a numeric suffix when exhausted.
const COMMON_WORDS: &[&str] = &[
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p",
    "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "aa", "ab", "ac", "ad", "ae",
    "af", "ag", "ah", "ai", "aj", "ak", "al", "am", "an", "ao", "ap", "aq", "ar",
    "as", "at", "au", "av", "aw", "ax", "ay", "az", "ba", "bb", "bc", "bd", "be",
];

impl AlphabetKind {
    /// Render a key-dictionary symbol for `ordinal`.
    pub fn key_symbol(self, ordinal: usize) -> String {
        match self {
            Self::Control => format!("\u{0001}{}", to_base36(ordinal)),
            Self::Printable => format!("~{}", to_base36(ordinal)),
            Self::Bare => to_base36(ordinal),
            Self::CommonWord => common_word(ordinal),
        }
    }

    /// Render a value-intern symbol for `ordinal`.
    pub fn value_symbol(self, ordinal: usize) -> String {
        match self {
            Self::Control => format!("\u{0003}{}", to_base36(ordinal)),
            Self::Printable => format!("^{}", to_base36(ordinal)),
            Self::Bare => to_base36(ordinal),
            // Offset into a disjoint half of the table so keys/values do not share
            // the same short token when both transforms run.
            Self::CommonWord => common_word(ordinal.saturating_add(COMMON_WORDS.len() / 2)),
        }
    }

    /// Tabular collapse sentinel string.
    pub fn tabular_sentinel(self) -> &'static str {
        match self {
            Self::Control => "\u{0002}T",
            Self::Printable => "~T",
            Self::Bare | Self::CommonWord => "__T",
        }
    }

    /// Parse a key symbol back to an ordinal, if it matches this alphabet.
    pub fn parse_key_symbol(self, s: &str) -> Option<usize> {
        match self {
            Self::Control => {
                let mut chars = s.chars();
                if chars.next() != Some('\u{0001}') {
                    return None;
                }
                from_base36(chars.as_str())
            }
            Self::Printable => {
                let mut chars = s.chars();
                if chars.next() != Some('~') {
                    return None;
                }
                let rest = chars.as_str();
                if rest == "T" {
                    return None; // tabular sentinel, not a key symbol
                }
                from_base36(rest)
            }
            Self::Bare => {
                if s == "__T" {
                    return None;
                }
                from_base36(s)
            }
            Self::CommonWord => parse_common_word(s),
        }
    }

    /// Parse a value symbol back to an ordinal.
    pub fn parse_value_symbol(self, s: &str) -> Option<usize> {
        match self {
            Self::Control => {
                let mut chars = s.chars();
                if chars.next() != Some('\u{0003}') {
                    return None;
                }
                from_base36(chars.as_str())
            }
            Self::Printable => {
                let mut chars = s.chars();
                if chars.next() != Some('^') {
                    return None;
                }
                from_base36(chars.as_str())
            }
            Self::Bare => {
                if s == "__T" {
                    return None;
                }
                from_base36(s)
            }
            Self::CommonWord => {
                let idx = parse_common_word(s)?;
                idx.checked_sub(COMMON_WORDS.len() / 2)
            }
        }
    }

    /// Estimated UTF-8 length of a key symbol at `ordinal` (for profitability).
    pub fn key_symbol_len(self, ordinal: usize) -> usize {
        self.key_symbol(ordinal).len()
    }

    /// Estimated UTF-8 length of a value symbol at `ordinal`.
    pub fn value_symbol_len(self, ordinal: usize) -> usize {
        self.value_symbol(ordinal).len()
    }

    /// Return `true` if the tree already contains a colliding key for this alphabet.
    pub fn scan_key_collision(self, value: &Value) -> bool {
        scan_keys(value, self, 0)
    }

    /// Return `true` if the tree contains the tabular sentinel as a string.
    pub fn scan_tabular_collision(self, value: &Value) -> bool {
        scan_tabular(value, self.tabular_sentinel(), 0)
    }

    /// Return `true` if the tree contains a colliding string value for interning.
    pub fn scan_value_collision(self, value: &Value) -> bool {
        scan_values(value, self, 0, false)
    }
}

fn common_word(ordinal: usize) -> String {
    if ordinal < COMMON_WORDS.len() {
        COMMON_WORDS[ordinal].to_string()
    } else {
        format!("z{}", to_base36(ordinal - COMMON_WORDS.len()))
    }
}

fn parse_common_word(s: &str) -> Option<usize> {
    if let Some(pos) = COMMON_WORDS.iter().position(|w| *w == s) {
        return Some(pos);
    }
    let rest = s.strip_prefix('z')?;
    let n = from_base36(rest)?;
    Some(n + COMMON_WORDS.len())
}

fn scan_keys(value: &Value, alphabet: AlphabetKind, depth: u32) -> bool {
    if depth > MAX_DEPTH {
        return false;
    }
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                if key_looks_like_symbol(alphabet, k) {
                    return true;
                }
                if scan_keys(v, alphabet, depth + 1) {
                    return true;
                }
            }
            false
        }
        Value::Array(items) => items.iter().any(|v| scan_keys(v, alphabet, depth + 1)),
        _ => false,
    }
}

fn key_looks_like_symbol(alphabet: AlphabetKind, k: &str) -> bool {
    match alphabet {
        AlphabetKind::Control => k.starts_with('\u{0001}'),
        AlphabetKind::Printable => {
            k.starts_with('~') && alphabet.parse_key_symbol(k).is_some()
        }
        AlphabetKind::Bare => {
            // Ambiguous if the key is pure base36 (could be confused with a symbol).
            !k.is_empty()
                && k.chars()
                    .all(|c| c.is_ascii_alphanumeric() && (c.is_ascii_digit() || c.is_ascii_lowercase()))
                && from_base36(k).is_some()
                && k.len() <= 4
        }
        AlphabetKind::CommonWord => {
            COMMON_WORDS.contains(&k) || (k.starts_with('z') && from_base36(&k[1..]).is_some())
        }
    }
}

fn scan_tabular(value: &Value, sentinel: &str, depth: u32) -> bool {
    if depth > MAX_DEPTH {
        return false;
    }
    match value {
        Value::String(s) => s == sentinel,
        Value::Array(items) => items.iter().any(|v| scan_tabular(v, sentinel, depth + 1)),
        Value::Object(map) => map.values().any(|v| scan_tabular(v, sentinel, depth + 1)),
        _ => false,
    }
}

fn scan_values(value: &Value, alphabet: AlphabetKind, depth: u32, in_schema: bool) -> bool {
    if depth > MAX_DEPTH {
        return false;
    }
    let sentinel = alphabet.tabular_sentinel();
    match value {
        Value::String(s) => !in_schema && value_looks_like_symbol(alphabet, s),
        Value::Array(items) => {
            if items.first().and_then(|v| v.as_str()) == Some(sentinel) && items.len() == 3 {
                return items
                    .get(2)
                    .is_some_and(|rows| scan_values(rows, alphabet, depth + 1, false));
            }
            items
                .iter()
                .any(|v| scan_values(v, alphabet, depth + 1, false))
        }
        Value::Object(map) => map
            .values()
            .any(|v| scan_values(v, alphabet, depth + 1, false)),
        _ => false,
    }
}

fn value_looks_like_symbol(alphabet: AlphabetKind, s: &str) -> bool {
    match alphabet {
        AlphabetKind::Control => s.starts_with('\u{0003}'),
        AlphabetKind::Printable => s.starts_with('^') && alphabet.parse_value_symbol(s).is_some(),
        AlphabetKind::Bare => {
            !s.is_empty()
                && s.chars()
                    .all(|c| c.is_ascii_alphanumeric() && (c.is_ascii_digit() || c.is_ascii_lowercase()))
                && from_base36(s).is_some()
                && s.len() <= 4
        }
        AlphabetKind::CommonWord => alphabet.parse_value_symbol(s).is_some(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_roundtrip_symbols() {
        let a = AlphabetKind::Control;
        assert_eq!(a.parse_key_symbol(&a.key_symbol(0)), Some(0));
        assert_eq!(a.parse_key_symbol(&a.key_symbol(40)), Some(40));
        assert_eq!(a.parse_value_symbol(&a.value_symbol(3)), Some(3));
    }

    #[test]
    fn printable_roundtrip_symbols() {
        let a = AlphabetKind::Printable;
        assert_eq!(a.parse_key_symbol(&a.key_symbol(10)), Some(10));
        assert_eq!(a.parse_value_symbol(&a.value_symbol(10)), Some(10));
        assert_eq!(a.tabular_sentinel(), "~T");
    }

    #[test]
    fn bare_and_common_roundtrip() {
        let bare = AlphabetKind::Bare;
        assert_eq!(bare.parse_key_symbol(&bare.key_symbol(15)), Some(15));
        let cw = AlphabetKind::CommonWord;
        assert_eq!(cw.parse_key_symbol(&cw.key_symbol(0)), Some(0));
        assert_eq!(cw.parse_value_symbol(&cw.value_symbol(0)), Some(0));
    }
}
