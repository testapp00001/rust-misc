//! # Lesson 03: Homomorphic Encryption -- Compute on Ciphertexts
//!
//! ## What is Homomorphic Encryption?
//!
//! Homomorphic encryption (HE) allows computation on encrypted data without decrypting it first.
//! The result, when decrypted, equals the result of performing the same computation on plaintexts.
//!
//! ## Types of Homomorphic Encryption
//!
//! | Type | Operations | Example Schemes |
//! |------|-----------|-----------------|
//! | Partially HE | One operation (add OR multiply) | Paillier (add), RSA (mult) |
//! | Somewhat HE | Limited depth of both | BGV, BFV |
//! | Fully HE (FHE) | Unlimited add + multiply | CKKS, TFHE |
//!
//! ## Paillier Cryptosystem (Additively Homomorphic)
//!
//! Key generation: choose primes p, q; n = p*q; lambda = lcm(p-1, q-1)
//! Encryption: c = g^m * r^n mod n^2  (r is random)
//! Decryption: m = L(c^lambda mod n^2) * mu mod n  where L(x) = (x-1)/n
//!
//! Homomorphic property: E(m1) * E(m2) mod n^2 = E(m1 + m2)
//!
//! ## Simplified Model
//!
//! In this exercise we use a simplified model over Z_p to demonstrate the concepts.
//! Real Paillier works over Z_{n^2} with much larger numbers.
//!
//! ## Attack: Ciphertext Malleability
//!
//! Additively homomorphic encryption is malleable -- an adversary can modify ciphertexts
//! to change the underlying plaintext. If E(m) is intercepted, the adversary can compute
//! E(m + delta) without knowing m. This is why HE schemes need additional integrity checks.

use serde::{Deserialize, Serialize};

/// A public key for the simplified homomorphic scheme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKey {
    /// The modulus n = p * q (in Paillier, this is used as n^2 for ciphertext space)
    pub n: i64,
    /// The generator g
    pub g: i64,
}

/// A private key for the simplified homomorphic scheme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateKey {
    /// First prime factor
    pub p: i64,
    /// Second prime factor
    pub q: i64,
    /// The modulus n = p * q
    pub n: i64,
    /// lambda = lcm(p-1, q-1)
    pub lambda: i64,
    /// mu = lambda^(-1) mod n
    pub mu: i64,
}

/// A ciphertext in the simplified homomorphic scheme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ciphertext {
    /// The ciphertext value
    pub value: i64,
    /// The modulus used for encryption
    pub modulus: i64,
}

/// Exercise 1: Generate a simplified Paillier-like key pair.
///
/// Given two primes p and q:
/// 1. Compute n = p * q
/// 2. Compute lambda = lcm(p-1, q-1) = (p-1)*(q-1) / gcd(p-1, q-1)
/// 3. Choose g = n + 1 (simplification)
/// 4. Compute mu = lambda^(-1) mod n
///
/// Return (PublicKey { n, g }, PrivateKey { p, q, n, lambda, mu })
pub fn keygen(p: i64, q: i64) -> (PublicKey, PrivateKey) {
    todo!("Generate Paillier-like key pair")
}

/// Exercise 2: Encrypt a message using the simplified scheme.
///
/// Encryption: c = (g^m * r^n) mod n^2
///
/// For simplicity, use r = 1 (deterministic encryption -- insecure but demonstrates
/// the homomorphic property). In production, r must be random.
///
/// Hints:
/// - Compute n_sq = n * n
/// - c = g^m mod n_sq  (since r=1, r^n = 1)
/// - Use fast modular exponentiation
pub fn encrypt(message: i64, pk: &PublicKey) -> Ciphertext {
    todo!("Encrypt message using simplified Paillier")
}

/// Exercise 3: Decrypt a ciphertext using the simplified scheme.
///
/// Decryption: m = L(c^lambda mod n^2) * mu mod n
/// where L(x) = (x - 1) / n
///
/// Hints:
/// - Compute n_sq = n * n
/// - Compute u = c^lambda mod n_sq
/// - Compute L(u) = (u - 1) / n
/// - m = L(u) * mu mod n
pub fn decrypt(ciphertext: &Ciphertext, sk: &PrivateKey) -> i64 {
    todo!("Decrypt ciphertext using simplified Paillier")
}

/// Exercise 4: Homomorphic addition.
///
/// E(m1) * E(m2) mod n^2 = E(m1 + m2)
///
/// Given two ciphertexts encrypted under the same key, compute a new ciphertext
/// that encrypts the sum of the plaintexts.
pub fn homomorphic_add(c1: &Ciphertext, c2: &Ciphertext) -> Ciphertext {
    todo!("Add two ciphertexts homomorphically")
}

/// Exercise 5: Homomorphic scalar multiplication.
///
/// E(m)^k mod n^2 = E(m * k)
///
/// Multiply a ciphertext by a plaintext scalar.
pub fn homomorphic_mul_scalar(c: &Ciphertext, scalar: i64) -> Ciphertext {
    todo!("Multiply ciphertext by a plaintext scalar")
}

/// Exercise 6: Demonstrate malleability attack.
///
/// Given a ciphertext c = E(m), compute c' = E(m + delta) without decrypting.
///
/// Using the homomorphic property: c' = c * E(delta) mod n^2
/// This shows that anyone can modify encrypted values without the key.
pub fn malleability_attack(
    ciphertext: &Ciphertext,
    delta: i64,
    pk: &PublicKey,
) -> Ciphertext {
    todo!("Demonstrate malleability: shift encrypted value by delta")
}

/// Helper: fast modular exponentiation (base^exp mod m).
pub fn mod_pow(mut base: i64, mut exp: i64, m: i64) -> i64 {
    if m == 1 {
        return 0;
    }
    let mut result = 1i64;
    base %= m;
    while exp > 0 {
        if exp % 2 == 1 {
            result = result * base % m;
        }
        exp >>= 1;
        base = base * base % m;
    }
    result
}

/// Helper: compute GCD.
pub fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a.abs()
}

/// Helper: compute LCM.
pub fn lcm(a: i64, b: i64) -> i64 {
    (a / gcd(a, b)) * b
}

/// Helper: modular inverse via extended Euclidean algorithm.
pub fn mod_inv(a: i64, m: i64) -> i64 {
    let (g, x, _) = extended_gcd(a % m, m);
    if g != 1 {
        panic!("No modular inverse exists");
    }
    ((x % m) + m) % m
}

fn extended_gcd(a: i64, b: i64) -> (i64, i64, i64) {
    if a == 0 {
        return (b, 0, 1);
    }
    let (g, x, y) = extended_gcd(b % a, a);
    (g, y - (b / a) * x, x)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Use small primes for testing (insecure in practice)
    fn test_keys() -> (PublicKey, PrivateKey) {
        keygen(11, 13) // n = 143, n^2 = 20449
    }

    #[test]
    fn test_keygen() {
        let (pk, sk) = test_keys();
        assert_eq!(pk.n, 143);
        assert_eq!(sk.p, 11);
        assert_eq!(sk.q, 13);
        assert_eq!(sk.n, 143);
    }

    #[test]
    fn test_encrypt_decrypt() {
        let (pk, sk) = test_keys();
        let message = 42;
        let ct = encrypt(message, &pk);
        let decrypted = decrypt(&ct, &sk);
        assert_eq!(decrypted, message, "Decrypted value should match original");
    }

    #[test]
    fn test_encrypt_decrypt_various_values() {
        let (pk, sk) = test_keys();
        for msg in 0..50 {
            let ct = encrypt(msg, &pk);
            let dec = decrypt(&ct, &sk);
            assert_eq!(dec, msg, "Failed for message {}", msg);
        }
    }

    #[test]
    fn test_homomorphic_addition() {
        let (pk, sk) = test_keys();
        let m1 = 15;
        let m2 = 27;
        let c1 = encrypt(m1, &pk);
        let c2 = encrypt(m2, &pk);
        let c_sum = homomorphic_add(&c1, &c2);
        let decrypted = decrypt(&c_sum, &sk);
        assert_eq!(decrypted, (m1 + m2) % pk.n, "Homomorphic add should give m1+m2");
    }

    #[test]
    fn test_homomorphic_scalar_mul() {
        let (pk, sk) = test_keys();
        let m = 10;
        let k = 3;
        let ct = encrypt(m, &pk);
        let ct_prod = homomorphic_mul_scalar(&ct, k);
        let decrypted = decrypt(&ct_prod, &sk);
        assert_eq!(decrypted, (m * k) % pk.n, "Homomorphic scalar mul should give m*k");
    }

    #[test]
    fn test_malleability_attack() {
        let (pk, sk) = test_keys();
        let m = 20;
        let delta = 5;
        let ct = encrypt(m, &pk);
        let ct_attacked = malleability_attack(&ct, delta, &pk);
        let decrypted = decrypt(&ct_attacked, &sk);
        assert_eq!(
            decrypted,
            (m + delta) % pk.n,
            "Malleability attack should shift value by delta"
        );
    }

    #[test]
    fn test_homomorphic_sum_of_many() {
        let (pk, sk) = test_keys();
        let values = vec![3, 7, 11, 5];
        let ciphertexts: Vec<Ciphertext> = values.iter().map(|&v| encrypt(v, &pk)).collect();

        let sum_ct = ciphertexts
            .into_iter()
            .reduce(|a, b| homomorphic_add(&a, &b))
            .unwrap();
        let decrypted = decrypt(&sum_ct, &sk);
        assert_eq!(decrypted, values.iter().sum::<i64>() % pk.n);
    }

    #[test]
    fn test_mod_pow() {
        assert_eq!(mod_pow(2, 10, 1000), 24);  // 1024 mod 1000
        assert_eq!(mod_pow(3, 13, 100000), 1594323 % 100000);
    }
}
