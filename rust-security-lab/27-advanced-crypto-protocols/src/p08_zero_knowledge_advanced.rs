//! # Lesson 08: Zero-Knowledge for Arbitrary Statements -- Prove Anything
//!
//! ## Beyond Simple ZK Proofs
//!
//! In Module 05, you learned ZK proofs for discrete log and hash preimages. Here we extend
//! to proving **arbitrary NP statements**: any statement where a witness can be efficiently
//! verified.
//!
//! ## NP Statements
//!
//! An NP statement has the form: "There exists x such that R(x, w) = true"
//! where w is the witness (secret) and R is a polynomial-time relation.
//!
//! Examples:
//! - "I know the factors of N" (witness: p, q where p*q = N)
//! - "I know a Hamiltonian cycle in graph G" (witness: the cycle)
//! - "I know a valid signature on message m" (witness: the signing key)
//! - "I know a secret s such that H(s) = h" (witness: s)
//!
//! ## The Sigma Protocol Framework
//!
//! Many ZK proofs follow the 3-move Sigma protocol:
//! 1. **Commitment** (Prover -> Verifier): P sends a random commitment t
//! 2. **Challenge** (Verifier -> Prover): V sends a random challenge c
//! 3. **Response** (Prover -> Verifier): P sends response r = f(w, t, c)
//!
//! The verifier checks a relation involving (t, c, r).
//!
//! ## Fiat-Shamir Heuristic
//!
//! To make a Sigma protocol non-interactive:
//! - Replace the random challenge c with a hash: c = H(t || public_params)
//! - This turns a 3-message protocol into a single message (a "proof")
//!
//! ## Schnorr Protocol (ZK Proof of Discrete Log)
//!
//! Given public key Y = g^x, prove knowledge of x:
//! 1. P picks random r, sends T = g^r
//! 2. V sends challenge c
//! 3. P sends s = r + c*x  (mod q)
//! 4. V checks: g^s == T * Y^c
//!
//! ## Attack: Replay
//!
//! Without fresh randomness, replaying a proof allows extraction of the witness.
//! Each proof must use a fresh random nonce.

use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};

/// Public parameters for the ZK proof system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZKParams {
    /// Prime modulus
    pub p: i64,
    /// Group order (prime dividing p-1)
    pub q: i64,
    /// Generator of the subgroup of order q
    pub g: i64,
}

/// A Schnorr proof (Sigma protocol for discrete log).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchnorrProof {
    /// Commitment: T = g^r mod p
    pub commitment: i64,
    /// Challenge: c = H(T || Y || g) (Fiat-Shamir)
    pub challenge: i64,
    /// Response: s = r + c*x mod q
    pub response: i64,
}

/// A ZK proof of knowledge of a hash preimage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashPreimageProof {
    /// Commitment to random nonce
    pub commitment: Vec<u8>,
    /// Challenge (Fiat-Shamir)
    pub challenge: Vec<u8>,
    /// Response combining nonce and secret
    pub response: Vec<u8>,
}

/// A ZK proof that a committed value lies in a range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeProof {
    /// Commitment to the value
    pub commitment: i64,
    /// Proof that value >= min (bit decomposition proof)
    pub lower_bound_proof: Vec<u8>,
    /// Proof that value <= max (bit decomposition proof)
    pub upper_bound_proof: Vec<u8>,
}

/// Exercise 1: Generate ZK parameters.
///
/// Choose a safe prime p (where q = (p-1)/2 is also prime), and a generator g
/// of the subgroup of order q.
///
/// For simplicity, use p = 2027, q = 1013, g = 2 (verify g^q mod p == 1).
pub fn generate_zk_params() -> ZKParams {
    todo!("Generate ZK proof parameters (p, q, g)")
}

/// Exercise 2: Create a Schnorr proof (prover side).
///
/// Given secret x and public key Y = g^x mod p:
/// 1. Pick random r in [1, q)
/// 2. Compute T = g^r mod p
/// 3. Compute challenge c = H(T || Y) mod q  (Fiat-Shamir)
/// 4. Compute response s = (r + c * x) mod q
///
/// Return the proof (T, c, s).
pub fn schnorr_prove(
    secret_x: i64,
    public_y: i64,
    params: &ZKParams,
) -> SchnorrProof {
    todo!("Create a Schnorr ZK proof of discrete log knowledge")
}

/// Exercise 3: Verify a Schnorr proof (verifier side).
///
/// Given public key Y and proof (T, c, s):
/// 1. Recompute c' = H(T || Y) mod q
/// 2. Check c' == c
/// 3. Check g^s mod p == T * Y^c mod p
///
/// Return true if the proof is valid.
pub fn schnorr_verify(
    public_y: i64,
    proof: &SchnorrProof,
    params: &ZKParams,
) -> bool {
    todo!("Verify a Schnorr ZK proof")
}

/// Exercise 4: ZK proof of hash preimage knowledge.
///
/// Prove: "I know s such that SHA-256(s) = h" without revealing s.
///
/// Simplified protocol:
/// 1. Pick random nonce r
/// 2. Commitment = SHA-256(r)
/// 3. Challenge = SHA-256(commitment || h)
/// 4. Response = s XOR SHA-256(r || challenge)  (simplified, not fully secure)
///
/// Return the proof.
pub fn hash_preimage_prove(secret: &[u8], target_hash: &[u8]) -> HashPreimageProof {
    todo!("Prove knowledge of hash preimage in ZK")
}

/// Exercise 5: Verify a hash preimage proof.
///
/// Check that the proof is well-formed and consistent.
/// (In a real system, this would verify the algebraic relation.)
pub fn hash_preimage_verify(
    target_hash: &[u8],
    proof: &HashPreimageProof,
) -> bool {
    todo!("Verify a ZK proof of hash preimage knowledge")
}

/// Exercise 6: Prove a value is in a range [min, max].
///
/// Simplified range proof using bit decomposition:
/// 1. The prover shows that (value - min) has a non-negative representation
/// 2. And (max - value) has a non-negative representation
///
/// For simplicity, prove value is in [0, 255] by showing each bit is 0 or 1.
/// Use a commitment to each bit with a proof that the bit is binary.
pub fn range_prove(value: i64, min: i64, max: i64, params: &ZKParams) -> RangeProof {
    todo!("Prove that a committed value is in a range")
}

/// Exercise 7: Verify a range proof.
pub fn range_verify(proof: &RangeProof, min: i64, max: i64, params: &ZKParams) -> bool {
    todo!("Verify a range proof")
}

/// Helper: hash to a value mod q.
pub fn hash_to_field(data: &[u8], q: i64) -> i64 {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    let val = i64::from_le_bytes(hash[..8].try_into().unwrap()).abs();
    val % q
}

/// Helper: fast modular exponentiation.
pub fn mod_pow(mut base: i64, mut exp: i64, m: i64) -> i64 {
    if m == 1 {
        return 0;
    }
    let mut result = 1i64;
    base = ((base % m) + m) % m;
    while exp > 0 {
        if exp % 2 == 1 {
            result = result * base % m;
        }
        exp >>= 1;
        base = base * base % m;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_params() -> ZKParams {
        generate_zk_params()
    }

    #[test]
    fn test_generate_params() {
        let params = test_params();
        assert!(params.p > 0);
        assert!(params.q > 0);
        assert!(params.g > 0);
        // g^q mod p should be 1 (g generates subgroup of order q)
        assert_eq!(mod_pow(params.g, params.q, params.p), 1);
    }

    #[test]
    fn test_schnorr_prove_and_verify() {
        let params = test_params();
        let secret = 42;
        let public_y = mod_pow(params.g, secret, params.p);

        let proof = schnorr_prove(secret, public_y, &params);
        assert!(schnorr_verify(public_y, &proof, &params));
    }

    #[test]
    fn test_schnorr_wrong_secret() {
        let params = test_params();
        let secret = 42;
        let public_y = mod_pow(params.g, secret, params.p);

        // Try to prove with wrong secret
        let fake_proof = schnorr_prove(99, public_y, &params);
        // This should fail verification (the proof is for a different secret)
        // Note: the proof is valid for secret=99, but the public key is for secret=42
        // So verification checks g^s == T * Y^c, which won't hold
        assert!(!schnorr_verify(public_y, &fake_proof, &params));
    }

    #[test]
    fn test_schnorr_different_proofs_different() {
        let params = test_params();
        let secret = 42;
        let public_y = mod_pow(params.g, secret, params.p);

        let proof1 = schnorr_prove(secret, public_y, &params);
        let proof2 = schnorr_prove(secret, public_y, &params);
        // Different random nonces -> different proofs
        // (extremely unlikely to collide)
        assert_ne!(proof1.commitment, proof2.commitment);
    }

    #[test]
    fn test_hash_preimage_prove_and_verify() {
        let secret = b"my_secret_password";
        let mut hasher = Sha256::new();
        hasher.update(secret);
        let target = hasher.finalize().to_vec();

        let proof = hash_preimage_prove(secret, &target);
        assert!(hash_preimage_verify(&target, &proof));
    }

    #[test]
    fn test_hash_preimage_wrong_target() {
        let secret = b"my_secret_password";
        let mut hasher = Sha256::new();
        hasher.update(secret);
        let target = hasher.finalize().to_vec();

        let proof = hash_preimage_prove(secret, &target);
        let wrong_target = vec![0u8; 32];
        assert!(!hash_preimage_verify(&wrong_target, &proof));
    }

    #[test]
    fn test_range_prove_valid() {
        let params = test_params();
        let value = 42;
        let proof = range_prove(value, 0, 100, &params);
        assert!(range_verify(&proof, 0, 100, &params));
    }

    #[test]
    fn test_range_prove_boundary() {
        let params = test_params();
        // Value at boundary
        let proof = range_prove(0, 0, 255, &params);
        assert!(range_verify(&proof, 0, 255, &params));

        let proof2 = range_prove(255, 0, 255, &params);
        assert!(range_verify(&proof2, 0, 255, &params));
    }
}
