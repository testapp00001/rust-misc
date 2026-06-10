//! # Lesson 10: Bulletproofs — Efficient Range Proofs Without Trusted Setup (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// Modular exponentiation.
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

pub struct BulletproofParams {
    pub g: u64,
    pub h: u64,
    pub p: u64,
    pub q: u64,
}

pub fn default_bp_params() -> BulletproofParams {
    BulletproofParams { g: 2, h: 3, p: 23, q: 11 }
}

/// Compute Pedersen commitment: V = g^v * h^gamma mod p.
pub fn pedersen_commit(value: u64, blinding: u64, params: &BulletproofParams) -> u64 {
    let gv = mod_pow(params.g, value, params.p);
    let hb = mod_pow(params.h, blinding, params.p);
    ((gv as u128 * hb as u128) % params.p as u128) as u64
}

/// Inner product of two vectors mod q.
pub fn inner_product(a: &[u64], b: &[u64], modulus: u64) -> u64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (*x as u128 * *y as u128) % modulus as u128)
        .sum::<u128>() as u64
        % modulus
}

/// Vector addition mod q.
pub fn vec_add(a: &[u64], b: &[u64], modulus: u64) -> Vec<u64> {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x + y) % modulus)
        .collect()
}

/// Scalar-vector multiplication mod q.
pub fn vec_scalar_mul(a: &[u64], scalar: u64, modulus: u64) -> Vec<u64> {
    a.iter()
        .map(|x| (*x as u128 * scalar as u128 % modulus as u128) as u64)
        .collect()
}

/// Vector commitment using a single generator.
pub fn vector_commit(a: &[u64], g: u64, p: u64) -> u64 {
    let sum: u64 = a.iter().sum::<u64>();
    mod_pow(g, sum, p)
}

/// One round of inner product argument: split vectors and compute cross products.
pub fn ipa_one_round(
    a: &[u64],
    b: &[u64],
    modulus: u64,
) -> (u64, u64, Vec<u64>, Vec<u64>) {
    let mid = a.len() / 2;
    let (a_lo, a_hi) = a.split_at(mid);
    let (b_lo, b_hi) = b.split_at(mid);

    // L = <a_lo, b_hi>, R = <a_hi, b_lo>
    let l = inner_product(a_lo, b_hi, modulus);
    let r = inner_product(a_hi, b_lo, modulus);

    // Reduced vectors: element-wise sum of halves
    let reduced_a = a_lo.iter().zip(a_hi.iter()).map(|(x, y)| (x + y) % modulus).collect();
    let reduced_b = b_lo.iter().zip(b_hi.iter()).map(|(x, y)| (x + y) % modulus).collect();

    (l, r, reduced_a, reduced_b)
}

/// Verify one round of IPA.
///
/// The inner product relation: <a, b> = <a_lo, b_lo> + <a_lo, b_hi> + <a_hi, b_lo> + <a_hi, b_hi>
/// = <a_lo, b_lo> + L + R + <a_hi, b_hi>
///
/// After reduction: <a_lo + a_hi, b_lo + b_hi> = <a_lo, b_lo> + <a_lo, b_hi> + <a_hi, b_lo> + <a_hi, b_hi>
/// So: <reduced_a, reduced_b> = <a, b> + L + R (mod q)
pub fn ipa_verify_round(
    original_a: &[u64],
    original_b: &[u64],
    reduced_a: &[u64],
    reduced_b: &[u64],
    l: u64,
    r: u64,
    modulus: u64,
) -> bool {
    let original_ip = inner_product(original_a, original_b, modulus);
    let reduced_ip = inner_product(reduced_a, reduced_b, modulus);
    let expected_reduced = (original_ip + l + r) % modulus;
    reduced_ip == expected_reduced
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pedersen_commit() {
        let params = default_bp_params();
        let v = pedersen_commit(5, 3, &params);
        assert!(v > 0);
    }

    #[test]
    fn test_pedersen_hiding() {
        let params = default_bp_params();
        let c1 = pedersen_commit(5, 3, &params);
        let c2 = pedersen_commit(5, 7, &params);
        assert_ne!(c1, c2);
    }

    #[test]
    fn test_inner_product_basic() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];
        let result = inner_product(&a, &b, 100);
        assert_eq!(result, 32);
    }

    #[test]
    fn test_inner_product_modular() {
        let a = vec![10, 20];
        let b = vec![30, 40];
        let result = inner_product(&a, &b, 11);
        assert_eq!(result, 1100 % 11);
    }

    #[test]
    fn test_vec_add() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];
        let c = vec_add(&a, &b, 100);
        assert_eq!(c, vec![5, 7, 9]);
    }

    #[test]
    fn test_vec_scalar_mul() {
        let a = vec![1, 2, 3];
        let c = vec_scalar_mul(&a, 3, 100);
        assert_eq!(c, vec![3, 6, 9]);
    }

    #[test]
    fn test_ipa_one_round() {
        let a = vec![1, 2, 3, 4];
        let b = vec![5, 6, 7, 8];
        let modulus = 1000;
        let (l, r, _ra, _rb) = ipa_one_round(&a, &b, modulus);
        assert!(l > 0);
        assert!(r > 0);
    }

    #[test]
    fn test_ipa_round_verification() {
        let a = vec![1, 2, 3, 4];
        let b = vec![5, 6, 7, 8];
        let modulus = 1000;
        let (l, r, ra, rb) = ipa_one_round(&a, &b, modulus);
        assert!(ipa_verify_round(&a, &b, &ra, &rb, l, r, modulus));
    }

    #[test]
    fn test_vector_commit() {
        let a = vec![1, 2, 3];
        let c = vector_commit(&a, 2, 23);
        assert!(c > 0);
    }
}
