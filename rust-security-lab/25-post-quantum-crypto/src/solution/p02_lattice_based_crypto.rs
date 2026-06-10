//! # Lesson 02: Lattice-Based Cryptography — Learning With Errors (LWE) (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// Generate an LWE sample: b = a * s + e mod q.
///
/// Each element: b[i] = (a[i] * s + e[i]) mod q
pub fn lwe_sample(a: &[i64], s: i64, e: &[i64], q: i64) -> Vec<i64> {
    a.iter()
        .zip(e.iter())
        .map(|(&ai, &ei)| {
            let val = ai * s + ei;
            ((val % q) + q) % q // ensure non-negative
        })
        .collect()
}

/// Verify LWE sample: check that b[i] - a[i]*s is a small error.
///
/// Computes the implicit error and checks it stays within [-max_error, max_error].
pub fn verify_lwe_sample(a: &[i64], b: &[i64], s: i64, q: i64, max_error: i64) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).all(|(&ai, &bi)| {
        let raw_error = bi - ai * s;
        // Center the error around 0 (handle modular wrap-around)
        let centered = ((raw_error % q) + q) % q;
        let centered = if centered > q / 2 { centered - q } else { centered };
        centered.abs() <= max_error
    })
}

/// Encrypt a single bit using simplified LWE encryption.
///
/// Selects a random subset of LWE samples, sums them, and adds the message
/// encoding (0 or q/2).
pub fn lwe_encrypt_bit(
    a_matrix: &[Vec<i64>],
    b: &[i64],
    subset_indices: &[usize],
    message_bit: u8,
    q: i64,
) -> (Vec<i64>, i64) {
    let n_cols = a_matrix[0].len();
    let mut u = vec![0i64; n_cols];
    let mut v = 0i64;

    for &idx in subset_indices {
        for j in 0..n_cols {
            u[j] = (u[j] + a_matrix[idx][j]) % q;
        }
        v = (v + b[idx]) % q;
    }

    // Add message encoding: 0 maps to 0, 1 maps to floor(q/2)
    if message_bit == 1 {
        v = (v + q / 2) % q;
    }

    (u, v)
}

/// Decrypt an LWE ciphertext bit.
///
/// Computes noise = v - u . s mod q, centered around 0.
/// If |noise| < q/4, the bit is 0; otherwise it's 1.
pub fn lwe_decrypt_bit(u: &[i64], v: i64, s: &[i64], q: i64) -> u8 {
    let dot: i64 = u.iter().zip(s.iter()).map(|(ui, si)| ui * si).sum();
    let noise = ((v - dot) % q + q) % q;
    let centered = if noise > q / 2 { noise - q } else { noise };

    if centered.abs() < q / 4 {
        0
    } else {
        1
    }
}

/// Compute the absolute noise level of an LWE ciphertext.
///
/// Noise = |v - u . s| mod q, centered around 0.
pub fn lwe_noise_level(u: &[i64], v: i64, s: &[i64], q: i64) -> i64 {
    let dot: i64 = u.iter().zip(s.iter()).map(|(ui, si)| ui * si).sum();
    let noise = ((v - dot) % q + q) % q;
    let centered = if noise > q / 2 { noise - q } else { noise };
    centered.abs()
}

/// Evaluate LWE parameter security.
///
/// Checks basic security criteria: sufficient dimension, non-trivial modulus,
/// and error bound that allows correct decryption.
pub fn are_parameters_secure(n: usize, q: i64, error_bound: i64) -> bool {
    // Dimension must be large enough
    if n < 64 {
        return false;
    }
    // Modulus must be non-trivial
    if q < 2 {
        return false;
    }
    // Error must be small enough for correct decryption
    // (need |error| < q/4 for bit decryption to work)
    if error_bound >= q / 2 {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lwe_sample_basic() {
        let a = vec![3, 5, 7];
        let s = 2;
        let e = vec![1, 0, -1];
        let q = 11;
        let b = lwe_sample(&a, s, &e, q);
        assert_eq!(b, vec![7, 10, 2]);
    }

    #[test]
    fn test_verify_lwe_sample_valid() {
        let a = vec![3, 5, 7];
        let b = vec![7, 10, 2];
        let s = 2;
        let q = 11;
        assert!(verify_lwe_sample(&a, &b, s, q, 1));
    }

    #[test]
    fn test_verify_lwe_sample_invalid() {
        let a = vec![3, 5, 7];
        let b = vec![7, 10, 99];
        let s = 2;
        let q = 11;
        assert!(!verify_lwe_sample(&a, &b, s, q, 1));
    }

    #[test]
    fn test_lwe_encrypt_decrypt_roundtrip() {
        let a_matrix = vec![
            vec![1, 2],
            vec![3, 4],
            vec![5, 6],
            vec![7, 8],
        ];
        let s = vec![2, 3];
        let b: Vec<i64> = a_matrix.iter().map(|row| {
            let val: i64 = row.iter().zip(s.iter()).map(|(a, s)| a * s).sum();
            (val + 0) % 17
        }).collect();

        let subset = vec![0, 1, 2];
        let (u, v) = lwe_encrypt_bit(&a_matrix, &b, &subset, 0, 17);
        let decrypted = lwe_decrypt_bit(&u, &v, &s, 17);
        assert_eq!(decrypted, 0);
    }

    #[test]
    fn test_lwe_noise_level_zero() {
        let u = vec![1, 2];
        let s = vec![3, 4];
        let v = 11;
        assert_eq!(lwe_noise_level(&u, v, &s, 17), 0);
    }

    #[test]
    fn test_lwe_noise_level_nonzero() {
        let u = vec![1, 2];
        let s = vec![3, 4];
        let v = 12;
        assert_eq!(lwe_noise_level(&u, v, &s, 17), 1);
    }

    #[test]
    fn test_parameters_secure_good() {
        assert!(are_parameters_secure(256, 7681, 3));
    }

    #[test]
    fn test_parameters_secure_too_small() {
        assert!(!are_parameters_secure(32, 7681, 3));
    }

    #[test]
    fn test_parameters_secure_bad_modulus() {
        assert!(!are_parameters_secure(256, 1, 3));
    }

    #[test]
    fn test_parameters_secure_error_too_large() {
        assert!(!are_parameters_secure(256, 100, 60));
    }
}
