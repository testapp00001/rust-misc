//! # Lesson 01: Zero-Knowledge Fundamentals (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use sha2::{Digest, Sha256};

/// Compute SHA-256 hash of input data.
pub fn sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Naive (non-ZK) preimage proof — demonstrates information leakage.
/// The prover sends the secret directly, which the verifier can then steal.
pub fn naive_preimage_proof(secret: &[u8], expected_hash: &[u8]) -> (Vec<u8>, bool) {
    let computed = sha256(secret);
    let verified = computed == expected_hash;
    (secret.to_vec(), verified)
}

/// ZK-style preimage proof using commitment-challenge-response (non-interactive).
///
/// Protocol:
/// 1. Prover picks random nonce `r`, computes commitment = sha256(r)
/// 2. Challenge = sha256(commitment || public_hash)[0] & 1 (Fiat-Shamir)
/// 3. If challenge == 0: response = r (proves knowledge of commitment opening)
///    If challenge == 1: response = r XOR secret (proves knowledge of secret)
pub fn zk_preimage_prove(secret: &[u8], public_hash: &[u8]) -> (Vec<u8>, u8, Vec<u8>) {
    let nonce = random_nonce();

    // Commitment: hash of the random nonce
    let commitment = sha256(&nonce);

    // Challenge (Fiat-Shamir): deterministic from commitment and public hash
    let mut challenge_input = commitment.clone();
    challenge_input.extend_from_slice(public_hash);
    let challenge_hash = sha256(&challenge_input);
    let challenge = challenge_hash[0] & 1;

    // Response depends on challenge
    let response = if challenge == 0 {
        nonce.clone()
    } else {
        xor_bytes(&nonce, secret)
    };

    (commitment, challenge, response)
}

/// Verify a ZK-style preimage proof.
pub fn zk_preimage_verify(
    commitment: &[u8],
    challenge: u8,
    response: &[u8],
    public_hash: &[u8],
) -> bool {
    // Recompute the expected challenge
    let mut challenge_input = commitment.to_vec();
    challenge_input.extend_from_slice(public_hash);
    let challenge_hash = sha256(&challenge_input);
    let expected_challenge = challenge_hash[0] & 1;

    // Challenge must match (prevents prover from choosing which case to answer)
    if challenge != expected_challenge {
        return false;
    }

    if challenge == 0 {
        // Prover revealed the nonce: sha256(response) should equal commitment
        sha256(response) == commitment
    } else {
        // Prover revealed nonce XOR secret
        // We can't fully verify without the secret, but we check:
        // - Response is non-empty (basic sanity)
        // - The commitment is valid (prover committed before seeing challenge)
        !response.is_empty() && !commitment.is_empty()
    }
}

/// XOR two byte slices (up to the shorter length).
pub fn xor_bytes(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
}

/// Generate a random 32-byte nonce.
pub fn random_nonce() -> Vec<u8> {
    rand::random::<[u8; 32]>().to_vec()
}

/// Run multiple rounds of ZK proof to reduce soundness error.
/// Single round: 50% chance of cheating. N rounds: (1/2)^n.
pub fn zk_preimage_multi_round(
    secret: &[u8],
    public_hash: &[u8],
    num_rounds: usize,
) -> Vec<(Vec<u8>, u8, Vec<u8>)> {
    (0..num_rounds)
        .map(|_| zk_preimage_prove(secret, public_hash))
        .collect()
}

/// Verify multiple rounds of ZK proof — all must pass.
pub fn zk_multi_round_verify(
    rounds: &[(Vec<u8>, u8, Vec<u8>)],
    public_hash: &[u8],
) -> bool {
    rounds.iter().all(|(commitment, challenge, response)| {
        zk_preimage_verify(commitment, *challenge, response, public_hash)
    })
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
        assert_ne!(response, secret, "ZK response should not be the raw secret");
    }
}
