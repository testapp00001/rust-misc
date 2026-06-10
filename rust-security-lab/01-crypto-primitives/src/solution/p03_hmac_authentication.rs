//! # Lesson 03: HMAC — Hash-based Message Authentication Code (Reference Solution)

use ring::hmac;

/// Generate a random HMAC-SHA256 key.
///
/// `ring::hmac::Key::generate` uses the OS CSPRNG to produce a
/// cryptographically secure random key of the correct length.
pub fn generate_hmac_key() -> hmac::Key {
    let rng = ring::rand::SystemRandom::new();
    hmac::Key::generate(hmac::HMAC_SHA256, &rng).expect("Failed to generate HMAC key")
}

/// Compute HMAC-SHA256 tag for a message.
///
/// `hmac::sign` computes HMAC using the key and message, returning a
/// fixed-length tag. The tag is deterministic: same key + message = same tag.
pub fn compute_hmac(key: &hmac::Key, message: &[u8]) -> Vec<u8> {
    hmac::sign(key, message).as_ref().to_vec()
}

/// Verify an HMAC tag against a message.
///
/// `hmac::verify` performs constant-time comparison internally, preventing
/// timing attacks where an attacker measures response time to guess the tag.
pub fn verify_hmac(key: &hmac::Key, message: &[u8], tag: &[u8]) -> bool {
    hmac::verify(key, message, tag).is_ok()
}

/// Compute HMAC and return as hex string.
pub fn hmac_hex(key: &hmac::Key, message: &[u8]) -> String {
    hex::encode(compute_hmac(key, message))
}

/// Create a signed message: (message_bytes, hmac_tag).
pub fn sign_message(key: &hmac::Key, message: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let tag = compute_hmac(key, message);
    (message.to_vec(), tag)
}

/// Verify a signed message. Returns the message if valid, None otherwise.
pub fn verify_signed_message(key: &hmac::Key, message: &[u8], tag: &[u8]) -> Option<Vec<u8>> {
    if verify_hmac(key, message, tag) {
        Some(message.to_vec())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key() {
        let key = generate_hmac_key();
        let tag = compute_hmac(&key, b"test");
        assert_eq!(tag.len(), 32, "HMAC-SHA256 tag should be 32 bytes");
    }

    #[test]
    fn test_compute_and_verify() {
        let key = generate_hmac_key();
        let message = b"Hello, HMAC!";
        let tag = compute_hmac(&key, message);
        assert!(verify_hmac(&key, message, &tag));
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = generate_hmac_key();
        let key2 = generate_hmac_key();
        let message = b"authenticated message";
        let tag = compute_hmac(&key1, message);
        assert!(!verify_hmac(&key2, message, &tag));
    }

    #[test]
    fn test_tampered_message_fails() {
        let key = generate_hmac_key();
        let message = b"original message";
        let tag = compute_hmac(&key, message);
        assert!(!verify_hmac(&key, b"tampered message", &tag));
    }

    #[test]
    fn test_tampered_tag_fails() {
        let key = generate_hmac_key();
        let message = b"authentic message";
        let mut tag = compute_hmac(&key, message);
        tag[0] ^= 0xFF;
        assert!(!verify_hmac(&key, message, &tag));
    }

    #[test]
    fn test_empty_message() {
        let key = generate_hmac_key();
        let tag = compute_hmac(&key, b"");
        assert_eq!(tag.len(), 32);
        assert!(verify_hmac(&key, b"", &tag));
    }

    #[test]
    fn test_long_message() {
        let key = generate_hmac_key();
        let message = vec![0xABu8; 1_000_000];
        let tag = compute_hmac(&key, &message);
        assert!(verify_hmac(&key, &message, &tag));
    }

    #[test]
    fn test_hmac_hex() {
        let key = generate_hmac_key();
        let hex_tag = hmac_hex(&key, b"test");
        assert_eq!(hex_tag.len(), 64);
        assert!(hex_tag.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sign_and_verify_roundtrip() {
        let key = generate_hmac_key();
        let message = b"roundtrip test";
        let (msg, tag) = sign_message(&key, message);
        let result = verify_signed_message(&key, &msg, &tag);
        assert_eq!(result, Some(message.to_vec()));
    }

    #[test]
    fn test_verify_signed_message_tampered() {
        let key = generate_hmac_key();
        let (msg, tag) = sign_message(&key, b"original");
        let mut tampered = msg.clone();
        tampered[0] = b'X';
        assert_eq!(verify_signed_message(&key, &tampered, &tag), None);
    }

    #[test]
    fn test_deterministic_with_same_key() {
        let key = generate_hmac_key();
        let tag1 = compute_hmac(&key, b"deterministic test");
        let tag2 = compute_hmac(&key, b"deterministic test");
        assert_eq!(tag1, tag2);
    }
}
