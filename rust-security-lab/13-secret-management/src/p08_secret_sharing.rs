//! # Lesson 08: Secret Sharing (Shamir's Secret Sharing)
//!
//! ## The Problem
//!
//! Some secrets are too important for a single person to hold:
//! - Root CA private keys
//! - Disaster recovery credentials
//! - Master encryption keys
//! - Nuclear launch codes (the classic example)
//!
//! If one person holds the secret, they are a single point of failure
//! (and a single point of compromise).
//!
//! ## Shamir's Secret Sharing (Simplified)
//!
//! Adi Shamir's scheme splits a secret into N shares where:
//! - Any K shares can reconstruct the secret ("K-of-N threshold")
//! - Fewer than K shares reveal nothing about the secret
//!
//! ```text
//! Secret: 42
//! Split into 5 shares, threshold 3:
//!   Share 1: (1, 17)
//!   Share 2: (2, 31)
//!   Share 3: (3, 53)
//!   Share 4: (4, 83)
//!   Share 5: (5, 121)
//!
//! Any 3 shares can reconstruct the secret.
//! 2 or fewer shares reveal nothing.
//! ```
//!
//! ## How It Works (Simplified)
//!
//! 1. Choose a random polynomial of degree K-1 where f(0) = secret
//! 2. Evaluate the polynomial at N different points to get N shares
//! 3. To reconstruct: use Lagrange interpolation to find f(0)
//!
//! ## This Exercise
//!
//! We'll implement a simplified version using GF(256) arithmetic
//! (arithmetic modulo a prime) for educational purposes.
//!
//! ## Attack: Single Point of Compromise
//!
//! Without secret sharing:
//! - One compromised admin means total compromise
//! - No separation of duties
//! - Insider threats are catastrophic
//!
//! ## Defense
//!
//! 1. Split critical secrets using threshold schemes
//! 2. Distribute shares to different people/locations
//! 3. Require multiple shareholders for reconstruction
//! 4. Audit share access independently

/// A share of a secret, consisting of an x-coordinate and y-coordinate.
#[derive(Debug, Clone, PartialEq)]
pub struct Share {
    pub x: u8,
    pub y: u64,
}

/// Exercise 1: Split a secret into shares using a simple polynomial scheme.
///
/// Given a secret (u64), a threshold `k`, and total shares `n`:
/// 1. Generate k-1 random coefficients: a1, a2, ..., a_{k-1}
/// 2. The polynomial is: f(x) = secret + a1*x + a2*x^2 + ... + a_{k-1}*x^{k-1}
/// 3. Evaluate f(x) at x = 1, 2, ..., n to get n shares
///
/// All arithmetic is modulo a prime (use 251, a prime that fits in u8 for x values).
/// The secret and coefficients should be reduced modulo the prime.
///
/// Return a Vec of Shares.
///
/// Hints:
/// - Choose random coefficients in range [0, PRIME)
/// - Evaluate the polynomial using Horner's method for each x
/// - Reduce all arithmetic modulo PRIME (251)
pub fn split_secret(secret: u64, k: usize, n: usize) -> Vec<Share> {
    todo!("Split a secret into n shares with threshold k")
}

/// Exercise 2: Reconstruct a secret from shares using Lagrange interpolation.
///
/// Given at least `k` shares, reconstruct the secret by evaluating
/// the Lagrange interpolation polynomial at x=0.
///
/// The formula for Lagrange interpolation at x=0:
///   secret = sum(y_i * L_i(0)) mod PRIME
///   where L_i(0) = product(-x_j / (x_i - x_j)) for j != i
///
/// Return the reconstructed secret.
///
/// Hints:
/// - For each share i, compute the Lagrange basis polynomial at x=0
/// - L_i(0) = product(-x_j / (x_i - x_j)) for all j != i
/// - All division is modular: a / b = a * mod_inverse(b, PRIME) mod PRIME
/// - Sum up: secret = sum(y_i * L_i(0)) mod PRIME
pub fn reconstruct_secret(shares: &[Share]) -> u64 {
    todo!("Reconstruct secret from shares using Lagrange interpolation")
}

/// Exercise 3: Compute modular inverse using Fermat's little theorem.
///
/// For prime p, the modular inverse of a is: a^(p-2) mod p
///
/// Return `Some(inverse)` if the inverse exists, `None` if a % p == 0.
///
/// Hints:
/// - If a % p == 0, return None (no inverse)
/// - Compute a^(p-2) mod p using modular exponentiation
/// - Use the PRIME constant (251)
pub fn mod_inverse(a: i64, prime: i64) -> Option<i64> {
    todo!("Compute modular inverse using Fermat's little theorem")
}

/// Exercise 4: Perform modular exponentiation.
///
/// Compute (base^exp) mod modulus efficiently.
///
/// Hints:
/// - Use the square-and-multiply algorithm
/// - Start with result = 1
/// - For each bit of exp (from LSB to MSB):
///   - If bit is 1: result = (result * base) % modulus
///   - base = (base * base) % modulus
/// - Return result
pub fn mod_pow(mut base: i64, mut exp: i64, modulus: i64) -> i64 {
    todo!("Compute (base^exp) mod modulus")
}

/// Exercise 5: Verify that a set of shares is valid for reconstruction.
///
/// Check that:
/// - There are at least `threshold` shares
/// - All x-coordinates are unique
/// - All x-coordinates are in valid range (1..=250)
///
/// Return `Ok(())` if valid, `Err(reason)` if not.
pub fn validate_shares(shares: &[Share], threshold: usize) -> Result<(), String> {
    todo!("Validate that shares can be used for reconstruction")
}

/// Exercise 6: Create a human-readable representation of a share.
///
/// Format: "Share {x}: {y}"
///
/// Hints:
/// - Simple string formatting
pub fn format_share(share: &Share) -> String {
    todo!("Format a share as a human-readable string")
}

/// Exercise 7: Parse a share from its string representation.
///
/// Parse "Share {x}: {y}" back into a Share struct.
/// Return `Ok(Share)` if parsing succeeds, `Err(msg)` if it fails.
///
/// Hints:
/// - Check that the string starts with "Share "
/// - Split on ": " to get x and y parts
/// - Parse x and y as numbers
pub fn parse_share(s: &str) -> Result<Share, String> {
    todo!("Parse a share from its string representation")
}

/// The prime modulus for our finite field arithmetic.
pub const PRIME: i64 = 251;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_and_reconstruct() {
        let secret: u64 = 42;
        let shares = split_secret(secret, 3, 5);
        assert_eq!(shares.len(), 5);

        // Reconstruct with any 3 shares
        let reconstructed = reconstruct_secret(&shares[0..3]);
        assert_eq!(reconstructed, secret);
    }

    #[test]
    fn test_reconstruct_different_subsets() {
        let secret: u64 = 100;
        let shares = split_secret(secret, 3, 5);

        // Different subsets of 3 shares should all give the same secret
        assert_eq!(reconstruct_secret(&[shares[0].clone(), shares[1].clone(), shares[2].clone()]), secret);
        assert_eq!(reconstruct_secret(&[shares[0].clone(), shares[3].clone(), shares[4].clone()]), secret);
        assert_eq!(reconstruct_secret(&[shares[2].clone(), shares[3].clone(), shares[4].clone()]), secret);
    }

    #[test]
    fn test_mod_pow_basic() {
        assert_eq!(mod_pow(2, 10, 1000), 1024 % 1000);
        assert_eq!(mod_pow(3, 5, 100), 243 % 100);
    }

    #[test]
    fn test_mod_inverse() {
        // a * a^(-1) should equal 1 mod p
        let a = 7i64;
        let inv = mod_inverse(a, PRIME).unwrap();
        assert_eq!((a * inv) % PRIME, 1);
    }

    #[test]
    fn test_mod_inverse_zero() {
        assert!(mod_inverse(0, PRIME).is_none());
    }

    #[test]
    fn test_validate_shares_valid() {
        let shares = vec![
            Share { x: 1, y: 10 },
            Share { x: 2, y: 20 },
            Share { x: 3, y: 30 },
        ];
        assert!(validate_shares(&shares, 3).is_ok());
    }

    #[test]
    fn test_validate_shares_duplicate_x() {
        let shares = vec![
            Share { x: 1, y: 10 },
            Share { x: 1, y: 20 },
        ];
        assert!(validate_shares(&shares, 2).is_err());
    }

    #[test]
    fn test_format_and_parse_share() {
        let share = Share { x: 3, y: 12345 };
        let formatted = format_share(&share);
        let parsed = parse_share(&formatted).unwrap();
        assert_eq!(parsed, share);
    }
}
