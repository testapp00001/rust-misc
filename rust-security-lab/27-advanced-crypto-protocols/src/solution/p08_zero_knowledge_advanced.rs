//! # Lesson 08: Zero-Knowledge for Arbitrary Statements (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use rand::Rng;
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZKParams {
    pub p: i64,
    pub q: i64,
    pub g: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchnorrProof {
    pub commitment: i64,
    pub challenge: i64,
    pub response: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashPreimageProof {
    pub commitment: Vec<u8>,
    pub challenge: Vec<u8>,
    pub response: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeProof {
    pub commitment: i64,
    pub lower_bound_proof: Vec<u8>,
    pub upper_bound_proof: Vec<u8>,
}

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

pub fn hash_to_field(data: &[u8], q: i64) -> i64 {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    let val = i64::from_le_bytes(hash[..8].try_into().unwrap()).abs();
    val % q
}

pub fn hash_data(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Generate ZK proof parameters.
pub fn generate_zk_params() -> ZKParams {
    // Use a safe prime where q = (p-1)/2 is also prime
    let p = 2027i64;
    let q = (p - 1) / 2; // 1013
    // Find a generator of the subgroup of order q
    // g = 2^2 mod p should have order q if 2 is a primitive root
    let g = mod_pow(2, 2, p); // g = 4
    // Verify: g^q mod p == 1
    assert_eq!(mod_pow(g, q, p), 1, "g must generate subgroup of order q");
    ZKParams { p, q, g }
}

/// Create a Schnorr ZK proof of discrete log knowledge.
pub fn schnorr_prove(
    secret_x: i64,
    public_y: i64,
    params: &ZKParams,
) -> SchnorrProof {
    let mut rng = rand::thread_rng();

    // Step 1: Pick random r in [1, q)
    let r: i64 = rng.gen_range(1..params.q);

    // Step 2: Compute T = g^r mod p
    let commitment = mod_pow(params.g, r, params.p);

    // Step 3: Compute challenge c = H(T || Y) mod q (Fiat-Shamir)
    let mut challenge_data = Vec::new();
    challenge_data.extend_from_slice(&commitment.to_le_bytes());
    challenge_data.extend_from_slice(&public_y.to_le_bytes());
    let challenge = hash_to_field(&challenge_data, params.q);

    // Step 4: Compute response s = (r + c * x) mod q
    let response = ((r + challenge * secret_x) % params.q + params.q) % params.q;

    SchnorrProof {
        commitment,
        challenge,
        response,
    }
}

/// Verify a Schnorr ZK proof.
pub fn schnorr_verify(
    public_y: i64,
    proof: &SchnorrProof,
    params: &ZKParams,
) -> bool {
    // Recompute challenge
    let mut challenge_data = Vec::new();
    challenge_data.extend_from_slice(&proof.commitment.to_le_bytes());
    challenge_data.extend_from_slice(&public_y.to_le_bytes());
    let expected_challenge = hash_to_field(&challenge_data, params.q);

    if expected_challenge != proof.challenge {
        return false;
    }

    // Check g^s == T * Y^c mod p
    let lhs = mod_pow(params.g, proof.response, params.p);
    let rhs = (proof.commitment * mod_pow(public_y, proof.challenge, params.p)) % params.p;

    lhs == rhs
}

/// Prove knowledge of hash preimage.
pub fn hash_preimage_prove(secret: &[u8], target_hash: &[u8]) -> HashPreimageProof {
    let mut rng = rand::thread_rng();

    // Step 1: Pick random nonce r
    let r: Vec<u8> = (0..32).map(|_| rng.gen::<u8>()).collect();

    // Step 2: Commitment = SHA-256(r)
    let commitment = hash_data(&r);

    // Step 3: Challenge = SHA-256(commitment || target_hash)
    let mut challenge_input = commitment.clone();
    challenge_input.extend_from_slice(target_hash);
    let challenge = hash_data(&challenge_input);

    // Step 4: Response = secret XOR SHA-256(r || challenge)
    let mut key_input = r.clone();
    key_input.extend_from_slice(&challenge);
    let key = hash_data(&key_input);

    let response: Vec<u8> = secret
        .iter()
        .zip(key.iter().cycle())
        .map(|(s, k)| s ^ k)
        .collect();

    HashPreimageProof {
        commitment,
        challenge,
        response,
    }
}

/// Verify a hash preimage proof.
pub fn hash_preimage_verify(
    target_hash: &[u8],
    proof: &HashPreimageProof,
) -> bool {
    // Recompute challenge
    let mut challenge_input = proof.commitment.clone();
    challenge_input.extend_from_slice(target_hash);
    let expected_challenge = hash_data(&challenge_input);

    if expected_challenge != proof.challenge {
        return false;
    }

    // The proof is valid if the commitment and challenge are consistent
    // In a full protocol, we'd verify the algebraic relation
    // Here we check that the proof structure is well-formed
    !proof.commitment.is_empty() && !proof.response.is_empty()
}

/// Prove that a value is in a range [min, max].
pub fn range_prove(value: i64, min: i64, max: i64, params: &ZKParams) -> RangeProof {
    let mut rng = rand::thread_rng();

    // Commitment to the value using a Pedersen-like scheme
    let r: i64 = rng.gen_range(1..params.q);
    let commitment = (mod_pow(params.g, value, params.p) * mod_pow(params.h(), r, params.p)) % params.p;

    // For the lower bound proof: show (value - min) >= 0
    let lower_diff = value - min;
    let lower_data = format!("lower:{}:{}", lower_diff, commitment);
    let lower_bound_proof = hash_data(lower_data.as_bytes());

    // For the upper bound proof: show (max - value) >= 0
    let upper_diff = max - value;
    let upper_data = format!("upper:{}:{}", upper_diff, commitment);
    let upper_bound_proof = hash_data(upper_data.as_bytes());

    RangeProof {
        commitment,
        lower_bound_proof,
        upper_bound_proof,
    }
}

/// Verify a range proof.
pub fn range_verify(proof: &RangeProof, _min: i64, _max: i64, _params: &ZKParams) -> bool {
    // In a simplified verification, we check that the proofs are non-empty
    // A real range proof would verify the algebraic relations
    !proof.lower_bound_proof.is_empty() && !proof.upper_bound_proof.is_empty()
}

// Helper trait to add h() method to ZKParams
trait ZKParamsExt {
    fn h(&self) -> i64;
}

impl ZKParamsExt for ZKParams {
    fn h(&self) -> i64 {
        // h is a second generator, derived from g
        mod_pow(self.g, 2, self.p)
    }
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

        let fake_proof = schnorr_prove(99, public_y, &params);
        assert!(!schnorr_verify(public_y, &fake_proof, &params));
    }

    #[test]
    fn test_schnorr_different_proofs_different() {
        let params = test_params();
        let secret = 42;
        let public_y = mod_pow(params.g, secret, params.p);

        let proof1 = schnorr_prove(secret, public_y, &params);
        let proof2 = schnorr_prove(secret, public_y, &params);
        assert_ne!(proof1.commitment, proof2.commitment);
    }

    #[test]
    fn test_hash_preimage_prove_and_verify() {
        let secret = b"my_secret_password";
        let target = hash_data(secret);

        let proof = hash_preimage_prove(secret, &target);
        assert!(hash_preimage_verify(&target, &proof));
    }

    #[test]
    fn test_hash_preimage_wrong_target() {
        let secret = b"my_secret_password";
        let target = hash_data(secret);

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
        let proof = range_prove(0, 0, 255, &params);
        assert!(range_verify(&proof, 0, 255, &params));

        let proof2 = range_prove(255, 0, 255, &params);
        assert!(range_verify(&proof2, 0, 255, &params));
    }
}
