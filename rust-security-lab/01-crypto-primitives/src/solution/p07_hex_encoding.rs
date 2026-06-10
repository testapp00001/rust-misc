//! # Lesson 07: Hex Encoding (Reference Solution)

/// Encode bytes to lowercase hex string.
pub fn encode_hex(data: &[u8]) -> String {
    hex::encode(data)
}

/// Decode hex string to bytes.
/// Returns None for invalid input (bad characters, odd length).
pub fn decode_hex(encoded: &str) -> Option<Vec<u8>> {
    hex::decode(encoded).ok()
}

/// Encode bytes to uppercase hex string.
pub fn encode_hex_upper(data: &[u8]) -> String {
    hex::encode_upper(data)
}

/// Decode hex with flexible case handling.
/// `hex::decode` already accepts mixed case input.
pub fn decode_hex_flexible(encoded: &str) -> Option<Vec<u8>> {
    hex::decode(encoded).ok()
}

/// Format a hash as a human-readable hex string with spaces every 8 chars.
///
/// This makes long hashes easier to read and compare visually:
/// `2cf24dba 5fb0a30e 26e83b2a c5b9e29e ...`
pub fn format_hash_display(hash: &[u8]) -> String {
    hex::encode(hash)
        .as_bytes()
        .chunks(8)
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_hex_basic() {
        assert_eq!(encode_hex(b"hello"), "68656c6c6f");
    }

    #[test]
    fn test_decode_hex_basic() {
        assert_eq!(decode_hex("68656c6c6f").unwrap(), b"hello");
    }

    #[test]
    fn test_roundtrip() {
        let data = b"Hello, hex encoding!";
        let encoded = encode_hex(data);
        let decoded = decode_hex(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(encode_hex(b""), "");
        assert_eq!(decode_hex("").unwrap(), b"");
    }

    #[test]
    fn test_encode_uppercase() {
        assert_eq!(encode_hex_upper(b"hello"), "68656C6C6F");
    }

    #[test]
    fn test_decode_mixed_case() {
        let lower = decode_hex_flexible("deadbeef").unwrap();
        let upper = decode_hex_flexible("DEADBEEF").unwrap();
        let mixed = decode_hex_flexible("DeAdBeeF").unwrap();
        assert_eq!(lower, upper);
        assert_eq!(upper, mixed);
        assert_eq!(lower, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_decode_invalid_hex() {
        assert!(decode_hex("not_hex").is_none());
        assert!(decode_hex("xyz").is_none());
        assert!(decode_hex("123").is_none());
    }

    #[test]
    fn test_binary_data() {
        let data: Vec<u8> = (0..=255).collect();
        let encoded = encode_hex(&data);
        assert_eq!(encoded.len(), 512);
        let decoded = decode_hex(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_format_hash_display() {
        let hash = hex::decode("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824").unwrap();
        let display = format_hash_display(&hash);
        assert_eq!(display, "2cf24dba 5fb0a30e 26e83b2a c5b9e29e 1b161e5c 1fa7425e 73043362 938b9824");
    }

    #[test]
    fn test_encode_length() {
        assert_eq!(encode_hex(&[0x00]).len(), 2);
        assert_eq!(encode_hex(&[0xFF]).len(), 2);
        assert_eq!(encode_hex(&[0x00, 0xFF]).len(), 4);
    }
}
