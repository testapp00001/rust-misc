//! # Lesson 09: Key Validation — Don't Trust, Verify
//!
//! ## Why Validate Public Keys?
//!
//! When you receive a public key from an untrusted source, you must verify:
//!
//! 1. **Point on curve**: Is this actually a valid point on the elliptic curve?
//! 2. **Not identity**: Is it the point at infinity (identity element)?
//! 3. **Not small subgroup**: Does it have the correct large order?
//! 4. **Not weak key**: For RSA, is the key large enough and properly generated?
//!
//! ## Small Subgroup Attack
//!
//! On an elliptic curve, a point P of small order k satisfies k*P = O (identity).
//! If Alice uses such a point in ECDH, the shared secret will be one of only
//! k possible values — trivially brute-forceable.
//!
//! Attack:
//! 1. Mallory sends a point of order 2 (instead of a random point)
//! 2. Alice computes shared_secret = alice_secret * P_small
//! 3. The result is one of 2 possible values
//! 4. Mallory tries both and recovers the message
//!
//! Defense: Always verify that received points have the correct order.
//!
//! ## Weak RSA Keys
//!
//! - Keys < 2048 bits are factorizable
//! - Keys with shared prime factors can be recovered via GCD
//! - Keys generated with poor randomness are predictable
//!
//! ## Attack Demo: Low-Order Point Injection
//!
//! An attacker sends the identity point (point at infinity) as their public key.
//! The shared secret becomes zero regardless of the private key.

use p256::{PublicKey, EncodedPoint, NistP256};
use p256::elliptic_curve::sec1::ToEncodedPoint;
use rsa::{RsaPublicKey, RsaPrivateKey};
use rand::rngs::OsRng;

/// Exercise 1: Validate that bytes represent a valid P-256 public key.
///
/// A valid P-256 public key must:
/// 1. Be a valid encoding (uncompressed: 0x04 || x || y, or compressed: 0x02/0x03 || x)
/// 2. Represent a point on the P-256 curve
/// 3. Not be the identity point
///
/// Returns `true` if the key is valid.
///
/// Hints:
/// - Try `PublicKey::from_sec1_bytes(bytes)` — it validates the point is on the curve
/// - Check that the point is not the identity (all zeros)
pub fn validate_p256_public_key(bytes: &[u8]) -> bool {
    todo!("Validate a P-256 public key")
}

/// Exercise 2: Check if a point is the identity (point at infinity).
///
/// The identity element acts as zero in elliptic curve arithmetic.
/// Using it as a public key makes the shared secret zero.
///
/// Hints:
/// - Decode the point
/// - Check if it's the identity element
/// - For uncompressed encoding: x=0, y=0 (after the 0x04 tag)
pub fn is_identity_point(bytes: &[u8]) -> bool {
    todo!("Check if a point is the identity element")
}

/// Exercise 3: Validate that a P-256 point has the correct order.
///
/// P-256 has order n. A valid public key P should satisfy n*P = O (identity).
/// Points of smaller order are dangerous.
///
/// Hints:
/// - Use `p256::ProjectivePoint` for arithmetic
/// - Multiply by the curve order and check if result is identity
/// - Use `p256::NistP256::ORDER` or `p256::ORDER`
pub fn has_correct_order(bytes: &[u8]) -> bool {
    todo!("Verify point has correct curve order")
}

/// Exercise 4: Validate an RSA public key for minimum security.
///
/// Checks:
/// 1. Key size >= 2048 bits
/// 2. Public exponent is odd (all valid RSA exponents are odd)
/// 3. Modulus is odd (product of two odd primes is always odd)
pub fn validate_rsa_public_key(public_key: &RsaPublicKey) -> bool {
    todo!("Validate RSA public key security properties")
}

/// Exercise 5: Demonstrate the identity point attack.
///
/// If an attacker sends the identity point as their "public key", the ECDH
/// shared secret will always be the identity — regardless of the other party's
/// private key. This means the attacker knows the shared secret.
///
/// Returns true if the attack is demonstrated (identity point produces zero secret).
pub fn demonstrate_identity_point_attack() -> bool {
    todo!("Show that identity point trivially breaks ECDH")
}

/// Exercise 6: Validate multiple keys from an untrusted source.
///
/// Given a list of public key byte vectors, return the indices of valid ones.
///
/// Hints:
/// - Try to parse each as a P-256 public key
/// - Return the indices (0-based) of keys that pass validation
pub fn filter_valid_keys(keys: &[Vec<u8>]) -> Vec<usize> {
    todo!("Filter valid P-256 keys from untrusted input")
}

#[cfg(test)]
mod tests {
    use super::*;
    use p256::ecdh::EphemeralSecret;

    #[test]
    fn test_valid_key_passes() {
        let secret = EphemeralSecret::random(&mut OsRng);
        let pub_key = secret.public_key();
        let encoded = pub_key.to_encoded_point(false);
        assert!(validate_p256_public_key(encoded.as_bytes()),
            "Valid key should pass validation");
    }

    #[test]
    fn test_random_bytes_fail() {
        let garbage = vec![0x04u8; 65]; // Not a valid point
        assert!(!validate_p256_public_key(&garbage),
            "Random bytes should fail validation");
    }

    #[test]
    fn test_identity_point_detected() {
        // Identity point: 0x04 || 0x00...00 || 0x00...00
        let mut identity = vec![0x04u8];
        identity.extend(vec![0u8; 64]);
        assert!(is_identity_point(&identity), "Should detect identity point");
    }

    #[test]
    fn test_valid_point_not_identity() {
        let secret = EphemeralSecret::random(&mut OsRng);
        let pub_key = secret.public_key();
        let encoded = pub_key.to_encoded_point(false);
        assert!(!is_identity_point(encoded.as_bytes()),
            "Random public key should not be identity");
    }

    #[test]
    fn test_correct_order_check() {
        let secret = EphemeralSecret::random(&mut OsRng);
        let pub_key = secret.public_key();
        let encoded = pub_key.to_encoded_point(false);
        assert!(has_correct_order(encoded.as_bytes()),
            "Random P-256 point should have correct order");
    }

    #[test]
    fn test_rsa_key_validation() {
        let priv_key = RsaPrivateKey::new(&mut OsRng, 2048).unwrap();
        let pub_key = priv_key.to_public_key();
        assert!(validate_rsa_public_key(&pub_key), "2048-bit RSA key should be valid");
    }

    #[test]
    fn test_filter_valid_keys() {
        let secret = EphemeralSecret::random(&mut OsRng);
        let pub_key = secret.public_key();
        let encoded = pub_key.to_encoded_point(false);

        let keys = vec![
            encoded.as_bytes().to_vec(),  // Valid
            vec![0x04u8; 65],             // Invalid (not on curve)
            encoded.as_bytes().to_vec(),  // Valid
        ];

        let valid_indices = filter_valid_keys(&keys);
        assert_eq!(valid_indices, vec![0, 2], "Should identify valid keys");
    }

    #[test]
    fn test_identity_point_attack() {
        assert!(demonstrate_identity_point_attack(),
            "Identity point should produce predictable shared secret");
    }
}
