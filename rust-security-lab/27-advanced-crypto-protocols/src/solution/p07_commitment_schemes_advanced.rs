//! # Lesson 07: Commitment Schemes Advanced (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentParams {
    pub p: i64,
    pub g: i64,
    pub h: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PedersenCommitment {
    pub value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ElGamalCommitment {
    pub c1: i64,
    pub c2: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PedersenDecommitment {
    pub value: i64,
    pub randomness: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElGamalDecommitment {
    pub value: i64,
    pub randomness: i64,
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

/// Generate public parameters.
pub fn generate_params() -> CommitmentParams {
    let p = 2027i64;
    let g = 2i64;
    // h = g^s mod p for a random secret s (s is discarded)
    let mut rng = rand::thread_rng();
    let s: i64 = rng.gen_range(2..p - 1);
    let h = mod_pow(g, s, p);
    CommitmentParams { p, g, h }
}

/// Pedersen commitment: C = g^v * h^r mod p.
pub fn pedersen_commit(
    value: i64,
    randomness: i64,
    params: &CommitmentParams,
) -> PedersenCommitment {
    let gv = mod_pow(params.g, value, params.p);
    let hr = mod_pow(params.h, randomness, params.p);
    PedersenCommitment {
        value: (gv * hr) % params.p,
    }
}

/// Verify a Pedersen commitment opening.
pub fn pedersen_verify(
    commitment: &PedersenCommitment,
    decommitment: &PedersenDecommitment,
    params: &CommitmentParams,
) -> bool {
    let expected = pedersen_commit(decommitment.value, decommitment.randomness, params);
    commitment.value == expected.value
}

/// Homomorphic addition of Pedersen commitments.
pub fn pedersen_add(
    c1: &PedersenCommitment,
    c2: &PedersenCommitment,
    params: &CommitmentParams,
) -> PedersenCommitment {
    PedersenCommitment {
        value: (c1.value * c2.value) % params.p,
    }
}

/// ElGamal commitment: (c1, c2) = (g^r, h^r * g^v) mod p.
pub fn elgamal_commit(
    value: i64,
    randomness: i64,
    params: &CommitmentParams,
) -> ElGamalCommitment {
    let c1 = mod_pow(params.g, randomness, params.p);
    let hr = mod_pow(params.h, randomness, params.p);
    let gv = mod_pow(params.g, value, params.p);
    ElGamalCommitment {
        c1,
        c2: (hr * gv) % params.p,
    }
}

/// Verify an ElGamal commitment opening.
pub fn elgamal_verify(
    commitment: &ElGamalCommitment,
    decommitment: &ElGamalDecommitment,
    params: &CommitmentParams,
) -> bool {
    let expected = elgamal_commit(decommitment.value, decommitment.randomness, params);
    commitment.c1 == expected.c1 && commitment.c2 == expected.c2
}

/// Coin flip protocol using commitments.
pub fn coin_flip(
    alice_bit: u8,
    alice_randomness: i64,
    bob_bit: u8,
    bob_randomness: i64,
    params: &CommitmentParams,
) -> (u8, PedersenCommitment, PedersenCommitment) {
    let c_a = pedersen_commit(alice_bit as i64, alice_randomness, params);
    let c_b = pedersen_commit(bob_bit as i64, bob_randomness, params);
    let result = alice_bit ^ bob_bit;
    (result, c_a, c_b)
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
        let wrong = PedersenDecommitment { value: 99, randomness };
        assert!(!pedersen_verify(&commitment, &wrong, &params));
    }

    #[test]
    fn test_pedersen_hiding() {
        let params = test_params();
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
        let (result, _c_a, _c_b) = coin_flip(1, 100, 0, 200, &params);
        assert_eq!(result, 1);
    }

    #[test]
    fn test_coin_flip_same_bits() {
        let params = test_params();
        let (result, _, _) = coin_flip(1, 100, 1, 200, &params);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_mod_pow() {
        assert_eq!(mod_pow(2, 10, 1000), 24);
        assert_eq!(mod_pow(3, 0, 7), 1);
        assert_eq!(mod_pow(5, 3, 13), 125 % 13);
    }
}
