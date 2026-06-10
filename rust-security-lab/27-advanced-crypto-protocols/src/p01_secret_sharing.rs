//! # Lesson 01: Shamir's Secret Sharing -- Split a Secret, Reconstruct with Threshold
//!
//! ## What is Shamir's Secret Sharing?
//!
//! Shamir's Secret Sharing (SSS) splits a secret `s` into `n` shares such that any `k`
//! shares can reconstruct `s`, but fewer than `k` shares reveal nothing about `s`.
//!
//! The scheme works over a finite field (GF(p) for prime p):
//! 1. Choose a random polynomial f(x) = s + a1*x + a2*x^2 + ... + a(k-1)*x^(k-1) mod p
//! 2. Share i is the point (i, f(i)) for i = 1..n
//! 3. Reconstruction uses Lagrange interpolation at x=0 to recover f(0) = s
//!
//! ## Lagrange Interpolation
//!
//! Given k points (x_i, y_i), the polynomial evaluated at x=0 is:
//!
//! ```text
//! s = sum_{i=1}^{k} y_i * prod_{j!=i} (0 - x_j) / (x_i - x_j)   (mod p)
//! ```
//!
//! Each term `L_i(0) = prod_{j!=i} (-x_j) / (x_i - x_j)` is the Lagrange basis polynomial
//! evaluated at 0.
//!
//! ## Security Properties
//!
//! - **k-1 shares reveal nothing**: The polynomial of degree k-1 is underdetermined with
//!   fewer than k points -- every possible secret is equally likely.
//! - **k shares fully determine the secret**: The unique polynomial through k points
//!   gives f(0) = s.
//!
//! ## Attack: Threshold Too Low
//!
//! If k is too small (e.g., k=2), an adversary who compromises 2 shares recovers the secret.
//! The threshold must match the security requirements and threat model.
//!
//! ## Attack: Share Forgery
//!
//! Without integrity checks, a malicious party can submit an invalid share, causing
//! reconstruction to yield a wrong secret. Verifiable Secret Sharing (VSS) adds
//! commitments to detect this.

use rand::Rng;
use serde::{Deserialize, Serialize};

/// A share in Shamir's Secret Sharing scheme.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Share {
    /// The x-coordinate (share index, 1-based)
    pub x: i64,
    /// The y-coordinate (share value)
    pub y: i64,
}

/// Parameters for a secret sharing scheme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretSharingParams {
    /// The prime modulus for finite field arithmetic
    pub prime: i64,
    /// Minimum number of shares required to reconstruct (threshold)
    pub threshold: usize,
    /// Total number of shares to generate
    pub total_shares: usize,
}

impl SecretSharingParams {
    /// Validate that parameters are consistent.
    pub fn validate(&self) -> Result<(), String> {
        todo!("Validate: threshold <= total_shares, prime > 2, etc.")
    }
}

/// Exercise 1: Modular arithmetic helpers.
///
/// Compute `(a + b) mod p`, `(a - b) mod p`, `(a * b) mod p`,
/// and `a^(-1) mod p` (modular inverse).
pub fn mod_add(a: i64, b: i64, p: i64) -> i64 {
    todo!("Compute (a + b) mod p, ensuring non-negative result")
}

pub fn mod_sub(a: i64, b: i64, p: i64) -> i64 {
    todo!("Compute (a - b) mod p, ensuring non-negative result")
}

pub fn mod_mul(a: i64, b: i64, p: i64) -> i64 {
    todo!("Compute (a * b) mod p, ensuring non-negative result")
}

/// Compute a^(-1) mod p using Fermat's little theorem (p must be prime).
/// a^(-1) = a^(p-2) mod p
pub fn mod_inv(a: i64, p: i64) -> i64 {
    todo!("Compute modular inverse using fast exponentiation")
}

/// Exercise 2: Evaluate a polynomial at a point x using Horner's method.
///
/// `coefficients[0]` is the constant term (the secret), `coefficients[i]` is a_i.
/// Evaluate: f(x) = c0 + c1*x + c2*x^2 + ... mod p
pub fn eval_polynomial(coefficients: &[i64], x: i64, p: i64) -> i64 {
    todo!("Evaluate polynomial at x using Horner's method mod p")
}

/// Exercise 3: Generate shares for a secret.
///
/// 1. Create a random polynomial of degree (threshold-1) with secret as constant term
/// 2. Evaluate at x = 1, 2, ..., total_shares
/// 3. Return the list of Share { x, y } pairs
///
/// Hints:
/// - Generate (threshold - 1) random coefficients in range [1, p)
/// - Use `eval_polynomial` for each x
pub fn split_secret(
    secret: i64,
    params: &SecretSharingParams,
) -> Result<Vec<Share>, String> {
    todo!("Split secret into n shares with threshold k")
}

/// Exercise 4: Reconstruct a secret from shares using Lagrange interpolation.
///
/// Given at least `threshold` shares, compute f(0) = secret.
///
/// Formula: s = sum_i ( y_i * prod_{j!=i} (-x_j) / (x_i - x_j) ) mod p
///
/// Hints:
/// - Only use `threshold` shares (ignore extras if more are given)
/// - Compute each Lagrange basis L_i(0) separately
/// - Handle the modular division using `mod_inv`
pub fn reconstruct_secret(
    shares: &[Share],
    params: &SecretSharingParams,
) -> Result<i64, String> {
    todo!("Reconstruct secret from shares using Lagrange interpolation")
}

/// Exercise 5: Verify that a set of shares can be combined.
///
/// Check that:
/// - At least `threshold` shares are provided
/// - All share x-values are distinct
/// - All share x-values are in valid range [1, total_shares]
pub fn verify_shares(shares: &[Share], params: &SecretSharingParams) -> Result<(), String> {
    todo!("Validate shares before reconstruction")
}

/// Exercise 6: Add two secrets in the encrypted domain.
///
/// In Shamir's Secret Sharing, addition is "free" -- just add corresponding shares:
/// new_share_i = share1_i + share2_i  (mod p)
///
/// The reconstructed result will be s1 + s2.
pub fn add_shared_secrets(
    shares1: &[Share],
    shares2: &[Share],
    p: i64,
) -> Result<Vec<Share>, String> {
    todo!("Homomorphic addition of shared secrets")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_params() -> SecretSharingParams {
        SecretSharingParams {
            prime: 2027, // A small prime
            threshold: 3,
            total_shares: 5,
        }
    }

    #[test]
    fn test_mod_add_basic() {
        assert_eq!(mod_add(10, 20, 7), 2); // 30 mod 7 = 2
        assert_eq!(mod_add(6, 6, 7), 5);   // 12 mod 7 = 5
    }

    #[test]
    fn test_mod_sub_no_negative() {
        assert_eq!(mod_sub(3, 10, 7), 4); // (3 - 10) mod 7 = -7 mod 7 = 0 ... actually (3-10)=-7, -7 mod 7 = 0
        // Let me recalculate: (3 - 10) = -7, ((-7 % 7) + 7) % 7 = 0
        assert_eq!(mod_sub(3, 10, 7), 0);
        assert_eq!(mod_sub(10, 3, 7), 0);  // 7 mod 7 = 0
    }

    #[test]
    fn test_mod_mul_basic() {
        assert_eq!(mod_mul(5, 3, 7), 1);  // 15 mod 7 = 1
        assert_eq!(mod_mul(4, 4, 7), 2);  // 16 mod 7 = 2
    }

    #[test]
    fn test_mod_inv_basic() {
        // 3 * 5 = 15 = 1 mod 7, so 3^(-1) = 5 mod 7
        assert_eq!(mod_inv(3, 7), 5);
        // 2 * 4 = 8 = 1 mod 7, so 2^(-1) = 4 mod 7
        assert_eq!(mod_inv(2, 7), 4);
    }

    #[test]
    fn test_eval_polynomial_simple() {
        // f(x) = 3 + 2x + 1x^2, f(2) = 3 + 4 + 4 = 11
        let coeffs = vec![3, 2, 1];
        assert_eq!(eval_polynomial(&coeffs, 2, 2027), 11);
    }

    #[test]
    fn test_split_and_reconstruct() {
        let params = test_params();
        let secret = 42;
        let shares = split_secret(secret, &params).unwrap();
        assert_eq!(shares.len(), 5);

        let reconstructed = reconstruct_secret(&shares, &params).unwrap();
        assert_eq!(reconstructed, secret);
    }

    #[test]
    fn test_reconstruct_with_minimal_shares() {
        let params = test_params();
        let secret = 100;
        let shares = split_secret(secret, &params).unwrap();

        // Use only the minimum number of shares (threshold)
        let minimal = &shares[..params.threshold];
        let reconstructed = reconstruct_secret(minimal, &params).unwrap();
        assert_eq!(reconstructed, secret);
    }

    #[test]
    fn test_reconstruct_different_subsets() {
        let params = test_params();
        let secret = 999;
        let shares = split_secret(secret, &params).unwrap();

        // Different subsets of threshold shares should all recover the same secret
        let subset1 = vec![shares[0].clone(), shares[1].clone(), shares[2].clone()];
        let subset2 = vec![shares[0].clone(), shares[2].clone(), shares[4].clone()];
        let subset3 = vec![shares[1].clone(), shares[3].clone(), shares[4].clone()];

        assert_eq!(reconstruct_secret(&subset1, &params).unwrap(), secret);
        assert_eq!(reconstruct_secret(&subset2, &params).unwrap(), secret);
        assert_eq!(reconstruct_secret(&subset3, &params).unwrap(), secret);
    }

    #[test]
    fn test_reconstruct_insufficient_shares_gives_wrong_result() {
        let params = test_params();
        let secret = 42;
        let shares = split_secret(secret, &params).unwrap();

        // Using fewer than threshold shares should NOT give the correct secret
        // (it might by chance, but generally won't)
        let too_few = &shares[..params.threshold - 1];
        let result = reconstruct_secret(too_few, &params);
        // Should return an error since we don't have enough shares
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_shares_valid() {
        let params = test_params();
        let shares = split_secret(42, &params).unwrap();
        assert!(verify_shares(&shares, &params).is_ok());
    }

    #[test]
    fn test_add_shared_secrets() {
        let params = test_params();
        let s1 = 30;
        let s2 = 70;
        let shares1 = split_secret(s1, &params).unwrap();
        let shares2 = split_secret(s2, &params).unwrap();

        let sum_shares = add_shared_secrets(&shares1, &shares2, params.prime).unwrap();
        let reconstructed = reconstruct_secret(&sum_shares, &params).unwrap();
        assert_eq!(reconstructed, (s1 + s2) % params.prime);
    }

    #[test]
    fn test_params_validate() {
        let valid = SecretSharingParams {
            prime: 2027,
            threshold: 3,
            total_shares: 5,
        };
        assert!(valid.validate().is_ok());

        let invalid = SecretSharingParams {
            prime: 2027,
            threshold: 6,
            total_shares: 5,
        };
        assert!(invalid.validate().is_err());
    }
}
