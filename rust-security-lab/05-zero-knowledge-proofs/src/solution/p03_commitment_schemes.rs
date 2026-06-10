//! # Lesson 03: Commitment Schemes — Pedersen Commitments (Reference Solution)
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

pub struct PedersenParams {
    pub g: u64,
    pub h: u64,
    pub p: u64,
    pub q: u64,
}

/// Default small Pedersen parameters for teaching.
pub fn default_pedersen_params() -> PedersenParams {
    PedersenParams { g: 2, h: 3, p: 23, q: 11 }
}

/// Compute Pedersen commitment: C = g^m * h^r mod p.
pub fn commit(m: u64, r: u64, params: &PedersenParams) -> u64 {
    let gm = mod_pow(params.g, m, params.p);
    let hr = mod_pow(params.h, r, params.p);
    ((gm as u128 * hr as u128) % params.p as u128) as u64
}

/// Generate a random blinding factor in [1, q-1].
pub fn random_blinding(params: &PedersenParams) -> u64 {
    (rand::random::<u64>() % (params.q - 1)) + 1
}

/// Verify a Pedersen commitment opening.
pub fn verify_commitment(
    m: u64,
    r: u64,
    commitment: u64,
    params: &PedersenParams,
) -> bool {
    commit(m, r, params) == commitment
}

/// Verify the homomorphic property: C(m1,r1) * C(m2,r2) = C(m1+m2, r1+r2).
pub fn verify_homomorphic(
    m1: u64, r1: u64, c1: u64,
    m2: u64, r2: u64, c2: u64,
    params: &PedersenParams,
) -> (u64, bool) {
    // Product of commitments
    let product = ((c1 as u128 * c2 as u128) % params.p as u128) as u64;
    // Commitment to the sum
    let sum_commitment = commit(m1 + m2, r1 + r2, params);
    (product, product == sum_commitment)
}

/// Simple hash-based commitment (not homomorphic).
pub fn hash_commit(m: u64) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(m.to_le_bytes());
    hasher.finalize().to_vec()
}

/// Verify hash-based commitment.
pub fn verify_hash_commit(m: u64, commitment: &[u8]) -> bool {
    hash_commit(m) == commitment
}

/// Demonstrate that hash commitments are NOT homomorphic.
pub fn hash_not_homomorphic(m1: u64, m2: u64) -> (Vec<u8>, Vec<u8>) {
    (hash_commit(m1), hash_commit(m2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pedersen_commit_and_verify() {
        let params = default_pedersen_params();
        let m = 5u64;
        let r = 3u64;
        let c = commit(m, r, &params);
        assert!(verify_commitment(m, r, c, &params));
    }

    #[test]
    fn test_pedersen_hiding() {
        let params = default_pedersen_params();
        let m = 5u64;
        let r1 = 3u64;
        let r2 = 7u64;
        let c1 = commit(m, r1, &params);
        let c2 = commit(m, r2, &params);
        assert_ne!(c1, c2, "Different blinding factors should produce different commitments");
    }

    #[test]
    fn test_pedersen_binding() {
        let params = default_pedersen_params();
        let m = 5u64;
        let r = 3u64;
        let c = commit(m, r, &params);
        let wrong_m = 6u64;
        assert!(!verify_commitment(wrong_m, r, c, &params));
    }

    #[test]
    fn test_pedersen_homomorphic() {
        let params = default_pedersen_params();
        let m1 = 3u64;
        let r1 = 2u64;
        let m2 = 4u64;
        let r2 = 5u64;
        let c1 = commit(m1, r1, &params);
        let c2 = commit(m2, r2, &params);
        let (product, is_homomorphic) = verify_homomorphic(m1, r1, c1, m2, r2, c2, &params);
        assert!(is_homomorphic, "Pedersen commitments should be homomorphic");
        assert!(product > 0);
    }

    #[test]
    fn test_hash_commit_and_verify() {
        let m = 42u64;
        let c = hash_commit(m);
        assert!(!c.is_empty());
        assert!(verify_hash_commit(m, &c));
    }

    #[test]
    fn test_hash_commit_wrong_message() {
        let m = 42u64;
        let c = hash_commit(m);
        assert!(!verify_hash_commit(43, &c));
    }

    #[test]
    fn test_hash_not_homomorphic() {
        let (h1, h2) = hash_not_homomorphic(3, 4);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_random_blinding_varies() {
        let params = default_pedersen_params();
        let r1 = random_blinding(&params);
        let r2 = random_blinding(&params);
        assert!(r1 >= 1 && r1 < params.q);
        assert!(r2 >= 1 && r2 < params.q);
    }
}
