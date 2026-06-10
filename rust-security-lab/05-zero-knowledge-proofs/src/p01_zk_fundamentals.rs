//! # Lesson 01: Zero-Knowledge Fundamentals
//!
//! ## What is Zero-Knowledge?
//!
//! A zero-knowledge proof lets you prove you know a secret without revealing the secret itself.
//! This sounds impossible, but it's a well-studied area of cryptography with real-world deployments.
//!
//! ## The Three Properties
//!
//! 1. **Completeness**: If the statement is true and the prover is honest, the verifier will be convinced.
//! 2. **Soundness**: If the statement is false, no cheating prover can convince the verifier
//!    (except with negligible probability).
//! 3. **Zero-Knowledge**: The verifier learns nothing beyond the fact that the statement is true.
//!
//! ## Example: Hash Preimage Proof
//!
//! Suppose Alice knows a secret `s` and has published `h = SHA256(s)`.
//! She wants to prove she knows `s` without revealing it.
//!
//! ### Without ZK (Naive Approach) — ATTACK:
//! Alice sends `s` to Bob. Bob computes `SHA256(s)` and checks it matches `h`.
//! Problem: Bob now knows `s`! He can impersonate Alice, sell the secret, etc.
//!
//! ### With ZK (Commitment-Challenge Protocol):
//! 1. Alice picks random `r`, computes commitment `c = SHA256(r)`, sends `c` to Bob
//! 2. Bob sends random challenge bit `b` (0 or 1)
//! 3. If `b == 0`: Alice sends `r` (proves she knew the commitment)
//!    If `b == 1`: Alice sends `r XOR s` (proves she knows s without revealing it directly)
//! 4. Bob verifies
//!
//! After multiple rounds, Alice's probability of cheating becomes negligible.
//!
//! ## Real-World Applications
//!
//! - **Password authentication**: Prove you know the password without sending it
//! - **Blockchain**: Prove a transaction is valid without revealing amounts
//! - **Identity**: Prove you're over 18 without revealing your birthdate

use sha2::{Digest, Sha256};

/// Exercise 1: Compute a SHA-256 hash.
///
/// This is a helper function used throughout the module.
///
/// Hints:
/// - Create a new `Sha256` hasher with `Sha256::new()`
/// - Feed data with `.update(data)`
/// - Finalize with `.finalize()`
/// - Convert to Vec<u8> with `.to_vec()`
pub fn sha256(data: &[u8]) -> Vec<u8> {
    todo!("Implement SHA-256 hashing")
}

/// Exercise 2: Hash preimage proof — demonstrate information leakage.
///
/// This shows the NAIVE (non-ZK) approach where the prover sends the secret directly.
/// The verifier can confirm the secret, but now also knows it.
///
/// Returns (prover_sends_secret, verifier_confirms).
///
/// Hints:
/// - Prover sends `secret` to verifier
/// - Verifier computes `sha256(secret)` and checks it equals `expected_hash`
/// - Return (secret.to_vec(), verification_result)
pub fn naive_preimage_proof(secret: &[u8], expected_hash: &[u8]) -> (Vec<u8>, bool) {
    todo!("Implement naive (non-ZK) preimage proof showing information leakage")
}

/// Exercise 3: ZK-style preimage proof — prove knowledge without revealing.
///
/// Uses a simple commitment-challenge-response protocol:
/// 1. Prover commits to random value `r`: commitment = sha256(r)
/// 2. Verifier sends challenge (hash of commitment + public_hash)
/// 3. Prover reveals based on challenge bit
///
/// For this simplified version, we use a non-interactive approach:
/// - commitment = sha256(random_nonce)
/// - challenge = sha256(commitment || public_hash)[0] & 1
/// - response = if challenge == 0 { random_nonce } else { xor_bytes(random_nonce, secret) }
///
/// Returns (commitment, challenge, response).
///
/// Hints:
/// - Use `rand::random::<[u8; 32]>()` to generate random nonce
/// - XOR two byte slices: `a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()`
/// - Challenge bit: take first byte of hash, mask with 1
pub fn zk_preimage_prove(secret: &[u8], public_hash: &[u8]) -> (Vec<u8>, u8, Vec<u8>) {
    todo!("Implement ZK-style preimage proof (non-interactive)")
}

/// Exercise 4: Verify a ZK-style preimage proof.
///
/// Given the commitment, challenge, and response, verify the proof.
///
/// Verification:
/// - If challenge == 0: sha256(response) should equal commitment
/// - If challenge == 1: sha256(response XOR public_hash) should equal commitment
///   (but we don't know public_hash preimage, so we check differently)
///
/// For this simplified protocol:
/// - Recompute challenge from commitment and public_hash
/// - If challenge == 0: sha256(response) == commitment (prover knew the nonce)
/// - If challenge == 1: sha256(response) can be used to derive a value that hashes
///   to public_hash (simplified check: response is non-empty and commitment is valid)
///
/// Returns true if the proof is valid.
///
/// Hints:
/// - Recompute challenge: hash = sha256(commitment || public_hash), take hash[0] & 1
/// - If challenge == 0: verify sha256(response) == commitment
/// - If challenge == 1: verify response is non-empty and commitment matches
pub fn zk_preimage_verify(
    commitment: &[u8],
    challenge: u8,
    response: &[u8],
    public_hash: &[u8],
) -> bool {
    todo!("Implement ZK preimage proof verification")
}

/// Exercise 5: XOR helper — XOR two byte slices.
///
/// Used in ZK protocols to combine secrets with random masks.
///
/// Hints:
/// - Zip the two slices and XOR corresponding bytes
/// - If lengths differ, XOR up to the shorter length
pub fn xor_bytes(a: &[u8], b: &[u8]) -> Vec<u8> {
    todo!("Implement byte-wise XOR")
}

/// Exercise 6: Generate a random 32-byte nonce.
///
/// Hints:
/// - Use `rand::random::<[u8; 32]>()`
/// - Return as Vec<u8>
pub fn random_nonce() -> Vec<u8> {
    todo!("Generate random 32-byte nonce")
}

/// Exercise 7: Multiple rounds of ZK proof (increase soundness).
///
/// Single-round proof has 50% soundness error. Multiple rounds reduce it to (1/2)^n.
/// Run `num_rounds` rounds and return all (commitment, challenge, response) tuples.
///
/// Hints:
/// - For each round, call `zk_preimage_prove` with the same secret and public_hash
/// - Collect results into a Vec
pub fn zk_preimage_multi_round(
    secret: &[u8],
    public_hash: &[u8],
    num_rounds: usize,
) -> Vec<(Vec<u8>, u8, Vec<u8>)> {
    todo!("Implement multi-round ZK proof")
}

/// Exercise 8: Verify multiple rounds of ZK proof.
///
/// All rounds must pass for the proof to be valid.
///
/// Hints:
/// - Iterate over rounds, call `zk_preimage_verify` for each
/// - Return true only if ALL rounds verify
pub fn zk_multi_round_verify(
    rounds: &[(Vec<u8>, u8, Vec<u8>)],
    public_hash: &[u8],
) -> bool {
    todo!("Implement multi-round ZK proof verification")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_basic() {
        let hash = sha256(b"hello");
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_naive_preimage_leaks_secret() {
        let secret = b"my_secret_password";
        let hash = sha256(secret);
        let (leaked_secret, confirmed) = naive_preimage_proof(secret, &hash);
        assert!(confirmed, "Verifier should confirm the secret");
        assert_eq!(leaked_secret, secret, "Naive approach leaks the secret!");
    }

    #[test]
    fn test_zk_preimage_proof_structure() {
        let secret = b"my_secret_password";
        let public_hash = sha256(secret);
        let (commitment, challenge, response) = zk_preimage_prove(secret, &public_hash);
        assert!(!commitment.is_empty(), "Commitment should not be empty");
        assert!(challenge <= 1, "Challenge should be 0 or 1");
        assert!(!response.is_empty(), "Response should not be empty");
    }

    #[test]
    fn test_zk_preimage_prove_and_verify() {
        let secret = b"my_secret_password";
        let public_hash = sha256(secret);
        let proof = zk_preimage_prove(secret, &public_hash);
        let result = zk_preimage_verify(&proof.0, proof.1, &proof.2, &public_hash);
        assert!(result, "Valid ZK proof should verify");
    }

    #[test]
    fn test_xor_bytes_basic() {
        let a = vec![0xFF, 0x00, 0xAA];
        let b = vec![0xFF, 0x00, 0xAA];
        let result = xor_bytes(&a, &b);
        assert_eq!(result, vec![0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_xor_bytes_identity() {
        let a = vec![1, 2, 3, 4, 5];
        let zeros = vec![0, 0, 0, 0, 0];
        assert_eq!(xor_bytes(&a, &zeros), a);
    }

    #[test]
    fn test_random_nonce_unique() {
        let n1 = random_nonce();
        let n2 = random_nonce();
        assert_eq!(n1.len(), 32);
        assert_ne!(n1, n2, "Random nonces should be different");
    }

    #[test]
    fn test_multi_round_improves_soundness() {
        let secret = b"my_secret_password";
        let public_hash = sha256(secret);
        let rounds = zk_preimage_multi_round(secret, &public_hash, 10);
        assert_eq!(rounds.len(), 10);
        assert!(zk_multi_round_verify(&rounds, &public_hash));
    }

    #[test]
    fn test_zk_does_not_leak_secret() {
        let secret = b"my_secret_password";
        let public_hash = sha256(secret);
        let (_commitment, _challenge, response) = zk_preimage_prove(secret, &public_hash);
        // The response should NOT be the raw secret
        assert_ne!(response, secret, "ZK response should not be the raw secret");
    }
}
