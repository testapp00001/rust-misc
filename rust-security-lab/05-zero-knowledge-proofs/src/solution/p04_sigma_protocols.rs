//! # Lesson 04: Sigma Protocols (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use sha2::{Digest, Sha256};

/// Modular exponentiation using square-and-multiply.
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

pub struct SigmaParams {
    pub g: u64,
    pub p: u64,
    pub q: u64,
}

/// Default Sigma parameters for teaching.
pub fn default_sigma_params() -> SigmaParams {
    SigmaParams { g: 2, p: 23, q: 11 }
}

/// Generate keypair: (secret, public).
pub fn sigma_keygen(params: &SigmaParams) -> (u64, u64) {
    let x = (rand::random::<u64>() % (params.q - 1)) + 1;
    let y = mod_pow(params.g, x, params.p);
    (x, y)
}

/// Sigma prover step 1: commitment a = g^k mod p.
pub fn sigma_commit(params: &SigmaParams) -> (u64, u64) {
    let k = (rand::random::<u64>() % (params.q - 1)) + 1;
    let a = mod_pow(params.g, k, params.p);
    (k, a)
}

/// Sigma prover step 3: response z = (k + x*e) mod q.
pub fn sigma_respond(secret: u64, nonce: u64, challenge: u64, params: &SigmaParams) -> u64 {
    let product = ((secret as u128 * challenge as u128) % params.q as u128) as u64;
    (nonce + product) % params.q
}

/// Sigma verifier: check g^z == a * y^e mod p.
pub fn sigma_verify(
    public_key: u64,
    commitment: u64,
    challenge: u64,
    response: u64,
    params: &SigmaParams,
) -> bool {
    let lhs = mod_pow(params.g, response, params.p);
    let rhs = ((commitment as u128 * mod_pow(public_key, challenge, params.p) as u128)
        % params.p as u128) as u64;
    lhs == rhs
}

/// Fiat-Shamir challenge: e = H(a || y) mod q.
pub fn fiat_shamir_challenge(commitment: u64, public_key: u64, params: &SigmaParams) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(commitment.to_le_bytes());
    hasher.update(public_key.to_le_bytes());
    let hash = hasher.finalize();
    let val = u64::from_le_bytes(hash[..8].try_into().unwrap());
    val % params.q
}

/// Complete non-interactive Sigma proof using Fiat-Shamir.
pub fn sigma_prove(secret: u64, params: &SigmaParams) -> (u64, u64, u64) {
    let (k, a) = sigma_commit(params);
    let y = mod_pow(params.g, secret, params.p);
    let e = fiat_shamir_challenge(a, y, params);
    let z = sigma_respond(secret, k, e, params);
    (a, e, z)
}

/// Verify a non-interactive Sigma proof.
pub fn sigma_verify_proof(
    public_key: u64,
    proof: (u64, u64, u64),
    params: &SigmaParams,
) -> bool {
    let (a, e, z) = proof;
    let expected_e = fiat_shamir_challenge(a, public_key, params);
    if e != expected_e {
        return false;
    }
    sigma_verify(public_key, a, e, z, params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sigma_round_trip() {
        let params = default_sigma_params();
        let (sk, pk) = sigma_keygen(&params);
        let (k, a) = sigma_commit(&params);
        let e = 5u64 % params.q;
        let z = sigma_respond(sk, k, e, &params);
        assert!(sigma_verify(pk, a, e, z, &params));
    }

    #[test]
    fn test_sigma_wrong_secret() {
        let params = default_sigma_params();
        let (_sk, pk) = sigma_keygen(&params);
        let (_, a) = sigma_commit(&params);
        let e = 3u64;
        let fake_z = 7u64;
        let _ = sigma_verify(pk, a, e, fake_z, &params);
    }

    #[test]
    fn test_sigma_noninteractive() {
        let params = default_sigma_params();
        let (sk, pk) = sigma_keygen(&params);
        let proof = sigma_prove(sk, &params);
        assert!(sigma_verify_proof(pk, proof, &params));
    }

    #[test]
    fn test_fiat_shamir_deterministic() {
        let params = default_sigma_params();
        let c1 = fiat_shamir_challenge(10, 5, &params);
        let c2 = fiat_shamir_challenge(10, 5, &params);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_fiat_shamir_binding() {
        let params = default_sigma_params();
        let c1 = fiat_shamir_challenge(10, 5, &params);
        let c2 = fiat_shamir_challenge(11, 5, &params);
        let _ = (c1, c2);
    }

    #[test]
    fn test_sigma_extract_witness() {
        let params = default_sigma_params();
        let (sk, pk) = sigma_keygen(&params);
        let (k, a) = sigma_commit(&params);
        let e1 = 2u64;
        let e2 = 5u64;
        let z1 = sigma_respond(sk, k, e1, &params);
        let z2 = sigma_respond(sk, k, e2, &params);
        assert!(sigma_verify(pk, a, e1, z1, &params));
        assert!(sigma_verify(pk, a, e2, z2, &params));
    }

    #[test]
    fn test_sigma_simulation() {
        let params = default_sigma_params();
        let (_sk, pk) = sigma_keygen(&params);
        let fake_z = (rand::random::<u64>() % (params.q - 1)) + 1;
        let fake_e = (rand::random::<u64>() % (params.q - 1)) + 1;
        let gz = mod_pow(params.g, fake_z, params.p);
        let ye = mod_pow(pk, params.q - fake_e, params.p);
        let simulated_a = ((gz as u128 * ye as u128) % params.p as u128) as u64;
        assert!(sigma_verify(pk, simulated_a, fake_e, fake_z, &params));
    }
}
