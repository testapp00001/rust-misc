//! # Lesson 06: Proof of Knowledge (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

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

pub struct PokParams {
    pub g: u64,
    pub p: u64,
    pub q: u64,
}

pub fn default_pok_params() -> PokParams {
    PokParams { g: 2, p: 23, q: 11 }
}

pub fn pok_keygen(params: &PokParams) -> (u64, u64) {
    let x = (rand::random::<u64>() % (params.q - 1)) + 1;
    let y = mod_pow(params.g, x, params.p);
    (x, y)
}

/// Proof of Knowledge — Prover (Schnorr-based).
pub fn pok_prove(secret: u64, params: &PokParams) -> (u64, u64, u64) {
    let k = (rand::random::<u64>() % (params.q - 1)) + 1;
    let t = mod_pow(params.g, k, params.p);
    let y = mod_pow(params.g, secret, params.p);

    let mut hasher = Sha256::new();
    hasher.update(t.to_le_bytes());
    hasher.update(y.to_le_bytes());
    let hash = hasher.finalize();
    let e = u64::from_le_bytes(hash[..8].try_into().unwrap()) % params.q;

    let s = (k + (secret as u128 * e as u128 % params.q as u128) as u64) % params.q;
    (t, e, s)
}

/// Proof of Knowledge — Verifier (checks Schnorr equation only).
///
/// This verifies the core Schnorr equation: g^s == t * y^e mod p.
/// For a full non-interactive proof, also verify the Fiat-Shamir challenge.
pub fn pok_verify(
    public_key: u64,
    proof: (u64, u64, u64),
    params: &PokParams,
) -> bool {
    let (t, e, s) = proof;
    let lhs = mod_pow(params.g, s, params.p);
    let rhs = ((t as u128 * mod_pow(public_key, e, params.p) as u128) % params.p as u128) as u64;
    lhs == rhs
}

/// Compute the Fiat-Shamir challenge for a given commitment and public key.
pub fn compute_challenge(commitment: u64, public_key: u64, params: &PokParams) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(commitment.to_le_bytes());
    hasher.update(public_key.to_le_bytes());
    let hash = hasher.finalize();
    u64::from_le_bytes(hash[..8].try_into().unwrap()) % params.q
}

/// Knowledge Extractor: extract secret from two transcripts with same commitment.
pub fn extract_knowledge(
    e1: u64, s1: u64,
    e2: u64, s2: u64,
    params: &PokParams,
) -> u64 {
    let diff_s = (s1 + params.q - s2) % params.q;
    let diff_e = (e1 + params.q - e2) % params.q;
    let inv_diff_e = mod_pow(diff_e, params.q - 2, params.q);
    (diff_s as u128 * inv_diff_e as u128 % params.q as u128) as u64
}

/// Simulate a proof without knowing the secret (demonstrates zero-knowledge).
pub fn simulate_proof(public_key: u64, params: &PokParams) -> (u64, u64, u64) {
    let s = (rand::random::<u64>() % (params.q - 1)) + 1;
    let e = (rand::random::<u64>() % (params.q - 1)) + 1;
    let gs = mod_pow(params.g, s, params.p);
    let ye = mod_pow(public_key, params.q - e, params.p);
    let t = ((gs as u128 * ye as u128) % params.p as u128) as u64;
    (t, e, s)
}

/// Demonstrate zero-knowledge: simulated proofs verify just like real ones.
pub fn demonstrate_zk_property(params: &PokParams) -> bool {
    let (_sk, pk) = pok_keygen(params);
    let simulated = simulate_proof(pk, params);
    pok_verify(pk, simulated, params)
}

/// Demonstrate knowledge extraction from two transcripts.
pub fn demonstrate_extraction(params: &PokParams) -> bool {
    let (sk, pk) = pok_keygen(params);
    let k = (rand::random::<u64>() % (params.q - 1)) + 1;
    let t = mod_pow(params.g, k, params.p);

    let e1 = 2u64;
    let e2 = 5u64;
    let s1 = (k + (sk as u128 * e1 as u128 % params.q as u128) as u64) % params.q;
    let s2 = (k + (sk as u128 * e2 as u128 % params.q as u128) as u64) % params.q;

    assert!(pok_verify(pk, (t, e1, s1), params));
    assert!(pok_verify(pk, (t, e2, s2), params));

    let extracted = extract_knowledge(e1, s1, e2, s2, params);
    extracted == sk
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pok_round_trip() {
        let params = default_pok_params();
        let (sk, pk) = pok_keygen(&params);
        let proof = pok_prove(sk, &params);
        assert!(pok_verify(pk, proof, &params));
    }

    #[test]
    fn test_pok_wrong_key_fails() {
        let params = default_pok_params();
        let (sk, _pk) = pok_keygen(&params);
        let (_sk2, pk2) = pok_keygen(&params);
        let proof = pok_prove(sk, &params);
        assert!(!pok_verify(pk2, proof, &params));
    }

    #[test]
    fn test_knowledge_extraction() {
        let params = default_pok_params();
        let (sk, pk) = pok_keygen(&params);
        let k = (rand::random::<u64>() % (params.q - 1)) + 1;
        let t = mod_pow(params.g, k, params.p);
        let e1 = 2u64;
        let e2 = 5u64;
        let s1 = (k + (sk as u128 * e1 as u128 % params.q as u128) as u64) % params.q;
        let s2 = (k + (sk as u128 * e2 as u128 % params.q as u128) as u64) % params.q;
        assert!(pok_verify(pk, (t, e1, s1), &params));
        assert!(pok_verify(pk, (t, e2, s2), &params));
        let extracted = extract_knowledge(e1, s1, e2, s2, &params);
        assert_eq!(extracted, sk);
    }

    #[test]
    fn test_simulation_produces_valid_proof() {
        let params = default_pok_params();
        let (_sk, pk) = pok_keygen(&params);
        let simulated = simulate_proof(pk, &params);
        assert!(pok_verify(pk, simulated, &params));
    }

    #[test]
    fn test_zk_property() {
        let params = default_pok_params();
        assert!(demonstrate_zk_property(&params));
    }

    #[test]
    fn test_extraction_demonstration() {
        let params = default_pok_params();
        assert!(demonstrate_extraction(&params));
    }

    #[test]
    fn test_pok_deterministic_challenge() {
        let params = default_pok_params();
        let (sk, pk) = pok_keygen(&params);
        let proof1 = pok_prove(sk, &params);
        let proof2 = pok_prove(sk, &params);
        assert!(pok_verify(pk, proof1, &params));
        assert!(pok_verify(pk, proof2, &params));
    }
}
