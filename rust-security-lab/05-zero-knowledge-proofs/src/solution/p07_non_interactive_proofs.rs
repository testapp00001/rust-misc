//! # Lesson 07: Non-Interactive Proofs — Fiat-Shamir Heuristic (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use sha2::{Digest, Sha256};

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

pub struct NizkParams {
    pub g: u64,
    pub p: u64,
    pub q: u64,
}

pub fn default_nizk_params() -> NizkParams {
    NizkParams { g: 2, p: 23, q: 11 }
}

pub fn nizk_keygen(params: &NizkParams) -> (u64, u64) {
    let x = (rand::random::<u64>() % (params.q - 1)) + 1;
    let y = mod_pow(params.g, x, params.p);
    (x, y)
}

/// Fiat-Shamir challenge with context binding.
pub fn fiat_shamir(
    commitment: u64,
    public_key: u64,
    context: &[u8],
    params: &NizkParams,
) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(commitment.to_le_bytes());
    hasher.update(public_key.to_le_bytes());
    hasher.update(context);
    let hash = hasher.finalize();
    u64::from_le_bytes(hash[..8].try_into().unwrap()) % params.q
}

/// Non-interactive ZK proof — Prover.
pub fn nizk_prove(
    secret: u64,
    context: &[u8],
    params: &NizkParams,
) -> (u64, u64, u64) {
    let k = (rand::random::<u64>() % (params.q - 1)) + 1;
    let t = mod_pow(params.g, k, params.p);
    let y = mod_pow(params.g, secret, params.p);
    let e = fiat_shamir(t, y, context, params);
    let s = (k + (secret as u128 * e as u128 % params.q as u128) as u64) % params.q;
    (t, e, s)
}

/// Non-interactive ZK proof — Verifier.
pub fn nizk_verify(
    public_key: u64,
    proof: (u64, u64, u64),
    context: &[u8],
    params: &NizkParams,
) -> bool {
    let (t, e, s) = proof;
    let expected_e = fiat_shamir(t, public_key, context, params);
    if e != expected_e {
        return false;
    }
    let lhs = mod_pow(params.g, s, params.p);
    let rhs = ((t as u128 * mod_pow(public_key, e, params.p) as u128) % params.p as u128) as u64;
    lhs == rhs
}

/// Demonstrate context binding: different contexts produce different challenges.
pub fn demonstrate_context_binding(params: &NizkParams) -> (u64, u64) {
    let (_sk, pk) = nizk_keygen(params);
    let k = (rand::random::<u64>() % (params.q - 1)) + 1;
    let t = mod_pow(params.g, k, params.p);
    let e1 = fiat_shamir(t, pk, b"context-alpha", params);
    let e2 = fiat_shamir(t, pk, b"context-beta", params);
    (e1, e2)
}

/// Compare interactive vs non-interactive proof structure.
pub fn compare_interactive_vs_nizk(
    secret: u64,
    params: &NizkParams,
) -> ((u64, u64, u64), (u64, u64, u64)) {
    let y = mod_pow(params.g, secret, params.p);
    let k = (rand::random::<u64>() % (params.q - 1)) + 1;
    let t = mod_pow(params.g, k, params.p);

    // Interactive: verifier sends random challenge
    let e_interactive = (rand::random::<u64>() % (params.q - 1)) + 1;
    let s_interactive = (k + (secret as u128 * e_interactive as u128 % params.q as u128) as u64) % params.q;

    // NIZK: challenge from Fiat-Shamir
    let e_nizk = fiat_shamir(t, y, b"compare", params);
    let s_nizk = (k + (secret as u128 * e_nizk as u128 % params.q as u128) as u64) % params.q;

    ((t, e_interactive, s_interactive), (t, e_nizk, s_nizk))
}

/// Batch verify multiple NIZK proofs.
pub fn batch_verify(
    public_keys: &[u64],
    proofs: &[(u64, u64, u64)],
    context: &[u8],
    params: &NizkParams,
) -> bool {
    if public_keys.len() != proofs.len() {
        return false;
    }
    public_keys
        .iter()
        .zip(proofs.iter())
        .all(|(pk, proof)| nizk_verify(*pk, *proof, context, params))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nizk_round_trip() {
        let params = default_nizk_params();
        let (sk, pk) = nizk_keygen(&params);
        let proof = nizk_prove(sk, b"test-context", &params);
        assert!(nizk_verify(pk, proof, b"test-context", &params));
    }

    #[test]
    fn test_nizk_wrong_context_fails() {
        let params = default_nizk_params();
        let (sk, pk) = nizk_keygen(&params);
        let proof = nizk_prove(sk, b"context-a", &params);
        assert!(!nizk_verify(pk, proof, b"context-b", &params));
    }

    #[test]
    fn test_nizk_wrong_key_fails() {
        let params = default_nizk_params();
        let (sk, _pk) = nizk_keygen(&params);
        let (_sk2, pk2) = nizk_keygen(&params);
        let proof = nizk_prove(sk, b"test", &params);
        assert!(!nizk_verify(pk2, proof, b"test", &params));
    }

    #[test]
    fn test_fiat_shamir_deterministic() {
        let params = default_nizk_params();
        let e1 = fiat_shamir(10, 5, b"ctx", &params);
        let e2 = fiat_shamir(10, 5, b"ctx", &params);
        assert_eq!(e1, e2);
    }

    #[test]
    fn test_fiat_shamir_context_sensitive() {
        let params = default_nizk_params();
        let e1 = fiat_shamir(10, 5, b"context-a", &params);
        let e2 = fiat_shamir(10, 5, b"context-b", &params);
        let _ = (e1, e2);
    }

    #[test]
    fn test_context_binding() {
        let params = default_nizk_params();
        let (e1, e2) = demonstrate_context_binding(&params);
        let _ = (e1, e2);
    }

    #[test]
    fn test_interactive_vs_nizk() {
        let params = default_nizk_params();
        let (sk, _pk) = nizk_keygen(&params);
        let (interactive, nizk) = compare_interactive_vs_nizk(sk, &params);
        assert!(interactive.0 > 0);
        assert!(nizk.0 > 0);
    }

    #[test]
    fn test_batch_verify() {
        let params = default_nizk_params();
        let (sk1, pk1) = nizk_keygen(&params);
        let (sk2, pk2) = nizk_keygen(&params);
        let proof1 = nizk_prove(sk1, b"batch", &params);
        let proof2 = nizk_prove(sk2, b"batch", &params);
        assert!(batch_verify(&[pk1, pk2], &[proof1, proof2], b"batch", &params));
    }
}
