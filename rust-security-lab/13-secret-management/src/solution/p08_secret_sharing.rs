//! # Lesson 08: Secret Sharing (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use rand::Rng;

#[derive(Debug, Clone, PartialEq)]
pub struct Share {
    pub x: u8,
    pub y: u64,
}

pub const PRIME: i64 = 251;

pub fn mod_pow(mut base: i64, mut exp: i64, modulus: i64) -> i64 {
    if modulus == 1 {
        return 0;
    }
    let mut result = 1i64;
    base %= modulus;
    while exp > 0 {
        if exp % 2 == 1 {
            result = (result * base) % modulus;
        }
        exp >>= 1;
        base = (base * base) % modulus;
    }
    result
}

pub fn mod_inverse(a: i64, prime: i64) -> Option<i64> {
    let a_mod = ((a % prime) + prime) % prime;
    if a_mod == 0 {
        return None;
    }
    Some(mod_pow(a_mod, prime - 2, prime))
}

pub fn split_secret(secret: u64, k: usize, n: usize) -> Vec<Share> {
    let mut rng = rand::thread_rng();
    let secret_mod = (secret % PRIME as u64) as i64;

    // Generate k-1 random coefficients
    let mut coefficients = vec![secret_mod]; // a0 = secret
    for _ in 1..k {
        let coeff = (rng.gen_range(0..PRIME as u64)) as i64;
        coefficients.push(coeff);
    }

    // Evaluate polynomial at x = 1, 2, ..., n
    let mut shares = Vec::new();
    for x in 1..=n {
        let x_i = x as i64;
        // Evaluate using Horner's method: f(x) = a0 + x*(a1 + x*(a2 + ...))
        let mut y = 0i64;
        for coeff in coefficients.iter().rev() {
            y = (y * x_i + coeff) % PRIME;
        }
        y = ((y % PRIME) + PRIME) % PRIME;
        shares.push(Share {
            x: x as u8,
            y: y as u64,
        });
    }

    shares
}

pub fn reconstruct_secret(shares: &[Share]) -> u64 {
    let mut secret = 0i64;

    for (i, share_i) in shares.iter().enumerate() {
        let x_i = share_i.x as i64;
        let y_i = share_i.y as i64;

        // Compute Lagrange basis polynomial at x=0
        let mut numerator = 1i64;
        let mut denominator = 1i64;

        for (j, share_j) in shares.iter().enumerate() {
            if i == j {
                continue;
            }
            let x_j = share_j.x as i64;
            numerator = (numerator * (-x_j)) % PRIME;
            denominator = (denominator * (x_i - x_j)) % PRIME;
        }

        numerator = ((numerator % PRIME) + PRIME) % PRIME;
        denominator = ((denominator % PRIME) + PRIME) % PRIME;

        let inv_denom = mod_inverse(denominator, PRIME).unwrap_or(0);
        let lagrange = (y_i * numerator % PRIME * inv_denom % PRIME) % PRIME;
        secret = (secret + lagrange) % PRIME;
    }

    ((secret % PRIME) + PRIME) as u64 % PRIME as u64
}

pub fn validate_shares(shares: &[Share], threshold: usize) -> Result<(), String> {
    if shares.len() < threshold {
        return Err(format!(
            "Need at least {} shares, got {}",
            threshold,
            shares.len()
        ));
    }

    // Check for unique x-coordinates
    let mut seen_x = std::collections::HashSet::new();
    for share in shares {
        if !seen_x.insert(share.x) {
            return Err(format!("Duplicate x-coordinate: {}", share.x));
        }
        if share.x == 0 || share.x > 250 {
            return Err(format!("x-coordinate out of valid range: {}", share.x));
        }
    }

    Ok(())
}

pub fn format_share(share: &Share) -> String {
    format!("Share {}: {}", share.x, share.y)
}

pub fn parse_share(s: &str) -> Result<Share, String> {
    let s = s.trim();
    if !s.starts_with("Share ") {
        return Err("Invalid format: must start with 'Share '".to_string());
    }
    let rest = &s[6..]; // Skip "Share "
    let parts: Vec<&str> = rest.split(": ").collect();
    if parts.len() != 2 {
        return Err("Invalid format: expected 'Share X: Y'".to_string());
    }
    let x = parts[0]
        .parse::<u8>()
        .map_err(|e| format!("Invalid x value: {}", e))?;
    let y = parts[1]
        .parse::<u64>()
        .map_err(|e| format!("Invalid y value: {}", e))?;
    Ok(Share { x, y })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_and_reconstruct() {
        let secret: u64 = 42;
        let shares = split_secret(secret, 3, 5);
        assert_eq!(shares.len(), 5);
        let reconstructed = reconstruct_secret(&shares[0..3]);
        assert_eq!(reconstructed, secret);
    }

    #[test]
    fn test_reconstruct_different_subsets() {
        let secret: u64 = 100;
        let shares = split_secret(secret, 3, 5);

        assert_eq!(
            reconstruct_secret(&[shares[0].clone(), shares[1].clone(), shares[2].clone()]),
            secret
        );
        assert_eq!(
            reconstruct_secret(&[shares[0].clone(), shares[3].clone(), shares[4].clone()]),
            secret
        );
        assert_eq!(
            reconstruct_secret(&[shares[2].clone(), shares[3].clone(), shares[4].clone()]),
            secret
        );
    }

    #[test]
    fn test_mod_pow_basic() {
        assert_eq!(mod_pow(2, 10, 1000), 1024 % 1000);
        assert_eq!(mod_pow(3, 5, 100), 243 % 100);
    }

    #[test]
    fn test_mod_inverse() {
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
        let shares = vec![Share { x: 1, y: 10 }, Share { x: 1, y: 20 }];
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
