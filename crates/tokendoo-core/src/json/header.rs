//! Header encode/decode for the Tokendoo wire format.

use crate::json::CodecError;

/// Total length of the fixed header prefix `"T1" + flags + ":"`.
pub const HEADER_LEN: usize = 5;

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

const RESERVED_MASK: u8 = 0xE0;

/// Encode `flags` and `body` into a wire string.
pub fn encode_header(flags: u8, body: &str) -> String {
    let mut out = String::with_capacity(HEADER_LEN + body.len());
    out.push_str("T1");
    out.push(hex_digit(flags >> 4));
    out.push(hex_digit(flags & 0x0F));
    out.push(':');
    out.push_str(body);
    out
}

/// Split a wire string into `(flags, body)`.
pub fn parse_header(compressed: &str) -> Result<(u8, &str), CodecError> {
    let bytes = compressed.as_bytes();
    if bytes.len() < HEADER_LEN {
        return Err(CodecError::MalformedHeader);
    }
    if bytes[0] != b'T' || bytes[1] != b'1' || bytes[4] != b':' {
        return Err(CodecError::MalformedHeader);
    }
    let hi = from_hex(bytes[2])?;
    let lo = from_hex(bytes[3])?;
    let flags = (hi << 4) | lo;
    if flags & RESERVED_MASK != 0 {
        return Err(CodecError::MalformedHeader);
    }
    // SAFETY: HEADER_LEN is within UTF-8 char boundaries (ASCII prefix).
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
}
