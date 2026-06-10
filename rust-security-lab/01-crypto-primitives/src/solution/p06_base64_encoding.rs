//! # Lesson 06: Base64 Encoding (Reference Solution)

use base64::{Engine, engine::general_purpose};

/// Encode bytes to standard Base64 with padding.
pub fn encode_base64(data: &[u8]) -> String {
    general_purpose::STANDARD.encode(data)
}

/// Decode standard Base64 to bytes.
/// Returns None for invalid input.
pub fn decode_base64(encoded: &str) -> Option<Vec<u8>> {
    general_purpose::STANDARD.decode(encoded).ok()
}

/// Encode bytes to URL-safe Base64 without padding.
///
/// URL-safe uses `-` instead of `+`, `_` instead of `/`, and omits `=`.
/// This is the standard encoding for JWT tokens and URL parameters.
pub fn encode_base64_url(data: &[u8]) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(data)
}

/// Decode URL-safe Base64 without padding.
pub fn decode_base64_url(encoded: &str) -> Option<Vec<u8>> {
    general_purpose::URL_SAFE_NO_PAD.decode(encoded).ok()
}

/// "Encode" a secret — demonstrates that Base64 is NOT encryption.
///
/// Anyone who sees this output can decode it and recover the original data.
/// For actual confidentiality, use encryption (AES-GCM, ChaCha20-Poly1305).
pub fn encode_secret_not_encrypted(data: &[u8]) -> String {
    general_purpose::STANDARD.encode(data)
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
        let encoded = encode_base64(b"A");
        assert_eq!(encoded.len(), 4);
        assert!(encoded.ends_with("=="));
    }

    #[test]
    fn test_padding_2_bytes() {
        let encoded = encode_base64(b"AB");
        assert_eq!(encoded.len(), 4);
        assert!(encoded.ends_with("="));
    }

    #[test]
    fn test_padding_3_bytes() {
        let encoded = encode_base64(b"ABC");
        assert_eq!(encoded.len(), 4);
        assert!(!encoded.contains('='));
    }

    #[test]
    fn test_url_safe_encoding() {
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
        let decoded = decode_base64(&encoded).unwrap();
        assert_eq!(decoded, secret);
    }
}
