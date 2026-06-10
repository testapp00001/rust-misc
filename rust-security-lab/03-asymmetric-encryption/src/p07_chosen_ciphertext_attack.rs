//! # Lesson 07: Chosen Ciphertext Attack — Bleichenbacher on Textbook RSA
//!
//! ## The Bleichenbacher Attack (1998)
//!
//! Daniel Bleichenbacher demonstrated a devastating attack against RSA with
//! PKCS#1 v1.5 padding. The attack works as follows:
//!
//! 1. The attacker has a target ciphertext c = m^e mod N
//! 2. The attacker sends modified ciphertexts c' to the server
//! 3. The server decrypts and responds with "valid padding" or "invalid padding"
//! 4. Each response leaks 1 bit of information about the plaintext
//! 5. After ~1 million queries, the attacker recovers the full plaintext
//!
//! This attack broke SSL 3.0 and forced the adoption of OAEP padding.
//!
//! ## Why OAEP Defeats Bleichenbacher
//!
//! OAEP uses a Feistel network with a random oracle (hash function).
//! The decryption process:
//! 1. Checks the OAEP structure (hash verification)
//! 2. If ANY check fails, returns a single generic error
//! 3. No distinguishable error messages for the attacker to exploit
//!
//! This makes the attack impossible — the attacker gets no useful feedback.
//!
//! ## Modern Impact
//!
//! - TLS 1.3 removed RSA key exchange entirely (uses ECDH + KEM)
//! - PKCS#1 v1.5 is still used in some legacy systems — and still vulnerable
//! - ROBOT attack (2017) showed Bleichenbacher variants still work against TLS
//!
//! ## This Lesson
//!
//! We demonstrate the concept of the attack using textbook RSA (no padding).
//! This is purely educational — real systems must use OAEP.

use rsa::{RsaPrivateKey, RsaPublicKey, Oaep, BigUint};
use rand::rngs::OsRng;

/// Exercise 1: Demonstrate that textbook RSA is deterministic (same plaintext → same ciphertext).
///
/// Textbook RSA: c = m^e mod N
/// This means encrypting the same message twice gives the same ciphertext.
/// OAEP adds randomness so this doesn't happen.
///
/// Hints:
/// - Encrypt the same message twice with OAEP
/// - The ciphertexts should be DIFFERENT (OAEP is randomized)
/// - If we used textbook RSA, they'd be the same
pub fn demonstrate_textbook_rsa_problem() -> bool {
    todo!("Show that OAEP randomizes ciphertexts (textbook RSA doesn't)")
}

/// Exercise 2: Demonstrate RSA malleability.
///
/// RSA is multiplicatively homomorphic: E(m1) * E(m2) = E(m1 * m2)
/// An attacker can manipulate ciphertexts to change the plaintext.
///
/// Given c = m^e mod N, compute c' = c * 2^e mod N.
/// Decrypting c' gives 2m — the attacker doubled the plaintext!
///
/// Returns (original_plaintext, manipulated_plaintext).
///
/// Hints:
/// - Encrypt a message
/// - Compute the encryption of 2: e2 = 2^e mod N (using the public key)
/// - Multiply: c' = c * e2 mod N
/// - Decrypt c' — should get 2 * original_plaintext
/// - NOTE: This won't work with OAEP (it would fail decryption), so we
///   demonstrate the concept by computing the mathematical relationship
pub fn demonstrate_rsa_malleability() -> (Vec<u8>, Vec<u8>) {
    todo!("Demonstrate RSA multiplicative homomorphism")
}

/// Exercise 3: Show why padding-oracle-style information leaks are dangerous.
///
/// Simulate a server that reveals whether padding is valid.
/// Even a single bit of information per query is enough to mount the attack.
///
/// Returns true if the simulated oracle leaks information.
pub fn padding_oracle_leaks_info() -> bool {
    todo!("Demonstrate information leakage from padding oracle")
}

/// Exercise 4: Demonstrate that OAEP prevents the attack.
///
/// With OAEP, any modification to the ciphertext causes decryption to fail
/// with the SAME error — no distinguishable responses for the attacker.
pub fn oaep_prevents_oracle_attack() -> bool {
    todo!("Show that OAEP gives uniform error responses")
}

/// Exercise 5: Simulate the Bleichenbacher search step.
///
/// In the real attack, the attacker uses the oracle to narrow down the
/// possible plaintext range. This function demonstrates one step:
/// given a "valid" response, compute the reduced search space.
///
/// Returns the number of possible plaintext candidates remaining.
pub fn bleichenbacher_search_step(
    key_bits: usize,
    valid_responses: usize,
) -> usize {
    todo!("Simulate one step of Bleichenbacher's search")
}

/// Exercise 6: Show that constant-time comparison prevents timing oracles.
///
/// Even if the server doesn't reveal padding validity in its response,
/// timing differences can leak information.
///
/// Returns true if constant-time comparison is used.
pub fn constant_time_prevents_timing_attack() -> bool {
    todo!("Demonstrate timing oracle prevention with constant-time operations")
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
        // Each valid response should reduce the search space
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
        // The manipulated plaintext should be different from the original
        // (specifically, it should be 2x the original in the textbook RSA demo)
        assert_ne!(original, manipulated, "Manipulation should change plaintext");
    }
}
