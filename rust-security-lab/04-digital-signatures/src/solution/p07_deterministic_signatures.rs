//! # Lesson 07: Deterministic Signatures / RFC 6979 (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use sha2::{Sha256, Digest};
use hmac::Hmac;
use sha2::digest::Mac;
use rand::Rng;

type HmacSha256 = Hmac<Sha256>;

/// Compute a deterministic nonce using HMAC-SHA256.
///
/// This is the fundamental operation of RFC 6979. The nonce is derived
/// from the private key and message, making it:
/// - Deterministic (same inputs -> same output)
/// - Unpredictable (without the key)
/// - Unique per message
pub fn compute_deterministic_nonce(key: &[u8], message: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(message);
    mac.finalize().into_bytes().to_vec()
}

/// Generate an RFC 6979-style deterministic nonce with counter.
///
/// If the initial nonce is zero (invalid), increment a counter and retry.
/// In practice, the chance of getting zero is negligible (1/2^256).
pub fn generate_rfc6979_nonce(
    key: &[u8],
    message: &[u8],
    curve_order: &[u8],
) -> Vec<u8> {
    for counter in 0u8..=255 {
        let mut input = Vec::with_capacity(message.len() + 1);
        input.extend_from_slice(message);
        input.push(counter);

        let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
        mac.update(&input);
        let nonce = mac.finalize().into_bytes().to_vec();

        // Check: nonce must be non-zero and less than curve order
        if nonce.iter().all(|&b| b == 0) {
            continue;
        }

        // Simple range check (not strictly RFC 6979 compliant but demonstrates the concept)
        if nonce.len() <= curve_order.len() {
            return nonce;
        }
    }
    panic!("Could not generate valid nonce after 256 attempts (astronomically unlikely)")
}

/// Demonstrate that deterministic nonces are repeatable.
pub fn demonstrate_determinism(key: &[u8], message: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let nonce1 = compute_deterministic_nonce(key, message);
    let nonce2 = compute_deterministic_nonce(key, message);
    (nonce1, nonce2)
}

/// Demonstrate that different messages produce different nonces.
pub fn demonstrate_uniqueness(key: &[u8], msg1: &[u8], msg2: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let nonce1 = compute_deterministic_nonce(key, msg1);
    let nonce2 = compute_deterministic_nonce(key, msg2);
    (nonce1, nonce2)
}

/// Simplified deterministic signing.
///
/// This demonstrates the concept: derive k deterministically, then use it.
/// Real ECDSA would compute R = k*G and s = k^(-1)(z + r*privkey).
pub fn deterministic_sign(private_key: &[u8], message: &[u8]) -> (Vec<u8>, Vec<u8>) {
    // Derive deterministic nonce
    let k = compute_deterministic_nonce(private_key, message);

    // Simplified "signature": HMAC(k, message)
    // Real ECDSA would do elliptic curve math here
    let mut mac = HmacSha256::new_from_slice(&k).expect("HMAC accepts any key length");
    mac.update(message);
    let sig = mac.finalize().into_bytes().to_vec();

    (k, sig)
}

/// Simulate the nonce-reuse attack.
///
/// ATTACK EXPLANATION:
/// In real ECDSA with reused k:
///   s1 = k^-1 * (z1 + r*privkey) mod n
///   s2 = k^-1 * (z2 + r*privkey) mod n
///
/// Subtracting: s1 - s2 = k^-1 * (z1 - z2) mod n
/// Therefore:   k = (z1 - z2) / (s1 - s2) mod n
/// Then:        privkey = (s1*k - z1) / r mod n
///
/// This function demonstrates a simplified version using XOR as a
/// stand-in for modular arithmetic.
pub fn simulate_nonce_reuse_attack(
    k: &[u8],
    z1: &[u8],
    s1: &[u8],
    z2: &[u8],
    s2: &[u8],
) -> Vec<u8> {
    // Simplified: "recover" key material by combining signature components
    // In real ECDSA this would be modular arithmetic over the curve order
    let mut recovered = Vec::with_capacity(32);

    // Simulate: privkey = (s1 * k - z1) * r_inv
    // Using XOR as a simplified stand-in
    for i in 0..32.min(s1.len()).min(k.len()).min(z1.len()) {
        let byte = s1[i].wrapping_mul(k[i]).wrapping_sub(z1[i]);
        recovered.push(byte);
    }

    // Include information from second signature to show it helps
    for i in 0..32.min(s2.len()).min(z2.len()) {
        if i < recovered.len() {
            recovered[i] ^= s2[i].wrapping_sub(z2[i]);
        }
    }

    recovered
}

/// Compare deterministic vs random nonce generation.
pub fn compare_nonce_methods(key: &[u8], message: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let deterministic = compute_deterministic_nonce(key, message);

    let mut rng = rand::thread_rng();
    let random_nonce: Vec<u8> = (0..32).map(|_| rng.gen()).collect();

    (deterministic, random_nonce)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_nonce_not_empty() {
        let nonce = compute_deterministic_nonce(b"secret_key", b"message");
        assert!(!nonce.is_empty());
        assert_eq!(nonce.len(), 32);
    }

    #[test]
    fn test_deterministic_nonce_repeatable() {
        let (n1, n2) = demonstrate_determinism(b"key", b"msg");
        assert_eq!(n1, n2);
    }

    #[test]
    fn test_deterministic_nonce_unique_per_message() {
        let (n1, n2) = demonstrate_uniqueness(b"key", b"msg1", b"msg2");
        assert_ne!(n1, n2);
    }

    #[test]
    fn test_rfc6979_nonce_valid() {
        let order = vec![0xFFu8; 32];
        let nonce = generate_rfc6979_nonce(b"key", b"msg", &order);
        assert!(!nonce.is_empty());
        assert!(nonce.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_deterministic_sign_repeatable() {
        let key = b"private_key_material_here_32byte";
        let (k1, sig1) = deterministic_sign(key, b"message");
        let (k2, sig2) = deterministic_sign(key, b"message");
        assert_eq!(k1, k2);
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_deterministic_sign_varies_by_message() {
        let key = b"private_key_material_here_32byte";
        let (_, sig1) = deterministic_sign(key, b"message1");
        let (_, sig2) = deterministic_sign(key, b"message2");
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_nonce_reuse_attack() {
        let k = [1u8; 32];
        let z1 = [2u8; 32];
        let s1 = [3u8; 32];
        let z2 = [4u8; 32];
        let s2 = [5u8; 32];
        let recovered = simulate_nonce_reuse_attack(&k, &z1, &s1, &z2, &s2);
        assert!(!recovered.is_empty());
    }

    #[test]
    fn test_compare_nonce_methods() {
        let key = b"test_key_32_bytes_padding_here!!";
        let message = b"test message";
        let (det, rand_nonce) = compare_nonce_methods(key, message);
        assert_eq!(det.len(), 32);
        assert_eq!(rand_nonce.len(), 32);
        let (det2, _) = compare_nonce_methods(key, message);
        assert_eq!(det, det2);
    }
}
