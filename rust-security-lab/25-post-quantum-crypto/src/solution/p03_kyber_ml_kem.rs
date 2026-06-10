//! # Lesson 03: Kyber / ML-KEM — NIST-Selected KEM (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use sha2::{Digest, Sha256};
use rand::Rng;

/// Simulated ML-KEM public key.
#[derive(Debug, Clone)]
pub struct MlKemPublicKey {
    pub seed: Vec<u8>,
    pub t: Vec<i64>,
    pub n: usize,
    pub q: i64,
}

/// Simulated ML-KEM secret key.
#[derive(Debug, Clone)]
pub struct MlKemSecretKey {
    pub s: Vec<i64>,
    pub pk: MlKemPublicKey,
}

/// Simulated ML-KEM ciphertext.
#[derive(Debug, Clone)]
pub struct MlKemCiphertext {
    pub u: Vec<i64>,
    pub v: i64,
}

/// Simulated shared secret.
#[derive(Debug, Clone, PartialEq)]
pub struct SharedSecret {
    pub bytes: Vec<u8>,
}

/// Deterministic matrix generation from seed (simplified).
///
/// In real Kyber, this uses an XOF (extendable output function) on the seed
/// to generate matrix A. Here we use a simple hash-based approach.
fn derive_matrix_row(seed: &[u8], row: usize, n: usize, q: i64) -> Vec<i64> {
    let mut result = Vec::with_capacity(n);
    for col in 0..n {
        let mut hasher = Sha256::new();
        hasher.update(seed);
        hasher.update((row as u32).to_be_bytes());
        hasher.update((col as u32).to_be_bytes());
        let hash = hasher.finalize();
        let val = u64::from_be_bytes(hash[..8].try_into().unwrap());
        result.push((val % q as u64) as i64);
    }
    result
}

/// Generate an ML-KEM keypair using simplified LWE.
pub fn ml_kem_keygen(n: usize, q: i64) -> (MlKemPublicKey, MlKemSecretKey) {
    let mut rng = rand::thread_rng();

    // Secret vector: small coefficients (0 or 1)
    let s: Vec<i64> = (0..n).map(|_| rng.gen_range(0..=1)).collect();

    // Random seed for public matrix A
    let seed: Vec<u8> = (0..32).map(|_| rng.gen()).collect();

    // Compute t = A*s + e
    let mut t = Vec::with_capacity(n);
    for i in 0..n {
        let a_row = derive_matrix_row(&seed, i, n, q);
        let as_i: i64 = a_row.iter().zip(s.iter()).map(|(a, s)| a * s).sum();
        let e_i: i64 = rng.gen_range(-1..=1); // small error
        t.push(((as_i + e_i) % q + q) % q);
    }

    let pk = MlKemPublicKey { seed, t, n, q };
    let sk = MlKemSecretKey { s, pk: pk.clone() };
    (pk, sk)
}

/// Encapsulate: generate ciphertext and shared secret.
///
/// Uses a simplified Regev-style encryption of a random message.
pub fn ml_kem_encapsulate(pk: &MlKemPublicKey) -> (MlKemCiphertext, SharedSecret) {
    let mut rng = rand::thread_rng();
    let n = pk.n;
    let q = pk.q;

    // Generate random message (32 bytes)
    let message: Vec<u8> = (0..32).map(|_| rng.gen()).collect();

    // Random subset vector r (0 or 1)
    let r: Vec<i64> = (0..n).map(|_| rng.gen_range(0..=1)).collect();

    // Compute u = A^T * r + e1
    let mut u = vec![0i64; n];
    for i in 0..n {
        let a_row = derive_matrix_row(&pk.seed, i, n, q);
        for j in 0..n {
            u[j] = (u[j] + a_row[j] * r[i]) % q;
        }
    }
    // Add small error to u
    for uj in u.iter_mut() {
        let e: i64 = rng.gen_range(-1..=1);
        *uj = ((*uj + e) % q + q) % q;
    }

    // Compute v = t^T * r + e2 + encode(message)
    let mut v: i64 = 0;
    for i in 0..n {
        v = (v + pk.t[i] * r[i]) % q;
    }
    let e2: i64 = rng.gen_range(-1..=1);
    v = ((v + e2) % q + q) % q;

    // Encode first byte of message into v
    // Map message[0] to {0, q/2} — coarse encoding for demo
    let msg_bit = if message[0] & 1 == 1 { q / 2 } else { 0 };
    v = (v + msg_bit) % q;

    // Shared secret = Hash(message)
    let ss_hash = Sha256::digest(&message);

    let ct = MlKemCiphertext { u, v };
    let ss = SharedSecret { bytes: ss_hash.to_vec() };
    (ct, ss)
}

/// Decapsulate: recover shared secret from ciphertext.
pub fn ml_kem_decapsulate(sk: &MlKemSecretKey, ct: &MlKemCiphertext) -> SharedSecret {
    let q = sk.pk.q;

    // Compute noise = v - u . s mod q
    let dot: i64 = ct.u.iter().zip(sk.s.iter()).map(|(ui, si)| ui * si).sum();
    let noise = ((ct.v - dot) % q + q) % q;
    let centered = if noise > q / 2 { noise - q } else { noise };

    // Recover message bit: if |noise| < q/4, bit is 0; else bit is 1
    let msg_bit: u8 = if centered.abs() < q / 4 { 0 } else { 1 };

    // Reconstruct minimal message (same as encapsulate used)
    let mut message = vec![0u8; 32];
    message[0] = msg_bit;

    let ss_hash = Sha256::digest(&message);
    SharedSecret { bytes: ss_hash.to_vec() }
}

/// Verify KEM correctness.
pub fn ml_kem_verify_keypair(pk: &MlKemPublicKey, sk: &MlKemSecretKey) -> bool {
    let (ct, ss_enc) = ml_kem_encapsulate(pk);
    let ss_dec = ml_kem_decapsulate(sk, &ct);
    ss_enc == ss_dec
}

/// Compute ciphertext overhead ratio vs ECDH-P256.
pub fn ciphertext_overhead_vs_ecdh(ml_kem_ct_size: usize) -> f64 {
    ml_kem_ct_size as f64 / 33.0
}

/// Map Kyber parameter set to NIST security level.
pub fn kyber_security_level(params: &str) -> u32 {
    match params {
        "Kyber512" | "ML-KEM-512" => 1,
        "Kyber768" | "ML-KEM-768" => 3,
        "Kyber1024" | "ML-KEM-1024" => 5,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_kem_keygen_dimensions() {
        let (pk, sk) = ml_kem_keygen(4, 17);
        assert_eq!(pk.n, 4);
        assert_eq!(pk.q, 17);
        assert_eq!(sk.s.len(), 4);
        assert_eq!(pk.t.len(), 4);
    }

    #[test]
    fn test_ml_kem_encaps_produces_output() {
        let (pk, _sk) = ml_kem_keygen(4, 97);
        let (ct, ss) = ml_kem_encapsulate(&pk);
        assert_eq!(ct.u.len(), 4);
        assert_eq!(ss.bytes.len(), 32);
    }

    #[test]
    fn test_ml_kem_decaps_matches() {
        let (pk, sk) = ml_kem_keygen(4, 97);
        let (ct, ss_enc) = ml_kem_encapsulate(&pk);
        let ss_dec = ml_kem_decapsulate(&sk, &ct);
        assert_eq!(ss_enc, ss_dec);
    }

    #[test]
    fn test_ml_kem_verify_keypair() {
        let (pk, sk) = ml_kem_keygen(4, 97);
        assert!(ml_kem_verify_keypair(&pk, &sk));
    }

    #[test]
    fn test_ml_kem_different_messages() {
        let (pk, sk) = ml_kem_keygen(4, 97);
        let (ct1, ss1) = ml_kem_encapsulate(&pk);
        let (ct2, ss2) = ml_kem_encapsulate(&pk);
        assert_ne!(ct1.u, ct2.u);
        let dec1 = ml_kem_decapsulate(&sk, &ct1);
        let dec2 = ml_kem_decapsulate(&sk, &ct2);
        assert_eq!(ss1, dec1);
        assert_eq!(ss2, dec2);
    }

    #[test]
    fn test_ciphertext_overhead() {
        let overhead = ciphertext_overhead_vs_ecdh(1088);
        assert!((overhead - 1088.0 / 33.0).abs() < 0.01);
    }

    #[test]
    fn test_kyber_security_levels() {
        assert_eq!(kyber_security_level("Kyber512"), 1);
        assert_eq!(kyber_security_level("ML-KEM-768"), 3);
        assert_eq!(kyber_security_level("Kyber1024"), 5);
        assert_eq!(kyber_security_level("ML-KEM-512"), 1);
        assert_eq!(kyber_security_level("unknown"), 0);
    }
}
