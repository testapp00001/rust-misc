//! # Lesson 07: Commitment Schemes Advanced -- Pedersen and ElGamal Commitments
//!
//! ## What is a Commitment Scheme?
//!
//! A commitment scheme lets you "lock" a value without revealing it, then later "open" it.
//! Like putting a message in a locked box, showing the box, and later providing the key.
//!
//! Two phases:
//! 1. **Commit(v, r)** -> C  (commit to value v with randomness r)
//! 2. **Open(C, v, r)** -> bool  (verify C was a commitment to v)
//!
//! Properties:
//! - **Hiding**: C reveals nothing about v before opening
//! - **Binding**: Cannot open C to a different value v'
//!
//! ## Pedersen Commitment
//!
//! In a group with generators g, h where nobody knows log_g(h):
//!
//! ```text
//! Commit(v, r) = g^v * h^r  mod p
//! ```
//!
//! - **Information-theoretically hiding**: For any C, every value v has a valid opening
//!   (just compute r = log_h(C / g^v), which exists)
//! - **Computationally binding**: Finding (v', r') != (v, r) with same commitment requires
//!   computing discrete log
//!
//! ## ElGamal Commitment
//!
//! ```text
//! Commit(v, r) = (g^r, h^r * g^v)  mod p
//! ```
//!
//! A pair (c1, c2). Can be converted to an ElGamal encryption of g^v under public key h.
//!
//! ## Applications
//!
//! - **E-voting**: Commit to your vote, reveal later for tallying
//! - **Coin flipping**: Both parties commit to bits, reveal simultaneously, XOR for result
//! - **Zero-knowledge proofs**: Commit to witness, prove statements about it
//! - **Sealed-bid auctions**: Commit to bid, reveal after all bids are in

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

/// Public parameters for the commitment scheme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentParams {
    /// The prime modulus
    pub p: i64,
    /// Generator g
    pub g: i64,
    /// Generator h (where log_g(h) is unknown)
    pub h: i64,
}

/// A Pedersen commitment: C = g^v * h^r mod p.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PedersenCommitment {
    pub value: i64,
}

/// An ElGamal commitment: (c1, c2) = (g^r, h^r * g^v) mod p.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ElGamalCommitment {
    pub c1: i64,
    pub c2: i64,
}

/// A decommitment (opening) for Pedersen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PedersenDecommitment {
    pub value: i64,
    pub randomness: i64,
}

/// A decommitment (opening) for ElGamal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElGamalDecommitment {
    pub value: i64,
    pub randomness: i64,
}

/// Exercise 1: Generate commitment parameters.
///
/// Choose a prime p, a generator g of the multiplicative group mod p,
/// and a second generator h where nobody knows the discrete log of h base g.
///
/// For simplicity, use:
/// - p = a safe prime (e.g., 2027)
/// - g = a small generator (e.g., 2)
/// - h = g^s mod p for a random secret s (in practice, s is discarded)
pub fn generate_params() -> CommitmentParams {
    todo!("Generate public parameters for commitment schemes")
}

/// Exercise 2: Create a Pedersen commitment.
///
/// C = g^v * h^r mod p
///
/// Use fast modular exponentiation.
pub fn pedersen_commit(
    value: i64,
    randomness: i64,
    params: &CommitmentParams,
) -> PedersenCommitment {
    todo!("Compute Pedersen commitment C = g^v * h^r mod p")
}

/// Exercise 3: Verify a Pedersen commitment opening.
///
/// Given C, value v, and randomness r, check that:
/// C == g^v * h^r mod p
pub fn pedersen_verify(
    commitment: &PedersenCommitment,
    decommitment: &PedersenDecommitment,
    params: &CommitmentParams,
) -> bool {
    todo!("Verify Pedersen commitment opening")
}

/// Exercise 4: Homomorphic addition of Pedersen commitments.
///
/// Commit(v1, r1) * Commit(v2, r2) = Commit(v1+v2, r1+r2)
/// C1 * C2 mod p = g^(v1+v2) * h^(r1+r2) mod p
///
/// This allows adding committed values without opening them.
pub fn pedersen_add(
    c1: &PedersenCommitment,
    c2: &PedersenCommitment,
    params: &CommitmentParams,
) -> PedersenCommitment {
    todo!("Homomorphically add two Pedersen commitments")
}

/// Exercise 5: Create an ElGamal commitment.
///
/// (c1, c2) = (g^r mod p, h^r * g^v mod p)
pub fn elgamal_commit(
    value: i64,
    randomness: i64,
    params: &CommitmentParams,
) -> ElGamalCommitment {
    todo!("Compute ElGamal commitment")
}

/// Exercise 6: Verify an ElGamal commitment opening.
///
/// Check that (c1, c2) == (g^r, h^r * g^v) mod p
pub fn elgamal_verify(
    commitment: &ElGamalCommitment,
    decommitment: &ElGamalDecommitment,
    params: &CommitmentParams,
) -> bool {
    todo!("Verify ElGamal commitment opening")
}

/// Exercise 7: Coin flipping protocol using commitments.
///
/// Two-party coin flip:
/// 1. Alice commits to bit_a, sends commitment to Bob
/// 2. Bob commits to bit_b, sends commitment to Alice
/// 3. Both reveal their commitments
/// 4. result = bit_a XOR bit_b
///
/// Simulate the protocol and return the result.
pub fn coin_flip(
    alice_bit: u8,
    alice_randomness: i64,
    bob_bit: u8,
    bob_randomness: i64,
    params: &CommitmentParams,
) -> (u8, PedersenCommitment, PedersenCommitment) {
    todo!("Two-party coin flip using Pedersen commitments")
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

    fn test_params() -> CommitmentParams {
        generate_params()
    }

    #[test]
    fn test_pedersen_commit_verify() {
        let params = test_params();
        let value = 42;
        let randomness = 123;
        let commitment = pedersen_commit(value, randomness, &params);
        let decommitment = PedersenDecommitment { value, randomness };
        assert!(pedersen_verify(&commitment, &decommitment, &params));
    }

    #[test]
    fn test_pedersen_binding() {
        let params = test_params();
        let value = 42;
        let randomness = 123;
        let commitment = pedersen_commit(value, randomness, &params);

        // Opening with wrong value should fail
        let wrong = PedersenDecommitment { value: 99, randomness };
        assert!(!pedersen_verify(&commitment, &wrong, &params));
    }

    #[test]
    fn test_pedersen_hiding() {
        let params = test_params();
        // Two different values should produce different commitments
        let c1 = pedersen_commit(10, 100, &params);
        let c2 = pedersen_commit(20, 200, &params);
        assert_ne!(c1.value, c2.value);
    }

    #[test]
    fn test_pedersen_homomorphic_add() {
        let params = test_params();
        let c1 = pedersen_commit(10, 100, &params);
        let c2 = pedersen_commit(20, 200, &params);
        let c_sum = pedersen_add(&c1, &c2, &params);

        // The sum commitment should open to 30 with randomness 300
        let decommitment = PedersenDecommitment {
            value: 30,
            randomness: 300,
        };
        assert!(pedersen_verify(&c_sum, &decommitment, &params));
    }

    #[test]
    fn test_elgamal_commit_verify() {
        let params = test_params();
        let value = 42;
        let randomness = 55;
        let commitment = elgamal_commit(value, randomness, &params);
        let decommitment = ElGamalDecommitment { value, randomness };
        assert!(elgamal_verify(&commitment, &decommitment, &params));
    }

    #[test]
    fn test_elgamal_binding() {
        let params = test_params();
        let commitment = elgamal_commit(42, 55, &params);
        let wrong = ElGamalDecommitment { value: 99, randomness: 55 };
        assert!(!elgamal_verify(&commitment, &wrong, &params));
    }

    #[test]
    fn test_coin_flip_fair() {
        let params = test_params();
        let (result, c_a, c_b) = coin_flip(1, 100, 0, 200, &params);
        // 1 XOR 0 = 1
        assert_eq!(result, 1);
    }

    #[test]
    fn test_coin_flip_same_bits() {
        let params = test_params();
        let (result, _, _) = coin_flip(1, 100, 1, 200, &params);
        // 1 XOR 1 = 0
        assert_eq!(result, 0);
    }

    #[test]
    fn test_mod_pow() {
        assert_eq!(mod_pow(2, 10, 1000), 24);
        assert_eq!(mod_pow(3, 0, 7), 1);
        assert_eq!(mod_pow(5, 3, 13), 125 % 13);
    }
}
