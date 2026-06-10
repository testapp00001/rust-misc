//! # Lesson 07: Hex Encoding
//!
//! ## What is Hex Encoding?
//!
//! Hex (hexadecimal) encoding represents each byte as two ASCII characters
//! using the characters `0-9` and `a-f` (or `A-F`).
//!
//! ```text
//! Byte 0x4A  →  "4a" (lowercase) or "4A" (uppercase)
//! ```
//!
//! Hex encoding doubles the size: 32 bytes → 64 hex characters.
//!
//! ## Hex vs Base64
//!
//! | Property | Hex | Base64 |
//! |----------|-----|--------|
//! | Size overhead | 2x (100%) | 1.33x (33%) |
//! | Characters | `0-9a-f` | `A-Za-z0-9+/` |
//! | Human readable | Very readable | Less readable |
//! | Use case | Hash display, fingerprints | Data transport |
//!
//! Hex is preferred when humans need to read, compare, or type the values
//! (e.g., hash digests, color codes, memory addresses).
//!
//! ## Security Warning
//!
//! Like Base64, hex encoding is NOT encryption. It's just a different way
//! to display the same bytes. Anyone can decode hex.
//!
//! ## Common Uses
//!
//! - Displaying hash digests: `sha256("hello") = "2cf24dba..."`
//! - Color codes: `#FF5733`
//! - MAC addresses: `00:1A:2B:3C:4D:5E`
//! - Memory addresses and debugging
//! - Cryptographic key fingerprints

/// Exercise 1: Encode bytes to lowercase hex string.
///
/// Hints:
/// - Use `hex::encode(data)` for lowercase hex
/// - Each byte becomes exactly 2 hex characters
pub fn encode_hex(data: &[u8]) -> String {
    todo!("Encode bytes to lowercase hex string")
}

/// Exercise 2: Decode hex string to bytes.
///
/// Returns `None` if the input is not valid hex.
///
/// Hints:
/// - Use `hex::decode(input)` — it returns `Result<Vec<u8>, DecodeError>`
/// - Convert to Option
pub fn decode_hex(encoded: &str) -> Option<Vec<u8>> {
    todo!("Decode hex string to bytes")
}

/// Exercise 3: Encode bytes to uppercase hex string.
///
/// Hints:
/// - Use `hex::encode_upper(data)`
pub fn encode_hex_upper(data: &[u8]) -> String {
    todo!("Encode bytes to uppercase hex string")
}

/// Exercise 4: Decode hex with flexible case handling.
///
/// Should accept both "DEADBEEF" and "deadbeef" and "DeAdBeeF".
///
/// Hints:
/// - `hex::decode()` already handles mixed case — just use it
pub fn decode_hex_flexible(encoded: &str) -> Option<Vec<u8>> {
    todo!("Decode hex with flexible case handling")
}

/// Exercise 5: Display a hash as a human-readable hex string.
///
/// Formats the hash with spaces every 8 characters for readability:
/// "2cf24dba 5fb0a30e 26e83b2a c5b9e29e"
///
/// Hints:
/// - First encode to hex
/// - Then insert spaces every 8 characters
/// - Use `.chars().collect::<Vec<_>>()` and chunk it, or use iterators
pub fn format_hash_display(hash: &[u8]) -> String {
    todo!("Format hash for human-readable display")
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
        // All should decode to the same bytes
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
        assert!(decode_hex("123").is_none()); // odd number of chars
    }

    #[test]
    fn test_binary_data() {
        let data: Vec<u8> = (0..=255).collect();
        let encoded = encode_hex(&data);
        assert_eq!(encoded.len(), 512); // 256 bytes → 512 hex chars
        let decoded = decode_hex(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_format_hash_display() {
        let hash = hex::decode("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824").unwrap();
        let display = format_hash_display(&hash);
        // Should have spaces every 8 hex characters
        assert_eq!(display, "2cf24dba 5fb0a30e 26e83b2a c5b9e29e 1b161e5c 1fa7425e 73043362 938b9824");
    }

    #[test]
    fn test_encode_length() {
        // Each byte → 2 hex chars
        assert_eq!(encode_hex(&[0x00]).len(), 2);
        assert_eq!(encode_hex(&[0xFF]).len(), 2);
        assert_eq!(encode_hex(&[0x00, 0xFF]).len(), 4);
    }
}
