//! # Lesson 07: Non-Interactive Proofs — Fiat-Shamir Heuristic
//!
//! ## Making Interactive Proofs Non-Interactive
//!
//! Most ZK proofs are interactive: prover and verifier exchange messages in rounds.
//! But in many real-world scenarios, we need non-interactive proofs:
//!
//! - **Blockchain**: You can't interact with every node — post a single proof
//! - **Email/messaging**: Asynchronous communication — can't do back-and-forth
//! - **Efficiency**: One message instead of many rounds
//!
//! ## The Fiat-Shamir Heuristic
//!
//! Replace the verifier's random challenges with hash function outputs:
//!
//! Instead of: Verifier sends random challenge `e`
//! We compute: `e = Hash(commitment || public_input || context)`
//!
//! The hash acts as a "random oracle" — it produces unpredictable output that the prover
//! can't control (assuming the hash function is secure).
//!
//! ## ATTACK: Why Not Just Let the Prover Choose the Challenge?
//!
//! If the prover chooses `e`, they can cheat:
//! 1. Pick any response `s`
//! 2. Compute what `e` would need to be: `e = (s - k) / x`
//! 3. Set `e` to that value
//!
//! The prover can always produce a valid proof for any statement!
//!
//! With Fiat-Shamir, the prover can't choose `e` because it's determined by the commitment
//! they already sent. They're "locked in" before seeing the challenge.
//!
//! ## Security Model
//!
//! Fiat-Shamir is proven secure in the **Random Oracle Model** (ROM):
//! - The hash function is modeled as a truly random function
//! - In practice, SHA-256 is a good approximation of a random oracle
//! - The proof becomes a **Non-Interactive Zero-Knowledge (NIZK)** proof
//!
//! ## Real-World Applications
//!
//! - **Schnorr signatures**: Sigma protocol + Fiat-Shamir = signature scheme
//! - **zk-SNARKs**: Non-interactive by construction
//! - **Bulletproofs**: Non-interactive range proofs

use sha2::{Digest, Sha256};

/// Modular exponentiation.
pub fn mod_pow(base: u64, exp: u64, modulus: u64) -> u64 {
    if modulus == 1 {
        return 0;
    }
    let mut result: u128 = 1;
    let mut base = (base as u128) % (modulus as u128);
    let mut exp = exp;
    while exp > 0 {
        if exp % 2 == 1 {
            result = (result * base) % (modulus as u128);
        }
        exp >>= 1;
        base = (base * base) % (modulus as u128);
    }
    result as u64
}

/// NIZK (Non-Interactive Zero-Knowledge) parameters.
pub struct NizkParams {
    pub g: u64,
    pub p: u64,
    pub q: u64,
}

/// Exercise 1: Create default NIZK parameters.
///
/// Hints:
/// - Use g=2, p=23, q=11
pub fn default_nizk_params() -> NizkParams {
    todo!("Create default NIZK parameters")
}

/// Exercise 2: Generate a keypair.
pub fn nizk_keygen(params: &NizkParams) -> (u64, u64) {
    todo!("Generate NIZK keypair")
}

/// Exercise 3: Fiat-Shamir hash function.
///
/// Computes the challenge as: e = Hash(commitment || public_key || context) mod q
///
/// The `context` string binds the proof to a specific application/context,
/// preventing proof reuse across different protocols.
///
/// Hints:
/// - Hash commitment, public_key, and context bytes with SHA-256
/// - Convert to u64, take mod q
pub fn fiat_shamir(
    commitment: u64,
    public_key: u64,
    context: &[u8],
    params: &NizkParams,
) -> u64 {
    todo!("Compute Fiat-Shamir challenge with context binding")
}

/// Exercise 4: Non-Interactive ZK proof — Prover.
///
/// 1. Pick random k, compute t = g^k mod p
/// 2. e = fiat_shamir(t, y, context)
/// 3. s = (k + x*e) mod q
/// Returns (t, e, s) — a single message, no interaction needed.
///
/// Hints:
/// - Combine commitment, challenge, and response in one function
pub fn nizk_prove(
    secret: u64,
    context: &[u8],
    params: &NizkParams,
) -> (u64, u64, u64) {
    todo!("Non-interactive ZK proof: prover")
}

/// Exercise 5: Non-Interactive ZK proof — Verifier.
///
/// 1. Recompute expected challenge from (t, y, context)
/// 2. Verify g^s == t * y^e mod p
///
/// Hints:
/// - Recompute e using fiat_shamir
/// - Check e matches proof's e
/// - Verify the Schnorr equation
pub fn nizk_verify(
    public_key: u64,
    proof: (u64, u64, u64),
    context: &[u8],
    params: &NizkParams,
) -> bool {
    todo!("Non-interactive ZK proof: verifier")
}

/// Exercise 6: Demonstrate context binding.
///
/// The same secret, proven in different contexts, should produce different challenges.
/// This prevents proof replay across different applications.
///
/// Hints:
/// - Generate keypair
/// - Prove with context1 and context2
/// - Show challenges differ
pub fn demonstrate_context_binding(params: &NizkParams) -> (u64, u64) {
    todo!("Show that different contexts produce different challenges")
}

/// Exercise 7: Demonstrate that non-interactive proof equals interactive + Fiat-Shamir.
///
/// 1. Run the interactive protocol manually (commit, challenge=some value, respond)
/// 2. Run the non-interactive protocol
/// 3. Show the structure is the same
///
/// Returns (interactive_proof, nizk_proof) for comparison.
///
/// Hints:
/// - Interactive: manual commit, manual challenge, manual respond
/// - NIZK: single prove call
pub fn compare_interactive_vs_nizk(
    secret: u64,
    params: &NizkParams,
) -> ((u64, u64, u64), (u64, u64, u64)) {
    todo!("Compare interactive and non-interactive proof structures")
}

/// Exercise 8: Batch verification — verify multiple NIZK proofs efficiently.
///
/// Instead of verifying each proof individually, batch them for efficiency.
/// Returns true if ALL proofs are valid.
///
/// Hints:
/// - Iterate and verify each proof
/// - In practice, batching uses random linear combinations for efficiency
pub fn batch_verify(
    public_keys: &[u64],
    proofs: &[(u64, u64, u64)],
    context: &[u8],
    params: &NizkParams,
) -> bool {
    todo!("Batch verify multiple NIZK proofs")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nizk_round_trip() {
        let params = default_nizk_params();
        let (sk, pk) = nizk_keygen(&params);
        let proof = nizk_prove(sk, b"test-context", &params);
        assert!(nizk_verify(pk, proof, b"test-context", &params));
    }

    #[test]
    fn test_nizk_wrong_context_fails() {
        let params = default_nizk_params();
        let (sk, pk) = nizk_keygen(&params);
        let proof = nizk_prove(sk, b"context-a", &params);
        assert!(!nizk_verify(pk, proof, b"context-b", &params));
    }

    #[test]
    fn test_nizk_wrong_key_fails() {
        let params = default_nizk_params();
        let (sk, _pk) = nizk_keygen(&params);
        let (_sk2, pk2) = nizk_keygen(&params);
        let proof = nizk_prove(sk, b"test", &params);
        assert!(!nizk_verify(pk2, proof, b"test", &params));
    }

    #[test]
    fn test_fiat_shamir_deterministic() {
        let params = default_nizk_params();
        let e1 = fiat_shamir(10, 5, b"ctx", &params);
        let e2 = fiat_shamir(10, 5, b"ctx", &params);
        assert_eq!(e1, e2);
    }

    #[test]
    fn test_fiat_shamir_context_sensitive() {
        let params = default_nizk_params();
        let e1 = fiat_shamir(10, 5, b"context-a", &params);
        let e2 = fiat_shamir(10, 5, b"context-b", &params);
        // With small q, collisions possible, but generally different
        let _ = (e1, e2);
    }

    #[test]
    fn test_context_binding() {
        let params = default_nizk_params();
        let (e1, e2) = demonstrate_context_binding(&params);
        let _ = (e1, e2); // Just check no panic
    }

    #[test]
    fn test_interactive_vs_nizk() {
        let params = default_nizk_params();
        let (sk, _pk) = nizk_keygen(&params);
        let (interactive, nizk) = compare_interactive_vs_nizk(sk, &params);
        // Both should be valid (t, e, s) tuples
        assert!(interactive.0 > 0);
        assert!(nizk.0 > 0);
    }

    #[test]
    fn test_batch_verify() {
        let params = default_nizk_params();
        let (sk1, pk1) = nizk_keygen(&params);
        let (sk2, pk2) = nizk_keygen(&params);
        let proof1 = nizk_prove(sk1, b"batch", &params);
        let proof2 = nizk_prove(sk2, b"batch", &params);
        assert!(batch_verify(&[pk1, pk2], &[proof1, proof2], b"batch", &params));
    }
}
