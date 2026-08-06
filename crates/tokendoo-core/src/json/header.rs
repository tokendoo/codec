//! Header encode/decode for the Tokendoo wire format.

use crate::json::alphabet::AlphabetKind;
use crate::json::CodecError;

/// Total length of the fixed header prefix `"T?" + flags + ":"`.
pub const HEADER_LEN: usize = 5;

/// Current wire-format magic (bumped T1 → T2 when default alphabet became bare).
pub const WIRE_MAGIC: &[u8; 2] = b"T2";

/// Key dictionary applied.
pub const FLAG_KEYS: u8 = 0x01;
/// Tabular collapse applied at least once.
pub const FLAG_TABULAR: u8 = 0x02;
/// Value interning applied.
pub const FLAG_VALUES: u8 = 0x04;
/// Identity fallback; exclusive with other transform bits.
pub const FLAG_IDENTITY: u8 = 0x08;
/// Original payload was a JSON string wrapping JSON text.
pub const FLAG_WRAPPED_STRING: u8 = 0x10;
/// Alphabet id occupies bits 5–6 (`0x20 | 0x40`).
pub const FLAG_ALPHABET_MASK: u8 = 0x60;
/// Bit 7 remains reserved.
const RESERVED_MASK: u8 = 0x80;

/// Encode an [`AlphabetKind`] into flag bits 5–6.
pub fn alphabet_flags(kind: AlphabetKind) -> u8 {
    let id: u8 = match kind {
        AlphabetKind::Control => 0,
        AlphabetKind::Printable => 1,
        AlphabetKind::Bare => 2,
        AlphabetKind::CommonWord => 3,
    };
    id << 5
}

/// Decode alphabet from flag bits 5–6.
pub fn alphabet_from_flags(flags: u8) -> AlphabetKind {
    match (flags & FLAG_ALPHABET_MASK) >> 5 {
        0 => AlphabetKind::Control,
        1 => AlphabetKind::Printable,
        2 => AlphabetKind::Bare,
        _ => AlphabetKind::CommonWord,
    }
}

/// Encode `flags` and `body` into a wire string using [`WIRE_MAGIC`].
pub fn encode_header(flags: u8, body: &str) -> String {
    encode_header_with_magic(WIRE_MAGIC, flags, body)
}

/// Encode with an explicit magic (used when bumping T1 → T2).
pub fn encode_header_with_magic(magic: &[u8; 2], flags: u8, body: &str) -> String {
    let mut out = String::with_capacity(HEADER_LEN + body.len());
    out.push(char::from(magic[0]));
    out.push(char::from(magic[1]));
    out.push(hex_digit(flags >> 4));
    out.push(hex_digit(flags & 0x0F));
    out.push(':');
    out.push_str(body);
    out
}

/// Split a wire string into `(flags, body)`.
///
/// Accepts the current [`WIRE_MAGIC`]. Older/newer magics yield [`CodecError::MalformedHeader`].
pub fn parse_header(compressed: &str) -> Result<(u8, &str), CodecError> {
    let bytes = compressed.as_bytes();
    if bytes.len() < HEADER_LEN {
        return Err(CodecError::MalformedHeader);
    }
    if bytes[0] != WIRE_MAGIC[0] || bytes[1] != WIRE_MAGIC[1] || bytes[4] != b':' {
        return Err(CodecError::MalformedHeader);
    }
    let hi = from_hex(bytes[2])?;
    let lo = from_hex(bytes[3])?;
    let flags = (hi << 4) | lo;
    if flags & RESERVED_MASK != 0 {
        return Err(CodecError::MalformedHeader);
    }
    Ok((flags, &compressed[HEADER_LEN..]))
}

fn hex_digit(nibble: u8) -> char {
    match nibble {
        0..=9 => char::from(b'0' + nibble),
        10..=15 => char::from(b'a' + (nibble - 10)),
        _ => '0',
    }
}

fn from_hex(b: u8) -> Result<u8, CodecError> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(CodecError::MalformedHeader),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_header() {
        let encoded = encode_header(FLAG_KEYS | FLAG_VALUES, "{\"a\":1}");
        let (flags, body) = parse_header(&encoded).expect("header");
        assert_eq!(flags, FLAG_KEYS | FLAG_VALUES);
        assert_eq!(body, "{\"a\":1}");
    }

    #[test]
    fn alphabet_bits_roundtrip() {
        for kind in AlphabetKind::ALL {
            let flags = alphabet_flags(kind) | FLAG_KEYS;
            assert_eq!(alphabet_from_flags(flags), kind);
            let encoded = encode_header(flags, "{}");
            let (parsed, _) = parse_header(&encoded).unwrap();
            assert_eq!(alphabet_from_flags(parsed), kind);
        }
    }
}
