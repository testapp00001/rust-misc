//! # Lesson 04: ECDH Key Exchange — P-256 Curve
//!
//! ## What is ECDH?
//!
//! Elliptic Curve Diffie-Hellman (ECDH) is the general form of key agreement
//! on elliptic curves. While X25519 uses Curve25519, ECDH can use any curve.
//! P-256 (secp256r1/prime256v1) is the NIST-standardized curve most commonly
//! used in enterprise and government systems.
//!
//! ## P-256 vs X25519
//!
//! | Property | P-256 ECDH | X25519 |
//! |----------|-----------|--------|
//! | Standard | NIST FIPS 186-4 | RFC 7748 |
//! | Key size | 256 bits | 256 bits |
//! | Security level | ~128-bit | ~128-bit |
//! | Constant-time | Requires careful impl | Built-in |
//! | Compliance | FIPS, NIST | Academic/broad adoption |
//! | Speed | Good | Faster |
//!
//! ## When to Use P-256
//!
//! - When FIPS 140-2/140-3 compliance is required
//! - When integrating with systems that only support NIST curves
//! - When using X.509 certificates from traditional CAs
//!
//! ## Attack Demo: Small Subgroup Attack on ECDH
//!
//! If a peer sends a point on a small subgroup (order 1, 2, 3, ...),
//! the shared secret will be one of a few known values.
//! **Defense**: Validate that received points are on the curve and in the correct subgroup.

use p256::{
    ecdh::{EphemeralSecret, SharedSecret},
    PublicKey, EncodedPoint,
};
use rand::rngs::OsRng;

/// Exercise 1: Perform ECDH key exchange using P-256.
///
/// Returns the shared secret computed by both parties.
///
/// Hints:
/// - Generate Alice's secret: `EphemeralSecret::random(&mut OsRng)`
/// - Get Alice's public key: `alice_secret.public_key()`
/// - Do the same for Bob
/// - Alice computes: `alice_secret.diffie_hellman(&bob_public)`
/// - Bob computes: `bob_secret.diffie_hellman(&alice_public)`
/// - Both should produce the same `SharedSecret`
pub fn ecdh_key_exchange() -> SharedSecret {
    todo!("Perform ECDH key exchange using P-256")
}

/// Exercise 2: Verify both parties compute the same shared secret.
///
/// Returns `true` if the shared secrets match.
pub fn verify_ecdh_shared_secret() -> bool {
    todo!("Verify P-256 ECDH produces matching shared secrets")
}

/// Exercise 3: Derive a symmetric key from the ECDH shared secret.
///
/// Use HKDF-SHA256 to derive a 32-byte AES key from the raw shared secret.
///
/// Hints:
/// - Perform ECDH to get the shared secret
/// - Use `ring::hkdf::Salt` with SHA-256
/// - Extract a 32-byte key using `.expand()` and `.fill()`
pub fn derive_ecdh_symmetric_key() -> Vec<u8> {
    todo!("Derive symmetric key from P-256 ECDH shared secret")
}

/// Exercise 4: Serialize and deserialize a P-256 public key.
///
/// Encode a public key to uncompressed point format, then decode it back.
///
/// Hints:
/// - Use `public_key.to_encoded_point(false)` for uncompressed format
/// - Use `.as_bytes()` to get the raw bytes
/// - Decode with `PublicKey::from_sec1_bytes(bytes)`
pub fn serialize_public_key() -> Vec<u8> {
    todo!("Serialize P-256 public key to uncompressed bytes")
}

/// Exercise 5: Demonstrate that a corrupted public key is rejected.
///
/// Modify a serialized public key and verify that deserialization fails.
///
/// Hints:
/// - Serialize a valid public key
/// - Flip a byte
/// - Try to deserialize — should return `Err`
pub fn corrupted_key_rejected() -> bool {
    todo!("Show that corrupted P-256 public keys are rejected")
}

/// Exercise 6: Validate that a point is on the P-256 curve.
///
/// Not all 65-byte sequences are valid P-256 points.
/// This function should return `true` only for valid curve points.
pub fn is_valid_point(bytes: &[u8]) -> bool {
    todo!("Check if bytes represent a valid P-256 point")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecdh_produces_shared_secret() {
        let secret = ecdh_key_exchange();
        // P-256 shared secrets are 32 bytes
        assert_eq!(secret.raw_bytes().len(), 32);
    }

    #[test]
    fn test_ecdh_parties_agree() {
        assert!(verify_ecdh_shared_secret(), "ECDH parties should agree on shared secret");
    }

    #[test]
    fn test_derived_key_length() {
        let key = derive_ecdh_symmetric_key();
        assert_eq!(key.len(), 32, "Derived key should be 32 bytes");
    }

    #[test]
    fn test_public_key_serialization() {
        let bytes = serialize_public_key();
        // Uncompressed P-256 point: 0x04 || x (32 bytes) || y (32 bytes) = 65 bytes
        assert_eq!(bytes.len(), 65, "Uncompressed P-256 point should be 65 bytes");
        assert_eq!(bytes[0], 0x04, "Uncompressed point starts with 0x04");
    }

    #[test]
    fn test_corrupted_key_detected() {
        assert!(corrupted_key_rejected(), "Corrupted public key should be rejected");
    }

    #[test]
    fn test_valid_point_check() {
        // A real P-256 point should validate
        let secret = EphemeralSecret::random(&mut OsRng);
        let pub_key = secret.public_key();
        let encoded = pub_key.to_encoded_point(false);
        assert!(is_valid_point(encoded.as_bytes()), "Valid point should pass validation");
    }

    #[test]
    fn test_invalid_point_rejected() {
        // A random 65 bytes is almost certainly not on the curve
        let fake_point = vec![0x04u8; 65];
        // This might fail or succeed depending on the random bytes — but 0x04 repeated is not on curve
        let result = is_valid_point(&fake_point);
        assert!(!result, "Repeated bytes should not be a valid curve point");
    }

    #[test]
    fn test_two_independent_exchanges_differ() {
        let secret1 = ecdh_key_exchange();
        let secret2 = ecdh_key_exchange();
        // With overwhelming probability, two random exchanges produce different secrets
        assert_ne!(secret1.raw_bytes(), secret2.raw_bytes(),
            "Independent ECDH exchanges should produce different secrets");
    }
}
