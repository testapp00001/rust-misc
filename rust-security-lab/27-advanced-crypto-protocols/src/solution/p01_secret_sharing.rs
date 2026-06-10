//! # Lesson 01: Shamir's Secret Sharing (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Share {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretSharingParams {
    pub prime: i64,
    pub threshold: usize,
    pub total_shares: usize,
}

impl SecretSharingParams {
    pub fn validate(&self) -> Result<(), String> {
        if self.threshold > self.total_shares {
            return Err(format!(
                "Threshold {} exceeds total shares {}",
                self.threshold, self.total_shares
            ));
        }
        if self.threshold == 0 {
            return Err("Threshold must be at least 1".to_string());
        }
        if self.prime <= 2 {
            return Err("Prime must be > 2".to_string());
        }
        Ok(())
    }
}

pub fn mod_add(a: i64, b: i64, p: i64) -> i64 {
    ((a % p + b % p) % p + p) % p
}

pub fn mod_sub(a: i64, b: i64, p: i64) -> i64 {
    ((a % p - b % p) % p + p) % p
}

pub fn mod_mul(a: i64, b: i64, p: i64) -> i64 {
    (((a % p) * (b % p)) % p + p) % p
}

/// Modular inverse using Fermat's little theorem: a^(p-2) mod p.
pub fn mod_inv(a: i64, p: i64) -> i64 {
    mod_pow(a, p - 2, p)
}

/// Fast modular exponentiation.
fn mod_pow(mut base: i64, mut exp: i64, m: i64) -> i64 {
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

/// Evaluate polynomial at x using Horner's method mod p.
pub fn eval_polynomial(coefficients: &[i64], x: i64, p: i64) -> i64 {
    let mut result = 0i64;
    for &coeff in coefficients.iter().rev() {
        result = mod_add(mod_mul(result, x, p), coeff, p);
    }
    result
}

/// Split a secret into n shares with threshold k.
pub fn split_secret(
    secret: i64,
    params: &SecretSharingParams,
) -> Result<Vec<Share>, String> {
    params.validate()?;

    let mut rng = rand::thread_rng();
    let mut coefficients = vec![secret % params.prime];

    for _ in 0..(params.threshold - 1) {
        let coeff = rng.gen_range(1..params.prime);
        coefficients.push(coeff);
    }

    let shares: Vec<Share> = (1..=params.total_shares as i64)
        .map(|x| Share {
            x,
            y: eval_polynomial(&coefficients, x, params.prime),
        })
        .collect();

    Ok(shares)
}

/// Reconstruct a secret from shares using Lagrange interpolation at x=0.
pub fn reconstruct_secret(
    shares: &[Share],
    params: &SecretSharingParams,
) -> Result<i64, String> {
    if shares.len() < params.threshold {
        return Err(format!(
            "Need at least {} shares, got {}",
            params.threshold,
            shares.len()
        ));
    }

    let k = params.threshold;
    let p = params.prime;
    let mut secret = 0i64;

    for i in 0..k {
        let xi = shares[i].x;
        let yi = shares[i].y;

        // Compute Lagrange basis L_i(0) = prod_{j!=i} (0 - x_j) / (x_i - x_j)
        let mut basis = 1i64;
        for j in 0..k {
            if i != j {
                let xj = shares[j].x;
                let numerator = mod_sub(0, xj, p);       // (0 - x_j)
                let denominator = mod_sub(xi, xj, p);     // (x_i - x_j)
                basis = mod_mul(basis, mod_mul(numerator, mod_inv(denominator, p), p), p);
            }
        }

        secret = mod_add(secret, mod_mul(yi, basis, p), p);
    }

    Ok(secret)
}

/// Validate shares before reconstruction.
pub fn verify_shares(shares: &[Share], params: &SecretSharingParams) -> Result<(), String> {
    if shares.len() < params.threshold {
        return Err(format!(
            "Need at least {} shares, got {}",
            params.threshold,
            shares.len()
        ));
    }

    // Check for distinct x-values
    let mut x_vals: Vec<i64> = shares.iter().map(|s| s.x).collect();
    x_vals.sort();
    x_vals.dedup();
    if x_vals.len() != shares.len() {
        return Err("Duplicate x-values found".to_string());
    }

    // Check x-values are in valid range [1, total_shares]
    for s in shares {
        if s.x < 1 || s.x > params.total_shares as i64 {
            return Err(format!("Share x={} is out of range [1, {}]", s.x, params.total_shares));
        }
    }

    Ok(())
}

/// Homomorphic addition of shared secrets.
pub fn add_shared_secrets(
    shares1: &[Share],
    shares2: &[Share],
    p: i64,
) -> Result<Vec<Share>, String> {
    if shares1.len() != shares2.len() {
        return Err("Share sets must have the same length".to_string());
    }

    let result: Vec<Share> = shares1
        .iter()
        .zip(shares2.iter())
        .map(|(s1, s2)| Share {
            x: s1.x,
            y: mod_add(s1.y, s2.y, p),
        })
        .collect();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_params() -> SecretSharingParams {
        SecretSharingParams {
            prime: 2027,
            threshold: 3,
            total_shares: 5,
        }
    }

    #[test]
    fn test_mod_add_basic() {
        assert_eq!(mod_add(10, 20, 7), 2);
        assert_eq!(mod_add(6, 6, 7), 5);
    }

    #[test]
    fn test_mod_sub_no_negative() {
        assert_eq!(mod_sub(3, 10, 7), 0);
        assert_eq!(mod_sub(10, 3, 7), 0);
    }

    #[test]
    fn test_mod_mul_basic() {
        assert_eq!(mod_mul(5, 3, 7), 1);
        assert_eq!(mod_mul(4, 4, 7), 2);
    }

    #[test]
    fn test_mod_inv_basic() {
        assert_eq!(mod_inv(3, 7), 5);
        assert_eq!(mod_inv(2, 7), 4);
    }

    #[test]
    fn test_eval_polynomial_simple() {
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

        let minimal = &shares[..params.threshold];
        let reconstructed = reconstruct_secret(minimal, &params).unwrap();
        assert_eq!(reconstructed, secret);
    }

    #[test]
    fn test_reconstruct_different_subsets() {
        let params = test_params();
        let secret = 999;
        let shares = split_secret(secret, &params).unwrap();

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

        let too_few = &shares[..params.threshold - 1];
        let result = reconstruct_secret(too_few, &params);
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
