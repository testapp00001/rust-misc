//! # Lesson 03: Homomorphic Encryption (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKey {
    pub n: i64,
    pub g: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateKey {
    pub p: i64,
    pub q: i64,
    pub n: i64,
    pub lambda: i64,
    pub mu: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ciphertext {
    pub value: i64,
    pub modulus: i64,
}

pub fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a.abs()
}

pub fn lcm(a: i64, b: i64) -> i64 {
    (a / gcd(a, b)) * b
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

fn extended_gcd(a: i64, b: i64) -> (i64, i64, i64) {
    if a == 0 {
        return (b, 0, 1);
    }
    let (g, x, y) = extended_gcd(b % a, a);
    (g, y - (b / a) * x, x)
}

pub fn mod_inv(a: i64, m: i64) -> i64 {
    let (g, x, _) = extended_gcd(a % m, m);
    if g != 1 {
        panic!("No modular inverse exists");
    }
    ((x % m) + m) % m
}

/// Generate a simplified Paillier-like key pair.
pub fn keygen(p: i64, q: i64) -> (PublicKey, PrivateKey) {
    let n = p * q;
    let g = n + 1;
    let lambda = lcm(p - 1, q - 1);
    let mu = mod_inv(lambda, n);

    (
        PublicKey { n, g },
        PrivateKey { p, q, n, lambda, mu },
    )
}

/// Encrypt: c = g^m mod n^2  (simplified, r=1).
pub fn encrypt(message: i64, pk: &PublicKey) -> Ciphertext {
    let n_sq = pk.n * pk.n;
    let value = mod_pow(pk.g, message, n_sq);
    Ciphertext {
        value,
        modulus: n_sq,
    }
}

/// Decrypt: m = L(c^lambda mod n^2) * mu mod n, where L(x) = (x-1)/n.
pub fn decrypt(ciphertext: &Ciphertext, sk: &PrivateKey) -> i64 {
    let u = mod_pow(ciphertext.value, sk.lambda, ciphertext.modulus);
    let l = (u - 1) / sk.n;
    ((l % sk.n) * sk.mu % sk.n + sk.n) % sk.n
}

/// Homomorphic addition: E(m1) * E(m2) mod n^2 = E(m1 + m2).
pub fn homomorphic_add(c1: &Ciphertext, c2: &Ciphertext) -> Ciphertext {
    assert_eq!(c1.modulus, c2.modulus, "Ciphertexts must use same modulus");
    Ciphertext {
        value: (c1.value * c2.value) % c1.modulus,
        modulus: c1.modulus,
    }
}

/// Homomorphic scalar multiplication: E(m)^k mod n^2 = E(m * k).
pub fn homomorphic_mul_scalar(c: &Ciphertext, scalar: i64) -> Ciphertext {
    Ciphertext {
        value: mod_pow(c.value, scalar, c.modulus),
        modulus: c.modulus,
    }
}

/// Malleability attack: shift encrypted value by delta.
pub fn malleability_attack(
    ciphertext: &Ciphertext,
    delta: i64,
    pk: &PublicKey,
) -> Ciphertext {
    let c_delta = encrypt(delta, pk);
    homomorphic_add(ciphertext, &c_delta)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_keys() -> (PublicKey, PrivateKey) {
        keygen(11, 13)
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
        assert_eq!(decrypted, message);
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
        assert_eq!(decrypted, (m1 + m2) % pk.n);
    }

    #[test]
    fn test_homomorphic_scalar_mul() {
        let (pk, sk) = test_keys();
        let m = 10;
        let k = 3;
        let ct = encrypt(m, &pk);
        let ct_prod = homomorphic_mul_scalar(&ct, k);
        let decrypted = decrypt(&ct_prod, &sk);
        assert_eq!(decrypted, (m * k) % pk.n);
    }

    #[test]
    fn test_malleability_attack() {
        let (pk, sk) = test_keys();
        let m = 20;
        let delta = 5;
        let ct = encrypt(m, &pk);
        let ct_attacked = malleability_attack(&ct, delta, &pk);
        let decrypted = decrypt(&ct_attacked, &sk);
        assert_eq!(decrypted, (m + delta) % pk.n);
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
        assert_eq!(mod_pow(2, 10, 1000), 24);
        assert_eq!(mod_pow(3, 13, 100000), 1594323 % 100000);
    }
}
