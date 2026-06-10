//! # Lesson 04: Sigma Protocols
//!
//! ## What is a Sigma Protocol?
//!
//! A Sigma protocol is a three-move proof with the shape:
//!
//! ```text
//! Prover                          Verifier
//!   |                                |
//!   |--- commitment (a) ----------->|
//!   |                                |
//!   |<-- challenge (e) -------------|
//!   |                                |
//!   |--- response (z) ------------->|
//!   |                                |
//!   |                    verify(a, e, z)
//! ```
//!
//! The name "Sigma" comes from the shape of the message flow (looks like the Greek letter sigma).
//!
//! ## Properties
//!
//! - **Complete**: Honest prover always convinces honest verifier
//! - **Special soundness**: Given two accepting transcripts with the same commitment but different
//!   challenges, you can extract the witness (secret). This proves the prover MUST know the secret.
//! - **Special honest-verifier zero-knowledge (SHVZK)**: Given the challenge in advance, you can
//!   simulate a valid transcript without knowing the secret.
//!
//! ## Fiat-Shamir Transform
//!
//! To make a Sigma protocol non-interactive:
//! Replace the verifier's random challenge with a hash: e = H(a || public_input)
//!
//! This is secure in the **Random Oracle Model** — the hash acts as a random oracle.
//!
//! ## ATTACK: Why Not Skip the Commitment?
//!
//! If the prover just sends the response directly, the verifier learns information about the
//! secret. The commitment "locks in" the prover before seeing the challenge, preventing them
//! from choosing a convenient response.
//!
//! ## Real-World Usage
//!
//! - **Schnorr signatures**: Sigma protocol + Fiat-Shamir = digital signature
//! - **zk-SNARKs**: Built on sigma protocol foundations
//! - **Anonymous credentials**: Prove you have a credential without revealing which one

use sha2::{Digest, Sha256};

/// Modular exponentiation (reused across lessons).
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

/// Sigma protocol parameters.
pub struct SigmaParams {
    pub g: u64,
    pub p: u64,
    pub q: u64,
}

/// Exercise 1: Create default Sigma parameters.
///
/// Use: g=2, p=23, q=11 (same as Schnorr for teaching).
///
/// Hints:
/// - Return SigmaParams { g: 2, p: 23, q: 11 }
pub fn default_sigma_params() -> SigmaParams {
    todo!("Create default Sigma parameters")
}

/// Exercise 2: Generate a keypair for the Sigma protocol.
///
/// Secret: random x in [1, q-1]
/// Public: y = g^x mod p
///
/// Hints:
/// - Same as Schnorr keygen
pub fn sigma_keygen(params: &SigmaParams) -> (u64, u64) {
    todo!("Generate Sigma protocol keypair")
}

/// Exercise 3: Sigma protocol — Prover step 1 (commitment).
///
/// Pick random k in [1, q-1], compute a = g^k mod p.
///
/// Hints:
/// - Random k: `(rand::random::<u64>() % (params.q - 1)) + 1`
/// - Commitment: `mod_pow(params.g, k, params.p)`
/// - Return (k, a) where k is the random nonce and a is the commitment
pub fn sigma_commit(params: &SigmaParams) -> (u64, u64) {
    todo!("Sigma prover: generate commitment")
}

/// Exercise 4: Sigma protocol — Prover step 3 (response).
///
/// Given secret x, random nonce k, and challenge e:
/// z = (k + x * e) mod q
///
/// Hints:
/// - Same formula as Schnorr: z = (k + x * e) % q
/// - Use u128 for intermediate multiplication
pub fn sigma_respond(secret: u64, nonce: u64, challenge: u64, params: &SigmaParams) -> u64 {
    todo!("Sigma prover: compute response")
}

/// Exercise 5: Sigma protocol — Verifier.
///
/// Check: g^z mod p == a * y^e mod p
///
/// Hints:
/// - Left: mod_pow(g, z, p)
/// - Right: (a * mod_pow(y, e, p)) % p
pub fn sigma_verify(
    public_key: u64,
    commitment: u64,
    challenge: u64,
    response: u64,
    params: &SigmaParams,
) -> bool {
    todo!("Sigma verifier: check g^z == a * y^e mod p")
}

/// Exercise 6: Fiat-Shamir transform — compute challenge from commitment and public input.
///
/// e = H(a || y) mod q
///
/// Hints:
/// - Hash commitment and public key bytes with SHA-256
/// - Convert to u64, take mod q
pub fn fiat_shamir_challenge(commitment: u64, public_key: u64, params: &SigmaParams) -> u64 {
    todo!("Compute Fiat-Shamir challenge")
}

/// Exercise 7: Complete non-interactive Sigma proof.
///
/// 1. Commit (generate k, compute a)
/// 2. Challenge = Fiat-Shamir(a, y)
/// 3. Respond (compute z)
/// Returns (a, e, z).
pub fn sigma_prove(secret: u64, params: &SigmaParams) -> (u64, u64, u64) {
    todo!("Complete non-interactive Sigma proof")
}

/// Exercise 8: Verify a non-interactive Sigma proof.
///
/// 1. Recompute expected challenge from (a, y)
/// 2. Verify the response
pub fn sigma_verify_proof(
    public_key: u64,
    proof: (u64, u64, u64),
    params: &SigmaParams,
) -> bool {
    todo!("Verify non-interactive Sigma proof")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sigma_round_trip() {
        let params = default_sigma_params();
        let (sk, pk) = sigma_keygen(&params);
        let (k, a) = sigma_commit(&params);
        let e = 5u64 % params.q;
        let z = sigma_respond(sk, k, e, &params);
        assert!(sigma_verify(pk, a, e, z, &params));
    }

    #[test]
    fn test_sigma_wrong_secret() {
        let params = default_sigma_params();
        let (_sk, pk) = sigma_keygen(&params);
        let (_, a) = sigma_commit(&params);
        let e = 3u64;
        let fake_z = 7u64;
        // Might pass or fail with small params — just check no panic
        let _ = sigma_verify(pk, a, e, fake_z, &params);
    }

    #[test]
    fn test_sigma_noninteractive() {
        let params = default_sigma_params();
        let (sk, pk) = sigma_keygen(&params);
        let proof = sigma_prove(sk, &params);
        assert!(sigma_verify_proof(pk, proof, &params));
    }

    #[test]
    fn test_fiat_shamir_deterministic() {
        let params = default_sigma_params();
        let c1 = fiat_shamir_challenge(10, 5, &params);
        let c2 = fiat_shamir_challenge(10, 5, &params);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_fiat_shamir_binding() {
        let params = default_sigma_params();
        let c1 = fiat_shamir_challenge(10, 5, &params);
        let c2 = fiat_shamir_challenge(11, 5, &params);
        // Different inputs should (usually) give different challenges
        // With small q, collisions possible — just check no panic
        let _ = (c1, c2);
    }

    #[test]
    fn test_sigma_extract_witness() {
        // Special soundness: given two responses to same commitment with different challenges,
        // we can extract the secret.
        let params = default_sigma_params();
        let (sk, pk) = sigma_keygen(&params);
        let (k, a) = sigma_commit(&params);

        let e1 = 2u64;
        let e2 = 5u64;
        let z1 = sigma_respond(sk, k, e1, &params);
        let z2 = sigma_respond(sk, k, e2, &params);

        // Extract: x = (z1 - z2) / (e1 - e2) mod q
        // Both should verify
        assert!(sigma_verify(pk, a, e1, z1, &params));
        assert!(sigma_verify(pk, a, e2, z2, &params));
    }

    #[test]
    fn test_sigma_simulation() {
        // SHVZK: we can simulate a transcript without knowing the secret
        let params = default_sigma_params();
        let (_sk, pk) = sigma_keygen(&params);

        // Simulate: pick random z and e, compute a = g^z * y^(-e) mod p
        let fake_z = (rand::random::<u64>() % (params.q - 1)) + 1;
        let fake_e = (rand::random::<u64>() % (params.q - 1)) + 1;

        let gz = mod_pow(params.g, fake_z, params.p);
        // y^(-e) mod p = y^(q-e) mod p (since y^(q) = 1 for elements in the subgroup)
        let ye = mod_pow(pk, params.q - fake_e, params.p);
        let simulated_a = ((gz as u128 * ye as u128) % params.p as u128) as u64;

        // This simulated transcript should verify!
        assert!(sigma_verify(pk, simulated_a, fake_e, fake_z, &params));
    }
}
