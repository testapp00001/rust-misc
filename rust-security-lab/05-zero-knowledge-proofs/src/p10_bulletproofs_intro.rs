//! # Lesson 10: Bulletproofs — Efficient Range Proofs Without Trusted Setup
//!
//! ## What are Bulletproofs?
//!
//! Bulletproofs are a type of zero-knowledge proof that provides:
//! - **Range proofs**: Prove a value is in [0, 2^n) without revealing it
//! - **No trusted setup**: Unlike zk-SNARKs, no toxic waste to worry about
//! - **Short proofs**: O(log n) size — a 64-bit range proof is ~700 bytes
//! - **Efficient verification**: Uses inner product arguments
//!
//! ## Key Innovations
//!
//! 1. **Inner Product Argument (IPA)**: Prove that <a, b> = c without revealing a or b
//! 2. **Logarithmic proof size**: Recursive halving reduces proof size from O(n) to O(log n)
//! 3. **No trusted setup**: Uses only the discrete log assumption
//!
//! ## How Bulletproofs Range Proof Works (High Level)
//!
//! 1. **Commit to value**: V = g^v * h^gamma (Pedersen commitment)
//! 2. **Bit decomposition**: v = v_0 + 2*v_1 + ... + 2^(n-1)*v_(n-1)
//! 3. **Prove each bit is 0 or 1**: Using b_i * (1 - b_i) = 0
//! 4. **Inner product argument**: Compress the proof logarithmically
//!
//! ## Comparison with Other Range Proofs
//!
//! | Property | Naive bit proof | Bulletproofs | zk-SNARK range |
//! |----------|----------------|--------------|----------------|
//! | Proof size | O(n) | O(log n) | O(1) |
//! | Trusted setup | No | No | Yes |
//! | Verification | O(n) | O(n) | O(1) |
//! | Proving time | O(n) | O(n) | O(n log n) |
//!
//! ## Real-World Usage
//!
//! - **Monero**: Confidential transactions use Bulletproofs to hide amounts
//! - **Mimblewimble**: Grin and Beam use Bulletproofs for transaction privacy
//! - **Zcash**: Uses zk-SNARKs but Bulletproofs are an alternative
//!
//! ## ATTACK: Why Not Use a Simple Range Proof?
//!
//! Simple bit decomposition gives O(n) proof size. For 64-bit values, that's 64 commitments.
//! Bulletproofs compress this to ~700 bytes regardless of range size using IPA.

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

/// Bulletproofs parameters (simplified for teaching).
pub struct BulletproofParams {
    pub g: u64,
    pub h: u64,
    pub p: u64,
    pub q: u64,
}

/// Exercise 1: Create default Bulletproofs parameters.
///
/// Hints:
/// - Use g=2, h=3, p=23, q=11
pub fn default_bp_params() -> BulletproofParams {
    todo!("Create default Bulletproofs parameters")
}

/// Exercise 2: Compute a Pedersen commitment (used in Bulletproofs).
///
/// V = g^v * h^gamma mod p
///
/// Hints:
/// - Same as p03 commitment
pub fn pedersen_commit(value: u64, blinding: u64, params: &BulletproofParams) -> u64 {
    todo!("Compute Pedersen commitment for Bulletproofs")
}

/// Exercise 3: Inner product of two vectors mod q.
///
/// <a, b> = sum(a_i * b_i) mod q
///
/// Hints:
/// - Zip the two vectors
/// - Multiply corresponding elements
/// - Sum and take mod q
pub fn inner_product(a: &[u64], b: &[u64], modulus: u64) -> u64 {
    todo!("Compute inner product of two vectors")
}

/// Exercise 4: Vector addition mod q.
///
/// c_i = (a_i + b_i) mod q
///
/// Hints:
/// - Zip and add
pub fn vec_add(a: &[u64], b: &[u64], modulus: u64) -> Vec<u64> {
    todo!("Vector addition mod q")
}

/// Exercise 5: Scalar-vector multiplication mod q.
///
/// c_i = (scalar * a_i) mod q
///
/// Hints:
/// - Map each element: (scalar * x) % q
pub fn vec_scalar_mul(a: &[u64], scalar: u64, modulus: u64) -> Vec<u64> {
    todo!("Scalar-vector multiplication mod q")
}

/// Exercise 6: Vector commitment — commit to a vector using generator vector.
///
/// C = product(g_i^a_i) mod p
///
/// For simplicity, use a single generator g raised to the inner product.
///
/// Hints:
/// - Compute inner_product of a with a vector of 1s (or just sum)
/// - Return g^sum mod p
pub fn vector_commit(a: &[u64], g: u64, p: u64) -> u64 {
    todo!("Vector commitment")
}

/// Exercise 7: Simplified inner product argument (IPA) — one round.
///
/// In a real IPA, we recursively halve the vectors. Here we demonstrate one round:
///
/// Given vectors a, b of length n:
/// 1. Split a = (a_lo, a_hi), b = (b_lo, b_hi)
/// 2. Compute L = <a_lo, b_hi>, R = <a_hi, b_lo>
/// 3. The inner product <a, b> = <a_lo, b_lo> + <a_hi, b_hi>
///    But we also send L and R to help the verifier
///
/// Returns (L, R, reduced_a, reduced_b) where reduced vectors are the sums.
///
/// Hints:
/// - Split at midpoint
/// - Compute cross products L and R
/// - Sum the halves for reduced vectors
pub fn ipa_one_round(
    a: &[u64],
    b: &[u64],
    modulus: u64,
) -> (u64, u64, Vec<u64>, Vec<u64>) {
    todo!("One round of inner product argument")
}

/// Exercise 8: Verify one round of IPA.
///
/// Check that <reduced_a, reduced_b> = <a, b> + L + R (mod q).
///
/// Hints:
/// - Compute original inner product
/// - Compute reduced inner product
/// - Check reduced_ip == (original_ip + L + R) mod q
pub fn ipa_verify_round(
    original_a: &[u64],
    original_b: &[u64],
    reduced_a: &[u64],
    reduced_b: &[u64],
    l: u64,
    r: u64,
    modulus: u64,
) -> bool {
    todo!("Verify one round of IPA")
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
        // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        let result = inner_product(&a, &b, 100);
        assert_eq!(result, 32);
    }

    #[test]
    fn test_inner_product_modular() {
        let a = vec![10, 20];
        let b = vec![30, 40];
        // 10*30 + 20*40 = 300 + 800 = 1100
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
