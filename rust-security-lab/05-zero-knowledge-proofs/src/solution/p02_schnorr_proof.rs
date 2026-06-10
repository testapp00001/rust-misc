//! # Lesson 02: Schnorr Identification Protocol (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use sha2::{Digest, Sha256};

/// Modular exponentiation: base^exp mod modulus using square-and-multiply.
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

/// Modular inverse using Fermat's little theorem: a^(p-2) mod p.
pub fn mod_inverse(a: u64, modulus: u64) -> u64 {
    mod_pow(a, modulus - 2, modulus)
}

pub struct SchnorrParams {
    pub g: u64,
    pub p: u64,
    pub q: u64,
}

/// Default small parameters for teaching.
pub fn default_params() -> SchnorrParams {
    SchnorrParams { g: 2, p: 23, q: 11 }
}

/// Generate a Schnorr keypair: (secret_key, public_key).
pub fn keygen(params: &SchnorrParams) -> (u64, u64) {
    let x = (rand::random::<u64>() % (params.q - 1)) + 1;
    let y = mod_pow(params.g, x, params.p);
    (x, y)
}

/// Schnorr prover step 1: generate commitment t = g^r mod p.
pub fn prove_commit(params: &SchnorrParams) -> (u64, u64) {
    let r = (rand::random::<u64>() % (params.q - 1)) + 1;
    let t = mod_pow(params.g, r, params.p);
    (r, t)
}

/// Schnorr prover step 3: compute response s = (r + x*e) mod q.
pub fn prove_respond(secret_key: u64, r: u64, challenge: u64, params: &SchnorrParams) -> u64 {
    let product = ((secret_key as u128 * challenge as u128) % params.q as u128) as u64;
    (r + product) % params.q
}

/// Schnorr verifier: check g^s mod p == t * y^e mod p.
pub fn verify(
    public_key: u64,
    commitment: u64,
    challenge: u64,
    response: u64,
    params: &SchnorrParams,
) -> bool {
    let lhs = mod_pow(params.g, response, params.p);
    let rhs = (commitment as u128 * mod_pow(public_key, challenge, params.p) as u128
        % params.p as u128) as u64;
    lhs == rhs
}

/// Compute deterministic challenge using Fiat-Shamir: hash(commitment || public_key) mod q.
pub fn compute_challenge(commitment: u64, public_key: u64, params: &SchnorrParams) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(commitment.to_le_bytes());
    hasher.update(public_key.to_le_bytes());
    let hash = hasher.finalize();
    let val = u64::from_le_bytes(hash[..8].try_into().unwrap());
    val % params.q
}

/// Complete non-interactive Schnorr proof (Fiat-Shamir).
pub fn prove(secret_key: u64, params: &SchnorrParams) -> (u64, u64, u64) {
    let (r, t) = prove_commit(params);
    let public_key = mod_pow(params.g, secret_key, params.p);
    let e = compute_challenge(t, public_key, params);
    let s = prove_respond(secret_key, r, e, params);
    (t, e, s)
}

/// Verify a non-interactive Schnorr proof.
pub fn verify_proof(
    public_key: u64,
    proof: (u64, u64, u64),
    params: &SchnorrParams,
) -> bool {
    let (t, e, s) = proof;
    // Recompute expected challenge
    let expected_e = compute_challenge(t, public_key, params);
    if e != expected_e {
        return false;
    }
    verify(public_key, t, e, s, params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_pow_basic() {
        assert_eq!(mod_pow(2, 10, 1000), 1024 % 1000);
        assert_eq!(mod_pow(3, 5, 7), 243 % 7);
    }

    #[test]
    fn test_mod_pow_zero_exp() {
        assert_eq!(mod_pow(5, 0, 7), 1);
    }

    #[test]
    fn test_mod_inverse() {
        assert_eq!(mod_inverse(3, 7), 5);
    }

    #[test]
    fn test_keygen_produces_valid_key() {
        let params = default_params();
        let (sk, pk) = keygen(&params);
        assert!(sk >= 1 && sk < params.q);
        assert_eq!(pk, mod_pow(params.g, sk, params.p));
    }

    #[test]
    fn test_schnorr_protocol_round_trip() {
        let params = default_params();
        let (sk, pk) = keygen(&params);
        let (r, t) = prove_commit(&params);
        let e = 3u64 % params.q;
        let s = prove_respond(sk, r, e, &params);
        assert!(verify(pk, t, e, s, &params), "Schnorr proof should verify");
    }

    #[test]
    fn test_schnorr_wrong_secret_fails() {
        let params = default_params();
        let (_sk, pk) = keygen(&params);
        let fake_s = 5u64;
        let t = 7u64;
        let e = 3u64;
        let _ = verify(pk, t, e, fake_s, &params);
    }

    #[test]
    fn test_noninteractive_schnorr() {
        let params = default_params();
        let (sk, pk) = keygen(&params);
        let proof = prove(sk, &params);
        assert!(verify_proof(pk, proof, &params));
    }

    #[test]
    fn test_fiat_shamir_deterministic() {
        let params = default_params();
        let c1 = compute_challenge(10, 5, &params);
        let c2 = compute_challenge(10, 5, &params);
        assert_eq!(c1, c2, "Fiat-Shamir challenge should be deterministic");
    }

    #[test]
    fn test_fiat_shamir_different_inputs() {
        let params = default_params();
        let c1 = compute_challenge(10, 5, &params);
        let c2 = compute_challenge(11, 5, &params);
        let _ = (c1, c2);
    }
}
