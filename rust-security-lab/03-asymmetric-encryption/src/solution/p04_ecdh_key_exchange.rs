//! # Lesson 04: ECDH Key Exchange — P-256 Curve (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use p256::{
    ecdh::{EphemeralSecret, SharedSecret},
    PublicKey,
};
use p256::elliptic_curve::sec1::ToEncodedPoint;
use ring::hkdf;
use rand::rngs::OsRng;

/// Perform ECDH key exchange using P-256.
///
/// P-256 (secp256r1) is the NIST-standardized curve used in FIPS environments.
/// The ECDH protocol is identical to X25519 conceptually, but on a different curve.
///
/// # Security Notes
/// - P-256 requires careful implementation to avoid timing side-channels
/// - The `p256` crate handles this correctly
/// - Always validate received public keys (see p09_key_validation)
pub fn ecdh_key_exchange() -> SharedSecret {
    let alice_secret = EphemeralSecret::random(&mut OsRng);
    let alice_public = alice_secret.public_key();

    let bob_secret = EphemeralSecret::random(&mut OsRng);
    let bob_public = bob_secret.public_key();

    let alice_shared = alice_secret.diffie_hellman(&bob_public);
    let bob_shared = bob_secret.diffie_hellman(&alice_public);

    // Verify both parties agree
    assert_eq!(alice_shared.raw_secret_bytes(), bob_shared.raw_secret_bytes());

    alice_shared
}

/// Verify both parties compute the same shared secret.
pub fn verify_ecdh_shared_secret() -> bool {
    let alice_secret = EphemeralSecret::random(&mut OsRng);
    let alice_public = alice_secret.public_key();

    let bob_secret = EphemeralSecret::random(&mut OsRng);
    let bob_public = bob_secret.public_key();

    let alice_shared = alice_secret.diffie_hellman(&bob_public);
    let bob_shared = bob_secret.diffie_hellman(&alice_public);

    alice_shared.raw_secret_bytes() == bob_shared.raw_secret_bytes()
}

/// Derive a symmetric key from the ECDH shared secret using HKDF.
///
/// The raw ECDH shared secret is a point on the curve (x-coordinate).
/// We use HKDF to derive a uniformly random key suitable for AES-256.
pub fn derive_ecdh_symmetric_key() -> Vec<u8> {
    let alice_secret = EphemeralSecret::random(&mut OsRng);

    let bob_secret = EphemeralSecret::random(&mut OsRng);
    let bob_public = bob_secret.public_key();

    let shared = alice_secret.diffie_hellman(&bob_public);

    // Derive key using HKDF-SHA256
    let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, b"ecdh-p256-session");
    let prk = salt.extract(shared.raw_secret_bytes().as_slice());
    let okm = prk.expand(&[b"aes-256-gcm-key"], hkdf::HKDF_SHA256).unwrap();
    let mut key = vec![0u8; 32];
    okm.fill(&mut key).unwrap();
    key
}

/// Serialize a P-256 public key to uncompressed point format.
///
/// Uncompressed format: 0x04 || x (32 bytes) || y (32 bytes) = 65 bytes
/// The leading 0x04 indicates uncompressed encoding.
///
/// Compressed format: 0x02/0x03 || x (32 bytes) = 33 bytes
/// (0x02 if y is even, 0x03 if y is odd)
pub fn serialize_public_key() -> Vec<u8> {
    let secret = EphemeralSecret::random(&mut OsRng);
    let public_key = secret.public_key();

    let encoded = public_key.to_encoded_point(false); // false = uncompressed
    encoded.as_bytes().to_vec()
}

/// Demonstrate that a corrupted public key is rejected.
///
/// P-256 public keys must satisfy the curve equation: y^2 = x^3 + ax + b (mod p)
/// Flipping any byte makes this equation false, so deserialization fails.
pub fn corrupted_key_rejected() -> bool {
    let secret = EphemeralSecret::random(&mut OsRng);
    let public_key = secret.public_key();
    let encoded = public_key.to_encoded_point(false);
    let mut bytes = encoded.as_bytes().to_vec();

    // Corrupt a byte
    bytes[10] ^= 0xFF;

    // Try to deserialize — should fail
    PublicKey::from_sec1_bytes(&bytes).is_err()
}

/// Check if bytes represent a valid P-256 point.
///
/// Valid P-256 points must:
/// 1. Have correct encoding (0x04 + 64 bytes for uncompressed)
/// 2. Satisfy the curve equation
/// 3. Not be the identity point
pub fn is_valid_point(bytes: &[u8]) -> bool {
    PublicKey::from_sec1_bytes(bytes).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecdh_produces_shared_secret() {
        let secret = ecdh_key_exchange();
        assert_eq!(secret.raw_secret_bytes().len(), 32);
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
        let secret = EphemeralSecret::random(&mut OsRng);
        let pub_key = secret.public_key();
        let encoded = pub_key.to_encoded_point(false);
        assert!(is_valid_point(encoded.as_bytes()), "Valid point should pass validation");
    }

    #[test]
    fn test_invalid_point_rejected() {
        let fake_point = vec![0x04u8; 65];
        let result = is_valid_point(&fake_point);
        assert!(!result, "Repeated bytes should not be a valid curve point");
    }

    #[test]
    fn test_two_independent_exchanges_differ() {
        let secret1 = ecdh_key_exchange();
        let secret2 = ecdh_key_exchange();
        assert_ne!(secret1.raw_secret_bytes(), secret2.raw_secret_bytes(),
            "Independent ECDH exchanges should produce different secrets");
    }
}
