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

/// Longest bare ordinal the codec may write or read.
///
/// Four base-36 digits address [`BARE_ORDINAL_CAPACITY`] symbols, which is far
/// beyond what a payload under [`crate::json::MAX_INPUT_BYTES`] can require.
pub const BARE_SYMBOL_MAX_LEN: usize = 4;

/// Number of ordinals the bare alphabet can express, `36^4`.
pub const BARE_ORDINAL_CAPACITY: usize = 36 * 36 * 36 * 36;

/// The exact set of strings the bare alphabet interprets as an ordinal.
///
/// Both directions go through this function, and that is the point. The bare
/// alphabet has no prefix to separate a symbol from ordinary content, so the
/// collision detector and the decoder must agree on the same set of strings,
/// character for character. When they disagreed, a literal outside the
/// detector's window survived interning and was then decoded as a dictionary
/// reference: either a hard [`crate::json::CodecError::DictionaryIndexOutOfRange`],
/// or — when the ordinal landed inside the table — a different string returned
/// with no error at all.
///
/// [`to_base36`] emits lowercase, and every ordinal below
/// [`BARE_ORDINAL_CAPACITY`] fits in [`BARE_SYMBOL_MAX_LEN`] characters, so the
/// encoder's output is contained in this set and this function is its exact
/// inverse. The tabular sentinel `__T` is excluded for free: `_` is not a
/// base-36 digit.
pub fn bare_ordinal(s: &str) -> Option<usize> {
    if s.is_empty() || s.len() > BARE_SYMBOL_MAX_LEN {
        return None;
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase())
    {
        return None;
    }
    from_base36(s)
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
            Self::Bare => bare_ordinal(s),
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
            Self::Bare => bare_ordinal(s),
            Self::CommonWord => {
                let idx = parse_common_word(s)?;
                idx.checked_sub(COMMON_WORDS.len() / 2)
            }
        }
    }

    /// Number of ordinals this alphabet can express, when it is bounded.
    ///
    /// Bounded only for [`AlphabetKind::Bare`], whose symbols carry no prefix
    /// and are therefore capped at [`BARE_SYMBOL_MAX_LEN`] characters so that a
    /// symbol stays distinguishable from ordinary content. The prefixed
    /// alphabets can grow a symbol indefinitely without ambiguity.
    pub const fn ordinal_capacity(self) -> Option<usize> {
        match self {
            Self::Bare => Some(BARE_ORDINAL_CAPACITY),
            Self::Control | Self::Printable | Self::CommonWord => None,
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
        // Ambiguous exactly when the decoder would read the key as an ordinal.
        AlphabetKind::Bare => bare_ordinal(k).is_some(),
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
        // Ambiguous exactly when the decoder would read the value as an ordinal.
        AlphabetKind::Bare => bare_ordinal(s).is_some(),
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

    #[test]
    fn every_bare_symbol_the_encoder_writes_is_readable() {
        let bare = AlphabetKind::Bare;
        for ordinal in [0, 1, 35, 36, 1_295, 1_296, BARE_ORDINAL_CAPACITY - 1] {
            let symbol = bare.value_symbol(ordinal);
            assert!(
                symbol.len() <= BARE_SYMBOL_MAX_LEN,
                "ordinal {ordinal} rendered as {symbol}, longer than the readable window"
            );
            assert_eq!(bare.parse_value_symbol(&symbol), Some(ordinal));
            assert_eq!(bare.parse_key_symbol(&bare.key_symbol(ordinal)), Some(ordinal));
        }
    }

    #[test]
    fn bare_capacity_is_the_boundary_the_guard_protects() {
        // One past the capacity needs five characters, which the decoder must
        // not accept; the transforms therefore fall back to identity rather
        // than emitting it. See the capacity guard in `intern` and `keys`.
        assert_eq!(to_base36(BARE_ORDINAL_CAPACITY).len(), 5);
        assert_eq!(bare_ordinal(&to_base36(BARE_ORDINAL_CAPACITY)), None);
        assert_eq!(
            AlphabetKind::Bare.ordinal_capacity(),
            Some(BARE_ORDINAL_CAPACITY)
        );
    }

    #[test]
    fn bare_collision_detector_and_decoder_accept_the_same_strings() {
        // The regression this pins: the detector used to accept only lowercase
        // strings of at most four characters while the decoder accepted
        // uppercase and any length, so a literal like "ZZZZZ" or "abcdef"
        // passed the detector and was then read as a dictionary reference.
        let bare = AlphabetKind::Bare;
        let probes = [
            "0", "9", "a", "z", "10", "zzzz", "__T", "ZZZZZ", "abcdef", "A", "AB12", "Z", "0Z",
            "a-b", "a_b", "", "hello", "1234", "12345", "café",
        ];
        for probe in probes {
            let decoder_reads_it = bare.parse_value_symbol(probe).is_some();
            assert_eq!(
                decoder_reads_it,
                value_looks_like_symbol(bare, probe),
                "value detector and decoder disagree on {probe:?}"
            );
            assert_eq!(
                bare.parse_key_symbol(probe).is_some(),
                key_looks_like_symbol(bare, probe),
                "key detector and decoder disagree on {probe:?}"
            );
        }
    }

    #[test]
    fn bare_detector_and_decoder_agree_over_every_boundary_string() {
        // The probe list above names the classes; this enumerates them. The
        // charset holds both ends of each base-36 range, both ends of the
        // uppercase range the old decoder wrongly accepted, and two characters
        // that are not base-36 digits at all. Lengths run one past
        // `BARE_SYMBOL_MAX_LEN`, so every string that sits on the readable
        // window's edge is covered.
        const CHARSET: &[char] = &['0', '9', 'a', 'z', 'A', 'Z', '_', '-'];
        let bare = AlphabetKind::Bare;
        let mut current: Vec<String> = vec![String::new()];
        let mut checked = 0_usize;
        for _ in 1..=BARE_SYMBOL_MAX_LEN + 1 {
            let mut next = Vec::with_capacity(current.len() * CHARSET.len());
            for prefix in &current {
                for c in CHARSET {
                    let candidate = format!("{prefix}{c}");
                    assert_eq!(
                        bare.parse_value_symbol(&candidate).is_some(),
                        value_looks_like_symbol(bare, &candidate),
                        "value detector and decoder disagree on {candidate:?}"
                    );
                    assert_eq!(
                        bare.parse_key_symbol(&candidate).is_some(),
                        key_looks_like_symbol(bare, &candidate),
                        "key detector and decoder disagree on {candidate:?}"
                    );
                    checked += 1;
                    next.push(candidate);
                }
            }
            current = next;
        }
        assert_eq!(checked, 37_448, "the enumeration lost strings");
    }

    #[test]
    fn bare_leaves_content_that_is_not_a_symbol_alone() {
        let bare = AlphabetKind::Bare;
        for literal in ["ZZZZZ", "abcdef", "A", "AB12", "__T", "a_b", "12345"] {
            assert_eq!(
                bare.parse_value_symbol(literal),
                None,
                "{literal:?} must not be read as an ordinal"
            );
        }
    }
}
