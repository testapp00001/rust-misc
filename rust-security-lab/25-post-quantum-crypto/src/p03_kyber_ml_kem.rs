//! # Lesson 03: Kyber / ML-KEM — NIST-Selected Key Encapsulation Mechanism
//!
//! ## What is ML-KEM?
//!
//! ML-KEM (Module-Lattice Key Encapsulation Mechanism), formerly known as Kyber,
//! is the NIST-selected algorithm for post-quantum key exchange (FIPS 203, 2024).
//!
//! It replaces RSA key exchange and ECDH in protocols like TLS.
//!
//! ## How KEM Works
//!
//! ```
//! Alice (Encapsulate)                    Bob (Decapsulate)
//! ─────────────────────                  ───────────────────
//!                                          Generate (pk, sk)
//!    Receive pk                            Send pk →
//!    ←────────────────────────────────
//!    Generate random msg
//!    (ciphertext, shared_secret) = Enc(pk, msg)
//!    Send ciphertext ──────────────→
//!                                          shared_secret = Dec(sk, ciphertext)
//!    Both now have the same shared_secret!
//! ```
//!
//! ## Kyber vs RSA Key Exchange
//!
//! | Property | RSA-2048 | Kyber-768 |
//! |----------|----------|-----------|
//! | Key size | 256 bytes | 1184 bytes |
//! | Ciphertext | 256 bytes | 1088 bytes |
//! | Shared secret | 256 bytes | 32 bytes |
//! | Quantum safe | No | Yes |
//! | Speed | Slow | Fast |
//!
//! ## Security Levels
//!
//! | Kyber | NIST Level | Classical Equiv. | pk + ct size |
//! |-------|------------|------------------|--------------|
//! | 512 | 1 | AES-128 | 1568 bytes |
//! | 768 | 3 | AES-192 | 2272 bytes |
//! | 1024 | 5 | AES-256 | 3168 bytes |

use sha2::{Digest, Sha256};
use rand::Rng;

/// Simulated ML-KEM public key (contains matrix A and vector t = A*s + e).
#[derive(Debug, Clone)]
pub struct MlKemPublicKey {
    pub seed: Vec<u8>,
    pub t: Vec<i64>,
    pub n: usize,
    pub q: i64,
}

/// Simulated ML-KEM secret key (the secret vector s).
#[derive(Debug, Clone)]
pub struct MlKemSecretKey {
    pub s: Vec<i64>,
    pub pk: MlKemPublicKey,
}

/// Simulated ML-KEM ciphertext.
#[derive(Debug, Clone)]
pub struct MlKemCiphertext {
    pub u: Vec<i64>,
    pub v: i64,
}

/// Simulated shared secret (32 bytes, derived from hashing).
#[derive(Debug, Clone, PartialEq)]
pub struct SharedSecret {
    pub bytes: Vec<u8>,
}

/// Exercise 1: Generate an ML-KEM keypair (simplified simulation).
///
/// This is a SIMPLIFIED model, not real Kyber (which uses Module-LWE with
/// polynomial rings). We use basic LWE for educational purposes.
///
/// Steps:
/// 1. Generate random secret vector `s` with small coefficients (0 or 1)
/// 2. Generate random public matrix seed
/// 3. Compute t = A*s + e (using the seed to derive A deterministically)
/// 4. Return (public_key, secret_key)
///
/// Parameters: n = dimension, q = modulus
pub fn ml_kem_keygen(n: usize, q: i64) -> (MlKemPublicKey, MlKemSecretKey) {
    todo!("Generate ML-KEM keypair")
}

/// Exercise 2: Encapsulate — generate a shared secret and ciphertext.
///
/// Given a public key, the sender:
/// 1. Generates a random message m (32 bytes)
/// 2. Computes ciphertext using LWE encryption
/// 3. Derives shared_secret = Hash(m)
/// 4. Returns (ciphertext, shared_secret)
///
/// Hints:
/// - Generate random bytes for the message
/// - Use the public key's t vector and seed to compute LWE encryption
/// - Hash the message with SHA-256 for the shared secret
pub fn ml_kem_encapsulate(pk: &MlKemPublicKey) -> (MlKemCiphertext, SharedSecret) {
    todo!("Encapsulate: generate ciphertext and shared secret")
}

/// Exercise 3: Decapsulate — recover the shared secret from ciphertext.
///
/// Given a secret key and ciphertext:
/// 1. Compute m' = v - u . s (mod q) — extract the noisy message
/// 2. Round to recover the original message bits
/// 3. Derive shared_secret = Hash(m')
/// 4. Return shared_secret
///
/// Hints:
/// - This is LWE decryption
/// - The rounding must be consistent with encapsulation
pub fn ml_kem_decapsulate(sk: &MlKemSecretKey, ct: &MlKemCiphertext) -> SharedSecret {
    todo!("Decapsulate: recover shared secret from ciphertext")
}

/// Exercise 4: Verify that encapsulation and decapsulation produce the same secret.
///
/// This is a convenience function: encapsulate with the public key,
/// decapsulate with the secret key, and verify the shared secrets match.
pub fn ml_kem_verify_keypair(pk: &MlKemPublicKey, sk: &MlKemSecretKey) -> bool {
    todo!("Verify KEM correctness: encaps then decaps, check secrets match")
}

/// Exercise 5: Compute the ciphertext overhead compared to ECDH.
///
/// ECDH-P256 produces a 33-byte compressed point.
/// ML-KEM-768 produces a 1088-byte ciphertext.
///
/// Given the ML-KEM ciphertext size in bytes, return the overhead ratio
/// compared to ECDH (ciphertext_size / 33.0).
pub fn ciphertext_overhead_vs_ecdh(ml_kem_ct_size: usize) -> f64 {
    todo!("Compute ciphertext overhead ratio vs ECDH")
}

/// Exercise 6: Determine the Kyber security level for a given parameter set.
///
/// Map Kyber parameter names to NIST security levels:
/// - "Kyber512" / "ML-KEM-512" → 1
/// - "Kyber768" / "ML-KEM-768" → 3
/// - "Kyber1024" / "ML-KEM-1024" → 5
/// - Anything else → 0 (unknown)
pub fn kyber_security_level(params: &str) -> u32 {
    todo!("Map Kyber parameter set to NIST security level")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_kem_keygen_dimensions() {
        let (pk, sk) = ml_kem_keygen(4, 17);
        assert_eq!(pk.n, 4);
        assert_eq!(pk.q, 17);
        assert_eq!(sk.s.len(), 4);
        assert_eq!(pk.t.len(), 4);
    }

    #[test]
    fn test_ml_kem_encaps_produces_output() {
        let (pk, _sk) = ml_kem_keygen(4, 97);
        let (ct, ss) = ml_kem_encapsulate(&pk);
        assert_eq!(ct.u.len(), 4);
        assert_eq!(ss.bytes.len(), 32);
    }

    #[test]
    fn test_ml_kem_decaps_matches() {
        let (pk, sk) = ml_kem_keygen(4, 97);
        let (ct, ss_enc) = ml_kem_encapsulate(&pk);
        let ss_dec = ml_kem_decapsulate(&sk, &ct);
        assert_eq!(ss_enc, ss_dec);
    }

    #[test]
    fn test_ml_kem_verify_keypair() {
        let (pk, sk) = ml_kem_keygen(4, 97);
        assert!(ml_kem_verify_keypair(&pk, &sk));
    }

    #[test]
    fn test_ml_kem_different_messages() {
        let (pk, sk) = ml_kem_keygen(4, 97);
        let (ct1, ss1) = ml_kem_encapsulate(&pk);
        let (ct2, ss2) = ml_kem_encapsulate(&pk);
        // Different encapsulations should (almost certainly) produce different secrets
        // Note: there's a tiny probability of collision, but negligible
        assert_ne!(ct1.u, ct2.u);
        let dec1 = ml_kem_decapsulate(&sk, &ct1);
        let dec2 = ml_kem_decapsulate(&sk, &ct2);
        assert_eq!(ss1, dec1);
        assert_eq!(ss2, dec2);
    }

    #[test]
    fn test_ciphertext_overhead() {
        let overhead = ciphertext_overhead_vs_ecdh(1088);
        assert!((overhead - 1088.0 / 33.0).abs() < 0.01);
    }

    #[test]
    fn test_kyber_security_levels() {
        assert_eq!(kyber_security_level("Kyber512"), 1);
        assert_eq!(kyber_security_level("ML-KEM-768"), 3);
        assert_eq!(kyber_security_level("Kyber1024"), 5);
        assert_eq!(kyber_security_level("ML-KEM-512"), 1);
        assert_eq!(kyber_security_level("unknown"), 0);
    }
}
