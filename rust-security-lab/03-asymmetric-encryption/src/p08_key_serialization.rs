//! # Lesson 08: Key Serialization — PEM, DER, and Raw Bytes
//!
//! ## Key Serialization Formats
//!
//! Cryptographic keys must be stored and transmitted. Common formats:
//!
//! | Format | Encoding | Human-readable? | Use case |
//! |--------|----------|-----------------|----------|
//! | Raw bytes | Binary | No | Wire protocol, embedded |
//! | DER | Binary (ASN.1) | No | Certificates, PKCS#8 |
//! | PEM | Base64 + headers | Yes | Config files, SSH |
//! | JWK | JSON | Yes | Web APIs, JWT |
//!
//! ## PEM Format
//!
//! PEM (Privacy Enhanced Mail) is DER-encoded data wrapped in Base64 with headers:
//! ```
//! -----BEGIN PUBLIC KEY-----
//! MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8A...
//! -----END PUBLIC KEY-----
//! ```
//!
//! Common PEM types:
//! - `BEGIN RSA PUBLIC KEY` / `BEGIN RSA PRIVATE KEY` — PKCS#1
//! - `BEGIN PUBLIC KEY` / `BEGIN PRIVATE KEY` — PKCS#8 (preferred)
//! - `BEGIN EC PUBLIC KEY` / `BEGIN EC PRIVATE KEY` — EC keys
//!
//! ## Security Concerns
//!
//! - **Private keys must be encrypted at rest** (PKCS#8 with password)
//! - **Never log or transmit private keys** in plaintext
//! - **Validate format before parsing** — malformed input can cause panics
//! - **Use constant-time parsing** where possible to prevent timing leaks
//!
//! ## Attack Demo: Key Confusion
//!
//! If a system accepts both public and private keys through the same API,
//! an attacker might submit a public key where a private key is expected,
//! causing the system to use weak key material.

use rsa::{RsaPrivateKey, RsaPublicKey};
use rand::rngs::OsRng;

/// Exercise 1: Serialize an RSA public key to raw bytes.
///
/// Returns the public key as a byte vector (DER-encoded PKCS#1 format).
///
/// Hints:
/// - Use `rsa::pkcs1::EncodeRsaPublicKey` trait
/// - Call `public_key.to_pkcs1_der()` to get DER bytes
pub fn serialize_public_key_der(public_key: &RsaPublicKey) -> Vec<u8> {
    todo!("Serialize RSA public key to DER format")
}

/// Exercise 2: Deserialize an RSA public key from DER bytes.
///
/// Hints:
/// - Use `rsa::pkcs1::DecodeRsaPublicKey` trait
/// - Call `RsaPublicKey::from_pkcs1_der(&bytes)`
pub fn deserialize_public_key_der(bytes: &[u8]) -> RsaPublicKey {
    todo!("Deserialize RSA public key from DER format")
}

/// Exercise 3: Serialize an RSA public key to PEM format.
///
/// PEM = Base64(DER) with headers.
///
/// Hints:
/// - Get DER bytes first
/// - Encode with base64
/// - Add "-----BEGIN RSA PUBLIC KEY-----" and "-----END RSA PUBLIC KEY-----" headers
/// - Insert newlines every 64 characters
pub fn serialize_public_key_pem(public_key: &RsaPublicKey) -> String {
    todo!("Serialize RSA public key to PEM format")
}

/// Exercise 4: Deserialize an RSA public key from PEM format.
///
/// Hints:
/// - Strip PEM headers
/// - Decode Base64
/// - Parse DER bytes
pub fn deserialize_public_key_pem(pem: &str) -> RsaPublicKey {
    todo!("Deserialize RSA public key from PEM format")
}

/// Exercise 5: Serialize a private key to PKCS#8 DER format.
///
/// PKCS#8 is the standard format for private keys. It includes the algorithm
/// identifier and the key material.
///
/// Hints:
/// - Use `rsa::pkcs8::EncodePrivateKey` trait
/// - Call `private_key.to_pkcs8_der()` to get DER bytes
pub fn serialize_private_key_der(private_key: &RsaPrivateKey) -> Vec<u8> {
    todo!("Serialize RSA private key to PKCS#8 DER format")
}

/// Exercise 6: Validate that serialized key roundtrips correctly.
///
/// Serialize a key, deserialize it, verify it's the same key.
/// Returns true if roundtrip is successful.
pub fn key_roundtrip_validation() -> bool {
    todo!("Verify key serialization roundtrip")
}

/// Exercise 7: Demonstrate safe key handling (zeroize on drop).
///
/// Show that private key material is cleared from memory when dropped.
pub fn safe_key_handling() {
    todo!("Demonstrate zeroize-on-drop for private keys")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_key_der_roundtrip() {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let public_key = private_key.to_public_key();

        let der = serialize_public_key_der(&public_key);
        let restored = deserialize_public_key_der(&der);

        // The restored key should be able to encrypt (same as original)
        assert_eq!(public_key.size(), restored.size());
    }

    #[test]
    fn test_public_key_pem_roundtrip() {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let public_key = private_key.to_public_key();

        let pem = serialize_public_key_pem(&public_key);
        assert!(pem.contains("-----BEGIN RSA PUBLIC KEY-----"), "PEM should have header");
        assert!(pem.contains("-----END RSA PUBLIC KEY-----"), "PEM should have footer");

        let restored = deserialize_public_key_pem(&pem);
        assert_eq!(public_key.size(), restored.size());
    }

    #[test]
    fn test_private_key_der_roundtrip() {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();

        let der = serialize_private_key_der(&private_key);
        assert!(!der.is_empty(), "DER should not be empty");
        // DER should start with ASN.1 SEQUENCE tag (0x30)
        assert_eq!(der[0], 0x30, "DER should start with SEQUENCE tag");
    }

    #[test]
    fn test_key_roundtrip() {
        assert!(key_roundtrip_validation(), "Key roundtrip should preserve key material");
    }

    #[test]
    fn test_pem_format_structure() {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let public_key = private_key.to_public_key();

        let pem = serialize_public_key_pem(&public_key);
        let lines: Vec<&str> = pem.lines().collect();

        // PEM should have header, content lines, footer
        assert!(lines.len() >= 3, "PEM should have at least header, content, footer");
        assert!(lines[0].starts_with("-----BEGIN"));
        assert!(lines.last().unwrap().starts_with("-----END"));

        // Content lines should be <= 64 chars (Base64 standard)
        for line in &lines[1..lines.len()-1] {
            assert!(line.len() <= 64, "PEM content lines should be <= 64 chars");
        }
    }

    #[test]
    fn test_der_is_binary() {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let public_key = private_key.to_public_key();

        let der = serialize_public_key_der(&public_key);
        // DER is binary, not text — should contain non-ASCII bytes
        assert!(der.iter().any(|&b| b > 127), "DER should contain non-ASCII bytes");
    }

    #[test]
    fn test_invalid_pem_rejected() {
        let result = std::panic::catch_unwind(|| {
            deserialize_public_key_pem("not a valid PEM");
        });
        assert!(result.is_err(), "Invalid PEM should be rejected");
    }

    #[test]
    fn test_safe_key_handling_runs() {
        // Just verify it doesn't panic
        safe_key_handling();
    }
}
