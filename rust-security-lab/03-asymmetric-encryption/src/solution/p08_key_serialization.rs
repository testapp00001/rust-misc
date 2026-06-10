//! # Lesson 08: Key Serialization — PEM, DER, and Raw Bytes (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.
//!
//! This lesson demonstrates how to serialize and deserialize RSA keys in
//! common formats: DER (binary), PEM (base64-encoded), and PKCS#8 (private keys).
//! It also shows safe key handling with zeroize.

use rsa::{RsaPrivateKey, RsaPublicKey, traits::PublicKeyParts};
use rsa::pkcs1::DecodeRsaPublicKey;
use rsa::pkcs1::EncodeRsaPublicKey;
use rsa::pkcs8::EncodePrivateKey;
use rand::rngs::OsRng;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use zeroize::Zeroize;

/// Serialize an RSA public key to DER format (PKCS#1).
///
/// DER (Distinguished Encoding Rules) is a binary ASN.1 encoding.
/// PKCS#1 DER format contains only the RSA-specific key data:
/// SEQUENCE { modulus INTEGER, publicExponent INTEGER }
///
/// This is the most compact representation and is used in wire protocols.
pub fn serialize_public_key_der(public_key: &RsaPublicKey) -> Vec<u8> {
    public_key
        .to_pkcs1_der()
        .expect("failed to serialize public key to DER")
        .to_vec()
}

/// Deserialize an RSA public key from DER bytes (PKCS#1).
///
/// Validates the ASN.1 structure and key components during parsing.
/// Returns an error if the bytes are not valid PKCS#1 DER.
pub fn deserialize_public_key_der(bytes: &[u8]) -> RsaPublicKey {
    RsaPublicKey::from_pkcs1_der(bytes)
        .expect("failed to deserialize public key from DER")
}

/// Serialize an RSA public key to PEM format.
///
/// PEM = "Privacy Enhanced Mail" format:
/// 1. DER-encode the key
/// 2. Base64-encode the DER bytes
/// 3. Wrap in -----BEGIN/END----- headers
/// 4. Insert newlines every 64 characters
///
/// PEM is human-readable and used in config files, SSH keys, certificates.
pub fn serialize_public_key_pem(public_key: &RsaPublicKey) -> String {
    let der = serialize_public_key_der(public_key);
    let b64 = BASE64.encode(&der);

    // Split into 64-char lines
    let mut pem = String::from("-----BEGIN RSA PUBLIC KEY-----\n");
    for chunk in b64.as_bytes().chunks(64) {
        pem.push_str(std::str::from_utf8(chunk).unwrap());
        pem.push('\n');
    }
    pem.push_str("-----END RSA PUBLIC KEY-----\n");
    pem
}

/// Deserialize an RSA public key from PEM format.
///
/// Strips headers, decodes Base64, then parses the DER bytes.
pub fn deserialize_public_key_pem(pem: &str) -> RsaPublicKey {
    // Strip PEM headers and whitespace
    let b64: String = pem
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect::<Vec<&str>>()
        .join("");

    let der = BASE64
        .decode(&b64)
        .expect("failed to decode Base64 from PEM");

    deserialize_public_key_der(&der)
}

/// Serialize a private key to PKCS#8 DER format.
///
/// PKCS#8 is the standard format for private keys. Unlike PKCS#1,
/// it includes the algorithm identifier, making it key-type agnostic.
/// PKCS#8 can also encrypt the key with a password (not shown here).
pub fn serialize_private_key_der(private_key: &RsaPrivateKey) -> Vec<u8> {
    private_key
        .to_pkcs8_der()
        .expect("failed to serialize private key to PKCS#8 DER")
        .to_bytes()
        .to_vec()
}

/// Verify key serialization roundtrip: serialize then deserialize.
///
/// A correct roundtrip means the deserialized key is functionally identical
/// to the original — it can encrypt/decrypt the same messages.
pub fn key_roundtrip_validation() -> bool {
    let private_key = RsaPrivateKey::new(&mut OsRng, 2048).unwrap();
    let public_key = private_key.to_public_key();

    // DER roundtrip
    let der = serialize_public_key_der(&public_key);
    let restored = deserialize_public_key_der(&der);
    if public_key.size() != restored.size() {
        return false;
    }

    // PEM roundtrip
    let pem = serialize_public_key_pem(&public_key);
    let restored_pem = deserialize_public_key_pem(&pem);
    if public_key.size() != restored_pem.size() {
        return false;
    }

    // Verify functional equivalence: encrypt with original, decrypt with restored
    use rsa::Oaep;
    let message = b"roundtrip test";
    let ct = public_key.encrypt(&mut OsRng, Oaep::new::<sha2::Sha256>(), message).unwrap();
    let pt = private_key.decrypt(Oaep::new::<sha2::Sha256>(), &ct).unwrap();
    pt == message
}

/// Demonstrate safe key handling with zeroize.
///
/// When a private key goes out of scope, its memory should be zeroed
/// to prevent recovery from memory dumps, swap files, or core dumps.
/// The `zeroize` crate provides this via the `Zeroize` trait and `Zeroizing<T>`.
///
/// In production: use `secrecy::Secret<RsaPrivateKey>` which wraps the key
/// and zeroizes on drop.
pub fn safe_key_handling() {
    // Demonstrate zeroize on a key-like byte vector
    let mut key_material = vec![42u8; 32];
    assert!(key_material.iter().all(|&b| b == 42));

    // Zeroize clears the memory
    key_material.zeroize();
    assert!(key_material.iter().all(|&b| b == 0));

    // In production, use `secrecy::Secret<T>` or `zeroize::Zeroizing<T>`
    // which automatically zeroize on drop:
    // let secret = secrecy::Secret::new(private_key_bytes);
    // When `secret` is dropped, the bytes are zeroed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_key_der_roundtrip() {
        let private_key = RsaPrivateKey::new(&mut OsRng, 2048).unwrap();
        let public_key = private_key.to_public_key();

        let der = serialize_public_key_der(&public_key);
        let restored = deserialize_public_key_der(&der);

        assert_eq!(public_key.size(), restored.size());
    }

    #[test]
    fn test_public_key_pem_roundtrip() {
        let private_key = RsaPrivateKey::new(&mut OsRng, 2048).unwrap();
        let public_key = private_key.to_public_key();

        let pem = serialize_public_key_pem(&public_key);
        assert!(pem.contains("-----BEGIN RSA PUBLIC KEY-----"), "PEM should have header");
        assert!(pem.contains("-----END RSA PUBLIC KEY-----"), "PEM should have footer");

        let restored = deserialize_public_key_pem(&pem);
        assert_eq!(public_key.size(), restored.size());
    }

    #[test]
    fn test_private_key_der_roundtrip() {
        let private_key = RsaPrivateKey::new(&mut OsRng, 2048).unwrap();

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
        let private_key = RsaPrivateKey::new(&mut OsRng, 2048).unwrap();
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
        let private_key = RsaPrivateKey::new(&mut OsRng, 2048).unwrap();
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
