/// Handle SIE character encoding.
/// SIE files use CP437 (IBM PC) encoding per the specification,
/// though many modern programs output ISO 8859-1 (Latin-1) or UTF-8.

/// Attempt to decode SIE file bytes to UTF-8 string.
/// Tries UTF-8 first, then falls back to ISO 8859-1 (Latin-1).
/// CP437 is mostly a superset of ASCII for printable chars,
/// and ISO 8859-1 covers Swedish characters (åäö/ÅÄÖ).
pub fn decode_sie_bytes(bytes: &[u8]) -> String {
    // Try UTF-8 first (many modern programs use it)
    if let Ok(s) = std::str::from_utf8(bytes) {
        return s.to_string();
    }

    // Fall back to ISO 8859-1 (Latin-1) — each byte maps directly to a Unicode codepoint
    bytes.iter().map(|&b| b as char).collect()
}

/// Encode a UTF-8 string to CP437/Latin-1 bytes for SIE export.
/// We use ISO 8859-1 which covers all Swedish characters.
pub fn encode_to_latin1(s: &str) -> Vec<u8> {
    s.chars()
        .map(|c| {
            let cp = c as u32;
            if cp <= 0xFF {
                cp as u8
            } else {
                b'?' // Replace unmappable characters
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8_passthrough() {
        let input = "Försäljning tjänster".as_bytes();
        let result = decode_sie_bytes(input);
        assert_eq!(result, "Försäljning tjänster");
    }

    #[test]
    fn test_latin1_decode() {
        // "Försäljning" in Latin-1: F=0x46, ö=0xF6, r=0x72, s=0x73, ä=0xE4, l=0x6C, j=0x6A, n=0x6E, i=0x69, n=0x6E, g=0x67
        let latin1_bytes: Vec<u8> = vec![
            0x46, 0xF6, 0x72, 0x73, 0xE4, 0x6C, 0x6A, 0x6E, 0x69, 0x6E, 0x67,
        ];
        let result = decode_sie_bytes(&latin1_bytes);
        assert_eq!(result, "Försäljning");
    }

    #[test]
    fn test_latin1_encode_roundtrip() {
        let original = "Årsredovisning för företaget";
        let encoded = encode_to_latin1(original);
        let decoded = decode_sie_bytes(&encoded);
        assert_eq!(decoded, original);
    }
}
