//! # Lesson 05: Threshold Signatures (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use rand::Rng;

/// Modular exponentiation: base^exp mod modulus
fn mod_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    if modulus == 1 {
        return 0;
    }
    let mut result = 1u64;
    base %= modulus;
    while exp > 0 {
        if exp % 2 == 1 {
            result = mul_mod(result, base, modulus);
        }
        exp >>= 1;
        base = mul_mod(base, base, modulus);
    }
    result
}

/// Modular multiplication avoiding overflow: (a * b) mod m
fn mul_mod(a: u64, b: u64, m: u64) -> u64 {
    // For small enough values, direct multiplication is safe
    // For the primes we use, this is fine
    ((a as u128 * b as u128) % m as u128) as u64
}

/// Modular inverse using Fermat's little theorem: a^(p-2) mod p
fn mod_inverse(a: u64, prime: u64) -> u64 {
    mod_pow(a, prime - 2, prime)
}

/// Evaluate a polynomial at a point using Horner's method.
/// polynomial[0] is the constant term (the secret).
fn eval_polynomial(coefficients: &[u64], x: u64, prime: u64) -> u64 {
    let mut result = 0u64;
    for &coeff in coefficients.iter().rev() {
        result = (mul_mod(result, x, prime) + coeff) % prime;
    }
    result
}

/// Split a secret into shares using Shamir's Secret Sharing.
///
/// The polynomial f(x) = secret + a1*x + ... + a(t-1)*x^(t-1) is evaluated
/// at x = 1, 2, ..., n. Each (x, f(x)) is a share.
///
/// SECURITY: The random coefficients are cryptographically generated.
/// The secret is the constant term f(0).
pub fn split_secret(secret: u64, threshold: usize, n_shares: usize, prime: u64) -> Vec<(u64, u64)> {
    let mut rng = rand::thread_rng();

    // Build polynomial: coefficients[0] = secret, rest are random
    let mut coefficients = vec![0u64; threshold];
    coefficients[0] = secret % prime;
    for i in 1..threshold {
        coefficients[i] = rng.gen_range(1..prime);
    }

    // Evaluate at x = 1..n
    (1..=n_shares as u64)
        .map(|x| (x, eval_polynomial(&coefficients, x, prime)))
        .collect()
}

/// Reconstruct the secret from shares using Lagrange interpolation.
///
/// Computes f(0) = sum_i( y_i * prod_{j!=i}( x_j / (x_j - x_i) ) ) mod p
///
/// This is the mathematical foundation of Shamir's scheme: given t points
/// on a degree-(t-1) polynomial, the polynomial is uniquely determined.
pub fn reconstruct_secret(shares: &[(u64, u64)], prime: u64) -> u64 {
    let mut secret = 0u64;

    for (i, &(xi, yi)) in shares.iter().enumerate() {
        // Compute Lagrange basis polynomial at x=0
        let mut numerator = 1u64;
        let mut denominator = 1u64;

        for (j, &(xj, _)) in shares.iter().enumerate() {
            if i == j {
                continue;
            }
            // numerator *= (0 - xj) = -xj mod prime
            numerator = mul_mod(numerator, (prime - xj) % prime, prime);
            // denominator *= (xi - xj) mod prime
            let diff = if xi >= xj {
                (xi - xj) % prime
            } else {
                prime - ((xj - xi) % prime)
            };
            denominator = mul_mod(denominator, diff, prime);
        }

        // lagrange_coeff = numerator / denominator = numerator * denominator^(-1)
        let lagrange_coeff = mul_mod(numerator, mod_inverse(denominator, prime), prime);
        secret = (secret + mul_mod(yi, lagrange_coeff, prime)) % prime;
    }

    secret
}

/// Validate shares before reconstruction.
///
/// Checks:
/// 1. Enough shares to meet the threshold
/// 2. All x-values are distinct
/// 3. All values are in valid range
pub fn validate_shares(shares: &[(u64, u64)], threshold: usize, prime: u64) -> bool {
    if shares.len() < threshold {
        return false;
    }

    // Check distinct x-values
    let mut x_values: Vec<u64> = shares.iter().map(|&(x, _)| x).collect();
    x_values.sort();
    x_values.dedup();
    if x_values.len() != shares.len() {
        return false;
    }

    // Check valid range
    shares.iter().all(|&(x, y)| x > 0 && x < prime && y < prime)
}

/// Demonstrate that t-1 shares don't reveal the secret.
///
/// With only t-1 shares, any secret value is consistent.
/// We construct a different polynomial that passes through the same t-1 points
/// but has a different f(0).
///
/// This proves information-theoretic security: the shares leak zero information.
pub fn demonstrate_insecurity(
    shares: &[(u64, u64)],
    threshold: usize,
    prime: u64,
) -> (u64, u64) {
    let original_secret = reconstruct_secret(&shares[..threshold], prime);

    // Take only threshold-1 shares
    let partial_shares = &shares[..threshold - 1];

    // Construct a new polynomial with a different secret
    // that passes through the same partial shares.
    // We can do this by picking any desired secret and using Lagrange
    // to find the polynomial.
    let desired_secret = (original_secret + 1) % prime;

    // Create a new share at x=threshold that would make the secret = desired_secret
    // For a degree-(t-1) polynomial with t-1 fixed points, the remaining degree of freedom
    // is exactly the secret value. So we just compute what the last share would be.

    // Actually, the simplest demonstration: reconstruct with different subsets
    // of size threshold. With only threshold-1 shares, multiple secrets are possible.
    // We show two possible secrets by choosing two different "last shares".

    // Pick two different x values for the missing share
    let used_x: Vec<u64> = partial_shares.iter().map(|&(x, _)| x).collect();
    let x_new1 = (1..)
        .find(|x| !used_x.contains(x))
        .unwrap();
    let x_new2 = (x_new1 + 1..)
        .find(|x| !used_x.contains(x))
        .unwrap();

    // For each choice of the missing share's x-coordinate, there exists a y-value
    // that produces any desired secret. We demonstrate two different secrets are possible
    // by using two different interpolation results.

    // Instead, let's just show that the reconstruction with threshold-1 shares
    // gives a different result depending on which additional share we pick.
    let mut all_shares_plus1 = partial_shares.to_vec();
    all_shares_plus1.push((x_new1, 0)); // arbitrary y
    let secret1 = reconstruct_secret(&all_shares_plus1, prime);

    let mut all_shares_plus2 = partial_shares.to_vec();
    all_shares_plus2.push((x_new2, 1)); // different arbitrary y
    let secret2 = reconstruct_secret(&all_shares_plus2, prime);

    // Both are "valid" secrets for the partial information
    let _ = desired_secret;
    (original_secret, if secret1 != original_secret { secret1 } else { secret2 })
}

/// Simulate threshold signing.
///
/// Each "signer" gets a subset of shares. They produce a partial result.
/// Any t partial results can reconstruct the signature.
pub fn simulate_threshold_sign(
    shares: &[(u64, u64)],
    threshold: usize,
    prime: u64,
) -> Vec<(u64, u64)> {
    // In this simplified model, each signer just holds a share
    // and can contribute it when asked. The partial "signature" is the share itself.
    shares
        .iter()
        .take(threshold)
        .map(|&(x, y)| (x, y))
        .collect()
}

/// Combine partial signatures into a full threshold signature.
///
/// In this simplified model, combining = Lagrange interpolation.
/// Real threshold ECDSA would involve combining partial signatures
/// through a more complex protocol.
pub fn combine_partial_signatures(
    partials: &[(u64, u64)],
    prime: u64,
) -> u64 {
    reconstruct_secret(partials, prime)
}

/// Threshold-based access control.
///
/// Returns true if enough valid shares are provided to reconstruct the secret,
/// and the reconstructed secret matches the expected value.
pub fn threshold_access_control(
    provided_shares: &[(u64, u64)],
    threshold: usize,
    prime: u64,
    expected_secret: u64,
) -> bool {
    if !validate_shares(provided_shares, threshold, prime) {
        return false;
    }
    let reconstructed = reconstruct_secret(provided_shares, prime);
    reconstructed == expected_secret
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRIME: u64 = 2_u64.pow(61) - 1; // Mersenne prime M61

    #[test]
    fn test_split_and_reconstruct() {
        let secret = 12345u64;
        let shares = split_secret(secret, 3, 5, PRIME);
        assert_eq!(shares.len(), 5);
        let reconstructed = reconstruct_secret(&shares, PRIME);
        assert_eq!(reconstructed, secret);
    }

    #[test]
    fn test_reconstruct_with_threshold() {
        let secret = 99999u64;
        let shares = split_secret(secret, 3, 5, PRIME);
        let partial: Vec<(u64, u64)> = shares[..3].to_vec();
        let reconstructed = reconstruct_secret(&partial, PRIME);
        assert_eq!(reconstructed, secret);
    }

    #[test]
    fn test_reconstruct_with_any_subset() {
        let secret = 42u64;
        let shares = split_secret(secret, 3, 5, PRIME);
        let subsets = vec![vec![0, 1, 2], vec![0, 2, 4], vec![1, 3, 4], vec![0, 1, 4]];
        for indices in subsets {
            let subset: Vec<(u64, u64)> = indices.iter().map(|&i| shares[i]).collect();
            let result = reconstruct_secret(&subset, PRIME);
            assert_eq!(result, secret, "Failed for subset {:?}", indices);
        }
    }

    #[test]
    fn test_validate_shares_valid() {
        let shares = split_secret(42, 3, 5, PRIME);
        assert!(validate_shares(&shares, 3, PRIME));
    }

    #[test]
    fn test_validate_shares_insufficient() {
        let shares = split_secret(42, 3, 5, PRIME);
        assert!(!validate_shares(&shares[..2], 3, PRIME));
    }

    #[test]
    fn test_demonstrate_insecurity() {
        let secret = 42u64;
        let shares = split_secret(secret, 3, 5, PRIME);
        let (original, possible) = demonstrate_insecurity(&shares, 3, PRIME);
        assert_eq!(original, secret);
        assert_ne!(original, possible, "Different secrets should be possible");
    }

    #[test]
    fn test_threshold_access_control() {
        let secret = 7777u64;
        let shares = split_secret(secret, 3, 5, PRIME);
        assert!(threshold_access_control(&shares[..3], 3, PRIME, secret));
        assert!(!threshold_access_control(&shares[..2], 3, PRIME, secret));
    }
}
