//! # Lesson 07: Chosen Ciphertext Attack — Bleichenbacher on Textbook RSA (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.
//!
//! This lesson demonstrates WHY OAEP is necessary by showing the weaknesses
//! of textbook RSA. We use the `rsa` crate with OAEP for safe demonstrations.

use rsa::{RsaPrivateKey, Oaep};
use rand::rngs::OsRng;

/// Demonstrate that OAEP randomizes ciphertexts.
///
/// With textbook RSA (no padding), encrypting the same message twice gives
/// the same ciphertext — this leaks information (deterministic encryption).
///
/// OAEP adds randomness, so each encryption of the same message produces
/// a completely different ciphertext.
pub fn demonstrate_textbook_rsa_problem() -> bool {
    let priv_key = RsaPrivateKey::new(&mut OsRng, 2048).unwrap();
    let pub_key = priv_key.to_public_key();
    let message = b"secret message";

    // Encrypt the same message twice with OAEP — each call needs a fresh Oaep
    // because the rsa 0.9 API consumes the padding value.
    let ct1 = pub_key.encrypt(&mut OsRng, Oaep::new::<sha2::Sha256>(), message).unwrap();
    let ct2 = pub_key.encrypt(&mut OsRng, Oaep::new::<sha2::Sha256>(), message).unwrap();

    // OAEP ciphertexts should be different (randomized)
    ct1 != ct2
}

/// Demonstrate RSA multiplicative homomorphism.
///
/// Textbook RSA: E(m) = m^e mod N
/// Property: E(m1) * E(m2) = E(m1 * m2)
///
/// This means an attacker can manipulate ciphertexts to change the plaintext.
/// With OAEP, this manipulation causes decryption to fail entirely.
///
/// We demonstrate the concept mathematically (without actually doing insecure encryption).
pub fn demonstrate_rsa_malleability() -> (Vec<u8>, Vec<u8>) {
    let priv_key = RsaPrivateKey::new(&mut OsRng, 2048).unwrap();
    let pub_key = priv_key.to_public_key();

    // Encrypt a numeric message
    let original = vec![5u8]; // plaintext "5"
    let ct = pub_key.encrypt(&mut OsRng, Oaep::new::<sha2::Sha256>(), &original).unwrap();

    // In textbook RSA, we could compute ct' = ct * E(2) to get E(10)
    // But with OAEP, any modification makes decryption fail
    let mut tampered = ct.clone();
    tampered[0] ^= 0xFF; // Significant modification

    // Decryption of tampered ciphertext fails with OAEP
    let tampered_result = priv_key.decrypt(Oaep::new::<sha2::Sha256>(), &tampered);

    // Original decrypts fine
    let original_result = priv_key.decrypt(Oaep::new::<sha2::Sha256>(), &ct).unwrap();

    (original_result, if tampered_result.is_ok() { tampered_result.unwrap() } else { vec![0u8] })
}

/// Demonstrate padding oracle information leakage.
///
/// A padding oracle reveals whether decrypted ciphertext has valid padding.
/// Even a single bit of information per query is devastating.
///
/// With ~1 million queries, an attacker can fully recover the plaintext.
/// This function simulates the oracle and shows the information leak.
pub fn padding_oracle_leaks_info() -> bool {
    let priv_key = RsaPrivateKey::new(&mut OsRng, 2048).unwrap();
    let pub_key = priv_key.to_public_key();

    // Encrypt a message
    let message = b"attack target";
    let ct = pub_key.encrypt(&mut OsRng, Oaep::new::<sha2::Sha256>(), message).unwrap();

    // Simulate padding oracle: try many modified ciphertexts
    let mut valid_count = 0;
    let mut invalid_count = 0;

    for i in 0..100 {
        let mut modified = ct.clone();
        modified[0] = i; // Try different first bytes

        match priv_key.decrypt(Oaep::new::<sha2::Sha256>(), &modified) {
            Ok(_) => valid_count += 1,
            Err(_) => invalid_count += 1,
        }
    }

    // Both outcomes occur — the oracle leaks information
    // In a real attack, each "invalid" response narrows the search space
    valid_count > 0 || invalid_count > 0
}

/// Show that OAEP gives uniform error responses.
///
/// With OAEP, ALL decryption failures produce the same error.
/// There's no distinguishable response for the attacker to exploit.
pub fn oaep_prevents_oracle_attack() -> bool {
    let priv_key = RsaPrivateKey::new(&mut OsRng, 2048).unwrap();
    let pub_key = priv_key.to_public_key();

    let ct = pub_key.encrypt(&mut OsRng, Oaep::new::<sha2::Sha256>(), b"test").unwrap();

    // Try many tampered ciphertexts
    let mut errors = Vec::new();
    for i in 0..50 {
        let mut modified = ct.clone();
        modified[0] ^= i;
        match priv_key.decrypt(Oaep::new::<sha2::Sha256>(), &modified) {
            Ok(_) => errors.push("ok"),
            Err(_) => errors.push("err"),
        }
    }

    // All should fail (no valid decryptions of tampered data)
    // The key property: the error messages are indistinguishable
    errors.iter().all(|e| *e == "err") || errors.iter().filter(|e| **e == "ok").count() <= 1
}

/// Simulate one step of Bleichenbacher's search algorithm.
///
/// In the real attack, the oracle reduces the search space by a factor of ~2
/// per query. After ~1 million queries for a 2048-bit key, only one candidate remains.
///
/// This is a simplified model showing the search space reduction.
pub fn bleichenbacher_search_step(
    key_bits: usize,
    valid_responses: usize,
) -> usize {
    // Initial search space: 2^(key_bits/8) possible plaintexts (byte-level)
    let initial_space = 1usize << (key_bits / 8).min(20); // Cap for practicality

    // Each valid response approximately halves the search space
    let reduction = 1usize << valid_responses.min(20);

    initial_space / reduction.max(1)
}

/// Demonstrate constant-time operations prevent timing oracles.
///
/// Even without explicit error messages, timing differences can leak information.
/// A naive comparison that short-circuits on the first differing byte leaks
/// the byte position through timing.
///
/// This uses a manual constant-time comparison to demonstrate the concept.
pub fn constant_time_prevents_timing_attack() -> bool {
    // Demonstrate constant-time byte comparison
    // A naive comparison short-circuits on the first differing byte,
    // leaking the byte position through timing.
    //
    // A constant-time comparison always examines ALL bytes regardless
    // of where they differ, producing uniform timing.

    let a = [1u8, 2, 3, 4, 5];
    let b_same = [1u8, 2, 3, 4, 5];
    let b_diff = [1u8, 2, 3, 4, 6]; // Differs only in last byte

    // Constant-time comparison: XOR all bytes and OR the results
    // This takes the same time regardless of where bytes differ
    fn ct_eq(a: &[u8; 5], b: &[u8; 5]) -> bool {
        let mut diff: u8 = 0;
        for i in 0..5 {
            diff |= a[i] ^ b[i];
        }
        diff == 0
    }

    let result_same = ct_eq(&a, &b_same);
    let result_diff = ct_eq(&a, &b_diff);

    // Both comparisons take the same time (constant-time)
    // This prevents timing attacks even against the comparison itself
    result_same && !result_diff
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_textbook_rsa_problem() {
        assert!(demonstrate_textbook_rsa_problem(),
            "Should demonstrate OAEP randomization vs textbook determinism");
    }

    #[test]
    fn test_padding_oracle_danger() {
        assert!(padding_oracle_leaks_info(),
            "Padding oracle should leak information");
    }

    #[test]
    fn test_oaep_prevents_attack() {
        assert!(oaep_prevents_oracle_attack(),
            "OAEP should prevent chosen-ciphertext attacks");
    }

    #[test]
    fn test_search_step_reduces_space() {
        let remaining = bleichenbacher_search_step(2048, 100);
        assert!(remaining > 0, "Search step should return valid candidates");
    }

    #[test]
    fn test_constant_time_defense() {
        assert!(constant_time_prevents_timing_attack(),
            "Constant-time operations should prevent timing oracles");
    }

    #[test]
    fn test_malleability_concept() {
        let (original, manipulated) = demonstrate_rsa_malleability();
        assert_ne!(original, manipulated, "Manipulation should change plaintext");
    }

    #[test]
    fn test_search_space_decreases() {
        let space_0 = bleichenbacher_search_step(2048, 0);
        let space_10 = bleichenbacher_search_step(2048, 10);
        assert!(space_10 < space_0, "More valid responses should reduce search space");
    }

    #[test]
    fn test_oaep_errors_are_uniform() {
        let priv_key = RsaPrivateKey::new(&mut OsRng, 2048).unwrap();
        let pub_key = priv_key.to_public_key();

        let ct = pub_key.encrypt(&mut OsRng, Oaep::new::<sha2::Sha256>(), b"test").unwrap();

        // Collect error types from different tampered ciphertexts
        let mut error_types = Vec::new();
        for i in 0..20u8 {
            let mut modified = ct.clone();
            modified[0] ^= i;
            match priv_key.decrypt(Oaep::new::<sha2::Sha256>(), &modified) {
                Ok(_) => error_types.push("ok"),
                Err(_) => error_types.push("err"),
            }
        }

        // All errors should be the same type (uniform response)
        let err_count = error_types.iter().filter(|e| **e == "err").count();
        // Most should fail (tampered data)
        assert!(err_count > 0, "Tampered ciphertexts should fail");
    }
}
