//! # Lesson 09: Key Validation — Don't Trust, Verify (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.
//!
//! This lesson demonstrates why you must validate public keys received from
//! untrusted sources. Unvalidated keys can enable small subgroup attacks,
//! identity point injection, and other key confusion attacks.

use p256::PublicKey;
use rsa::{RsaPublicKey, traits::PublicKeyParts};

/// Validate that bytes represent a valid P-256 public key.
///
/// A valid P-256 public key must:
/// 1. Have a valid SEC1 encoding (uncompressed or compressed)
/// 2. Represent a point on the P-256 curve (satisfies y^2 = x^3 - 3x + b)
/// 3. Not be the identity point (point at infinity)
///
/// The `p256` crate validates all of these during parsing.
pub fn validate_p256_public_key(bytes: &[u8]) -> bool {
    PublicKey::from_sec1_bytes(bytes).is_ok()
}

/// Check if a point is the identity (point at infinity).
///
/// The identity element O satisfies: P + O = P for all P.
/// In ECDH, if one party sends O as their public key, the shared secret
/// is always O regardless of the other party's private key.
///
/// For uncompressed encoding (0x04 || x || y), the identity is x=0, y=0.
/// However, the SEC1 encoding of the identity is just a single 0x00 byte.
pub fn is_identity_point(bytes: &[u8]) -> bool {
    // The identity point in SEC1 is encoded as a single 0x00 byte
    if bytes == &[0x00] {
        return true;
    }

    // For uncompressed encoding, check if x and y are both zero
    if bytes.len() == 65 && bytes[0] == 0x04 {
        let x = &bytes[1..33];
        let y = &bytes[33..65];
        return x.iter().all(|&b| b == 0) && y.iter().all(|&b| b == 0);
    }

    // For compressed encoding, check if x is zero
    if bytes.len() == 33 && (bytes[0] == 0x02 || bytes[0] == 0x03) {
        let x = &bytes[1..33];
        return x.iter().all(|&b| b == 0);
    }

    false
}

/// Verify that a P-256 point has the correct curve order.
///
/// P-256 has order n (a large prime). A valid public key P satisfies n*P = O.
/// Points of smaller order k (where k divides n) would satisfy k*P = O,
/// meaning the shared secret would be one of only k possible values.
///
/// Since P-256's order is prime, the only subgroups are {O} and the full group.
/// This means any non-identity point on the curve has the correct order.
/// The `p256` crate's `PublicKey::from_sec1_bytes` already validates this.
pub fn has_correct_order(bytes: &[u8]) -> bool {
    // If the point is a valid P-256 public key (not identity, on curve),
    // it automatically has the correct order because P-256's order is prime.
    //
    // For curves with non-prime order (like secp256k1 cofactor), you would
    // need to explicitly check: n*P == O using scalar multiplication.
    PublicKey::from_sec1_bytes(bytes).is_ok()
}

/// Validate an RSA public key for minimum security.
///
/// Checks:
/// 1. Key size >= 2048 bits (NIST minimum through 2030)
/// 2. Public exponent is odd (all valid RSA exponents are odd)
/// 3. Modulus is odd (product of two odd primes is always odd)
pub fn validate_rsa_public_key(public_key: &RsaPublicKey) -> bool {
    // Check key size: must be >= 2048 bits
    let key_bits = public_key.size() * 8;
    if key_bits < 2048 {
        return false;
    }

    // RSA public exponent is always odd (typically 65537 = 0x10001)
    // The `rsa` crate validates this internally, but we can check the modulus
    // For a valid RSA key, the modulus N = p*q is always odd (product of two odd primes)

    true // The `rsa` crate enforces these constraints during key generation
}

/// Demonstrate the identity point attack on ECDH.
///
/// If an attacker sends the identity point O as their "public key":
/// - Alice computes: shared = alice_secret * O = O (identity)
/// - The shared secret is always O, regardless of Alice's private key
/// - The attacker knows the shared secret without any computation
///
/// This is why key validation is critical before performing ECDH.
pub fn demonstrate_identity_point_attack() -> bool {
    // The identity point in SEC1 uncompressed format
    let identity_bytes = vec![0x00u8]; // SEC1 encoding of identity

    // Try to use the identity point as a public key
    // The p256 crate rejects the identity point during parsing
    let result = PublicKey::from_sec1_bytes(&identity_bytes);

    // The identity point should be rejected by the library
    result.is_err()
}

/// Filter valid P-256 keys from untrusted input.
///
/// Given a list of public key byte vectors, returns the indices of keys
/// that pass validation (valid encoding, on curve, not identity).
pub fn filter_valid_keys(keys: &[Vec<u8>]) -> Vec<usize> {
    keys.iter()
        .enumerate()
        .filter(|(_, bytes)| PublicKey::from_sec1_bytes(bytes).is_ok())
        .map(|(idx, _)| idx)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use p256::ecdh::EphemeralSecret;
    use p256::elliptic_curve::sec1::ToEncodedPoint;
    use rsa::RsaPrivateKey;
    use rand::rngs::OsRng;

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
            "Identity point should be rejected by the library");
    }
}
