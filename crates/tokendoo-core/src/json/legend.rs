//! Explanatory legend clauses counted in the identity size guard.
//!
//! A legend that teaches the model to read the compressed form is part of what
//! the client transmits. The codec's identity fallback therefore compares
//! **prompt** sizes (wire framing + dictionary + legend text), not wire sizes
//! alone. Omitting the legend from that decision compresses payloads on which
//! the client ends up paying more — the same measurement-boundary defect class
//! as omitting the dictionary (ADR-008).

use crate::json::header::{FLAG_IDENTITY, FLAG_TABULAR, FLAG_VALUES};

/// Explains the framing header. Needed by every non-original condition,
/// identity included, because the framing is transmitted there too.
pub const CLAUSE_HEADER: &str = "The DATA block is a Tokendoo-compressed JSON document. \
It begins with `T3`, two hexadecimal digits and a colon; that prefix is a format header and \
carries no data.";

/// Explains tabular collapse.
pub const CLAUSE_TABULAR: &str = "Any array shaped like [\"__T\", [name1, name2, ...], \
[[value1, value2, ...], ...]] is a collapsed table: the second element lists the field names, \
and each array inside the third element is one record whose values appear in that same order.";

/// Explains value interning against the dictionary.
pub const CLAUSE_DICTIONARY: &str = "A DICTIONARY block follows the data. Inside the document, \
a short string symbol (for example \"0\", \"a\", or another single-token spelling from the \
active alphabet) stands for the entry at that zero-based position in the dictionary's \"v\" \
array. Resolve such a string to its dictionary entry before answering.";

/// Build the legend text required to read a payload emitted with `flags`.
///
/// The identity form still carries the framing header, so it still needs the
/// header clause: pretending otherwise would hide a real cost of the framing.
pub fn legend_text_for_flags(flags: u8) -> String {
    let mut parts: Vec<&str> = vec![CLAUSE_HEADER];
    if flags & FLAG_IDENTITY == 0 {
        if flags & FLAG_TABULAR != 0 {
            parts.push(CLAUSE_TABULAR);
        }
        if flags & FLAG_VALUES != 0 {
            parts.push(CLAUSE_DICTIONARY);
        }
    }
    parts.join("\n")
}

/// UTF-8 byte length of [`legend_text_for_flags`].
pub fn legend_bytes_for_flags(flags: u8) -> usize {
    legend_text_for_flags(flags).len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::header::FLAG_KEYS;

    #[test]
    fn identity_carries_only_the_header_clause() {
        let text = legend_text_for_flags(FLAG_IDENTITY);
        assert_eq!(text, CLAUSE_HEADER);
        assert!(!text.contains("__T"));
        assert!(!text.contains("DICTIONARY block"));
    }

    #[test]
    fn transform_flags_add_their_clauses() {
        let both = legend_text_for_flags(FLAG_TABULAR | FLAG_VALUES);
        assert!(both.contains("__T"));
        assert!(both.contains("DICTIONARY block"));
        assert!(both.len() > legend_text_for_flags(FLAG_IDENTITY).len());
    }

    #[test]
    fn key_dictionary_flag_adds_no_clause_today() {
        let with_keys = legend_bytes_for_flags(FLAG_KEYS | FLAG_TABULAR | FLAG_VALUES);
        let without = legend_bytes_for_flags(FLAG_TABULAR | FLAG_VALUES);
        assert_eq!(with_keys, without);
    }
}
