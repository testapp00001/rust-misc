//! # Lesson 06: Base64 Encoding
//!
//! ## What is Base64?
//!
//! Base64 is a way to represent binary data using only 64 printable ASCII characters.
//! It converts every 3 bytes of input into 4 characters of output.
//!
//! ```text
//! Input bytes:  [0x48, 0x69, 0x21]    → "SGkh"
//! ```
//!
//! ## Why Base64?
//!
//! Many systems only handle text safely: email (SMTP), JSON, XML, URLs, HTML.
//! Base64 lets you embed binary data (images, encrypted blobs, keys) in these
//! text-only contexts.
//!
//! ## Standard vs URL-Safe Base64
//!
//! | Variant | Alphabet | Used In |
//! |---------|----------|---------|
//! | Standard | `A-Za-z0-9+/` with `=` padding | PEM, MIME, general use |
//! | URL-safe | `A-Za-z0-9-_` (no padding) | URLs, JWT, filenames |
//!
//! The `+` and `/` characters in standard Base64 are special in URLs, so
//! URL-safe Base64 replaces them with `-` and `_`.
//!
//! ## Padding
//!
//! Standard Base64 uses `=` padding to make the output length a multiple of 4:
//! - 1 byte input → 2 Base64 chars + "=="
//! - 2 byte input → 3 Base64 chars + "="
//! - 3 byte input → 4 Base64 chars (no padding)
//!
//! ## Security Warning
//!
//! Base64 is NOT encryption! It provides ZERO confidentiality. Anyone can
//! decode Base64 data. It's just a format conversion, like hex encoding.
//!
//! Never store passwords or secrets as "Base64 encoded" thinking they're protected.
//!
//! ## Common Uses
//!
//! - Encoding binary data in JSON (e.g., `{"key": "base64data"}`)
//! - HTTP Basic Authentication (`Authorization: Basic base64(user:pass)`)
//! - Email attachments (MIME)
//! - JWT tokens (header.payload.signature)
//! - Embedding images in HTML/CSS (`data:image/png;base64,...`)

use base64::{Engine, engine::general_purpose};

/// Exercise 1: Encode bytes to standard Base64.
///
/// Hints:
/// - Use `general_purpose::STANDARD.encode(data)`
/// - This uses the standard alphabet with `=` padding
pub fn encode_base64(data: &[u8]) -> String {
    todo!("Encode bytes to standard Base64")
}

/// Exercise 2: Decode standard Base64 to bytes.
///
/// Returns `None` if the input is not valid Base64.
///
/// Hints:
/// - Use `general_purpose::STANDARD.decode(input)`
/// - This returns `Result<Vec<u8>, DecodeError>` — convert to Option
pub fn decode_base64(encoded: &str) -> Option<Vec<u8>> {
    todo!("Decode standard Base64 to bytes")
}

/// Exercise 3: Encode bytes to URL-safe Base64 (without padding).
///
/// URL-safe Base64 uses `-` instead of `+` and `_` instead of `/`.
/// No padding characters are appended.
///
/// Hints:
/// - Use `general_purpose::URL_SAFE_NO_PAD.encode(data)`
pub fn encode_base64_url(data: &[u8]) -> String {
    todo!("Encode bytes to URL-safe Base64 without padding")
}

/// Exercise 4: Decode URL-safe Base64 (without padding).
///
/// Hints:
/// - Use `general_purpose::URL_SAFE_NO_PAD.decode(input)`
pub fn decode_base64_url(encoded: &str) -> Option<Vec<u8>> {
    todo!("Decode URL-safe Base64 to bytes")
}

/// Exercise 5: Show that Base64 is NOT encryption.
///
/// This function "encodes" secret data — but anyone can decode it!
/// It's a reminder that encoding != encrypting.
///
/// Hints:
/// - Just encode the data with Base64
/// - The "secret" is visible to anyone who decodes it
pub fn encode_secret_not_encrypted(data: &[u8]) -> String {
    todo!("Show that Base64 encoding is not encryption")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_base64_basic() {
        assert_eq!(encode_base64(b"hello"), "aGVsbG8=");
    }

    #[test]
    fn test_decode_base64_basic() {
        assert_eq!(decode_base64("aGVsbG8=").unwrap(), b"hello");
    }

    #[test]
    fn test_roundtrip() {
        let data = b"Hello, World! This is a test of Base64 encoding.";
        let encoded = encode_base64(data);
        let decoded = decode_base64(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_empty_input() {
        let encoded = encode_base64(b"");
        assert_eq!(encoded, "");
        let decoded = decode_base64("").unwrap();
        assert_eq!(decoded, b"");
    }

    #[test]
    fn test_padding_1_byte() {
        // 1 byte → 2 Base64 chars + "=="
        let encoded = encode_base64(b"A");
        assert_eq!(encoded.len(), 4);
        assert!(encoded.ends_with("=="));
    }

    #[test]
    fn test_padding_2_bytes() {
        // 2 bytes → 3 Base64 chars + "="
        let encoded = encode_base64(b"AB");
        assert_eq!(encoded.len(), 4);
        assert!(encoded.ends_with("="));
    }

    #[test]
    fn test_padding_3_bytes() {
        // 3 bytes → 4 Base64 chars, no padding
        let encoded = encode_base64(b"ABC");
        assert_eq!(encoded.len(), 4);
        assert!(!encoded.contains('='));
    }

    #[test]
    fn test_url_safe_encoding() {
        // Standard Base64 uses + and /; URL-safe uses - and _
        // 0xFB 0xFF 0xFE encodes to "+//+" in standard, "-_-_ " in URL-safe
        let data = [0xFB, 0xFF, 0xFE];
        let standard = encode_base64(&data);
        let url_safe = encode_base64_url(&data);
        assert!(standard.contains('+') || standard.contains('/'));
        assert!(!url_safe.contains('+'));
        assert!(!url_safe.contains('/'));
        assert!(!url_safe.contains('='));
    }

    #[test]
    fn test_url_safe_roundtrip() {
        let data = b"URL-safe Base64 test with special chars: <>&\"'";
        let encoded = encode_base64_url(data);
        let decoded = decode_base64_url(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_decode_invalid() {
        assert!(decode_base64("not!valid!base64!!!").is_none());
    }

    #[test]
    fn test_binary_data() {
        let data: Vec<u8> = (0..=255).collect();
        let encoded = encode_base64(&data);
        let decoded = decode_base64(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_base64_is_not_encryption() {
        let secret = b"password123";
        let encoded = encode_secret_not_encrypted(secret);
        // Anyone can decode this — it's just encoding, not encryption
        let decoded = decode_base64(&encoded).unwrap();
        assert_eq!(decoded, secret);
    }
}
