//! # Lesson 03: Commitment Schemes — Pedersen Commitments
//!
//! ## What is a Commitment Scheme?
//!
//! A commitment scheme is like putting a message in a locked box and giving someone the box.
//! Later, you can open the box to reveal the message.
//!
//! Two properties:
//! - **Hiding**: The commitment reveals nothing about the message (the box is opaque)
//! - **Binding**: You can't change the message after committing (the lock can't be picked)
//!
//! ## Pedersen Commitment
//!
//! C = g^m * h^r mod p
//!
//! Where:
//! - `m` is the message (value being committed to)
//! - `r` is a random blinding factor
//! - `g` and `h` are public generators where nobody knows log_g(h)
//!
//! Why it's **hiding**: Without knowing `r`, the commitment `C` could correspond to any message.
//! Why it's **binding**: To open to a different message `m'`, you'd need to find `r'` such that
//! `g^m * h^r = g^m' * h^r'`, which requires solving discrete log.
//!
//! ## ATTACK: Simple Hash Commitment
//!
//! `commit(m) = SHA256(m)` — is this hiding? Yes. Is it binding? Yes, if hash is collision-resistant.
//! But it's NOT **additively homomorphic**: you can't combine two commitments to get a commitment
//! to the sum. Pedersen commitments ARE homomorphic, which is crucial for many ZK protocols.
//!
//! ## Real-World Applications
//!
//! - **Blockchain**: Commit to transaction amounts before revealing
//! - **Coin flipping protocols**: Commit to a choice, then reveal
//! - **Sealed-bid auctions**: Commit to your bid, then reveal all bids
//! - **Zero-knowledge proofs**: Building block for range proofs, Bulletproofs

use sha2::{Digest, Sha256};

/// Modular exponentiation (reuse from p02).
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

/// Pedersen commitment parameters.
pub struct PedersenParams {
    pub g: u64,
    pub h: u64,
    pub p: u64,
    pub q: u64,
}

/// Exercise 1: Create Pedersen parameters.
///
/// For teaching, use small values: g=2, h=3, p=23, q=11.
/// In practice, use large primes where log_g(h) is unknown.
///
/// Hints:
/// - Return PedersenParams { g: 2, h: 3, p: 23, q: 11 }
pub fn default_pedersen_params() -> PedersenParams {
    todo!("Create default Pedersen parameters")
}

/// Exercise 2: Compute a Pedersen commitment.
///
/// C = g^m * h^r mod p
///
/// Hints:
/// - Compute `g^m mod p` using mod_pow
/// - Compute `h^r mod p` using mod_pow
/// - Multiply them and take mod p
/// - Use u128 intermediate to avoid overflow
pub fn commit(m: u64, r: u64, params: &PedersenParams) -> u64 {
    todo!("Compute Pedersen commitment C = g^m * h^r mod p")
}

/// Exercise 3: Generate a random blinding factor.
///
/// The blinding factor must be random and secret — it's what makes the commitment hiding.
///
/// Hints:
/// - Random value in [1, q-1]
pub fn random_blinding(params: &PedersenParams) -> u64 {
    todo!("Generate random blinding factor")
}

/// Exercise 4: Verify a Pedersen commitment opening.
///
/// Given (m, r, commitment), verify that commit(m, r) == commitment.
///
/// Hints:
/// - Recompute the commitment using `commit(m, r, params)`
/// - Compare with the given commitment
pub fn verify_commitment(
    m: u64,
    r: u64,
    commitment: u64,
    params: &PedersenParams,
) -> bool {
    todo!("Verify Pedersen commitment opening")
}

/// Exercise 5: Demonstrate the homomorphic property.
///
/// Pedersen commitments are additively homomorphic:
/// C(m1, r1) * C(m2, r2) = C(m1 + m2, r1 + r2)
///
/// Given two commitments and their openings, verify the homomorphic property.
/// Returns the product commitment and whether it equals C(m1+m2, r1+r2).
///
/// Hints:
/// - Product: `(c1 * c2) % p`
/// - Sum commitment: `commit(m1 + m2, r1 + r2, params)`
/// - Check they're equal
pub fn verify_homomorphic(
    m1: u64, r1: u64, c1: u64,
    m2: u64, r2: u64, c2: u64,
    params: &PedersenParams,
) -> (u64, bool) {
    todo!("Verify Pedersen homomorphic property")
}

/// Exercise 6: Simple hash-based commitment (for comparison).
///
/// commit(m) = SHA256(m_bytes)
/// This is hiding and binding, but NOT homomorphic.
///
/// Hints:
/// - Convert m to bytes
/// - Hash with SHA-256
pub fn hash_commit(m: u64) -> Vec<u8> {
    todo!("Compute hash-based commitment")
}

/// Exercise 7: Verify hash-based commitment.
///
/// Hints:
/// - Recompute hash_commit(m)
/// - Compare with given commitment
pub fn verify_hash_commit(m: u64, commitment: &[u8]) -> bool {
    todo!("Verify hash-based commitment")
}

/// Exercise 8: Demonstrate that hash commitments are NOT homomorphic.
///
/// Returns (hash_commit(m1) * hash_commit(m2), hash_commit(m1 + m2))
/// showing they are not equal (even conceptually — hash outputs aren't numbers to multiply).
///
/// Hints:
/// - Just compute both and return them for comparison
pub fn hash_not_homomorphic(m1: u64, m2: u64) -> (Vec<u8>, Vec<u8>) {
    todo!("Show hash commitments are not homomorphic")
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
        // Same message, different blinding factors => different commitments
        assert_ne!(c1, c2, "Different blinding factors should produce different commitments");
    }

    #[test]
    fn test_pedersen_binding() {
        let params = default_pedersen_params();
        let m = 5u64;
        let r = 3u64;
        let c = commit(m, r, &params);
        // Different message should not verify with same commitment
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
        let (h_product, h_sum) = hash_not_homomorphic(3, 4);
        // These should be different (hash outputs aren't algebraically related)
        assert_ne!(h_product, h_sum);
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
