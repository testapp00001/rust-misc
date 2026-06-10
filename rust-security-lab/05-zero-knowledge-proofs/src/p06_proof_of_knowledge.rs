//! # Lesson 06: Proof of Knowledge
//!
//! ## Proof of Knowledge vs Proof of Statement
//!
//! There's a subtle but important difference:
//!
//! - **Proof of a statement**: "There exists an x such that H(x) = y" (proves the statement is true)
//! - **Proof of knowledge**: "I know an x such that H(x) = y" (proves the prover KNOWS x)
//!
//! Why does this matter? A proof of a statement might be constructed without actually knowing
//! the witness. A proof of knowledge guarantees the prover could extract the witness.
//!
//! ## Knowledge Extraction
//!
//! A proof of knowledge has an extractor: given the ability to rewind the prover to the same
//! commitment state and send different challenges, you can extract the witness.
//!
//! This is the "proof of knowledge" requirement: if the prover convinces the verifier,
//! they MUST know the secret (because we could extract it by rewinding).
//!
//! ## ATTACK: Proof Without Knowledge
//!
//! Suppose someone publishes y = g^x mod p. Can you prove "x exists" without knowing x?
//! Yes! You can prove the statement "there exists some discrete log of y" trivially —
//! it's always true for any y in the group. But you can't prove you KNOW x unless you
//! actually know it.
//!
//! ## Applications
//!
//! - **Digital signatures**: Prove you know the signing key
//! - **Authentication**: Prove you know the password
//! - **Coin ownership**: Prove you own the private key for a Bitcoin address

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

/// Proof of Knowledge parameters.
pub struct PokParams {
    pub g: u64,
    pub p: u64,
    pub q: u64,
}

/// Exercise 1: Create default PoK parameters.
///
/// Hints:
/// - Use g=2, p=23, q=11
pub fn default_pok_params() -> PokParams {
    todo!("Create default PoK parameters")
}

/// Exercise 2: Generate a keypair.
///
/// Hints:
/// - Secret: random x in [1, q-1]
/// - Public: y = g^x mod p
pub fn pok_keygen(params: &PokParams) -> (u64, u64) {
    todo!("Generate PoK keypair")
}

/// Exercise 3: Proof of Knowledge — Prover.
///
/// Standard Schnorr-based proof of knowledge:
/// 1. Pick random k, compute t = g^k mod p
/// 2. Challenge e = H(t || y) mod q
/// 3. Response s = (k + x*e) mod q
/// Returns (t, e, s).
///
/// Hints:
/// - Similar to Schnorr/Sigma prove
pub fn pok_prove(secret: u64, params: &PokParams) -> (u64, u64, u64) {
    todo!("Proof of Knowledge: prover")
}

/// Exercise 4: Proof of Knowledge — Verifier.
///
/// Check: g^s mod p == t * y^e mod p
///
/// Hints:
/// - Same verification as Schnorr
pub fn pok_verify(
    public_key: u64,
    proof: (u64, u64, u64),
    params: &PokParams,
) -> bool {
    todo!("Proof of Knowledge: verifier")
}

/// Exercise 5: Knowledge Extractor — demonstrate extraction by rewinding.
///
/// Given two transcripts (t, e1, s1) and (t, e2, s2) with the same commitment t
/// but different challenges, extract the secret:
///
/// s1 = k + x*e1
/// s2 = k + x*e2
/// s1 - s2 = x*(e1 - e2)
/// x = (s1 - s2) / (e1 - e2) mod q
///
/// Hints:
/// - Compute diff_s = (s1 - s2 + q) % q (handle underflow)
/// - Compute diff_e = (e1 - e2 + q) % q
/// - Compute inverse of diff_e mod q using mod_pow(diff_e, q-2, q)
/// - x = (diff_s * inverse) % q
pub fn extract_knowledge(
    e1: u64, s1: u64,
    e2: u64, s2: u64,
    params: &PokParams,
) -> u64 {
    todo!("Extract secret from two transcripts (knowledge extraction)")
}

/// Exercise 6: Prove a statement exists WITHOUT knowledge (simulator).
///
/// A simulator can create a valid-looking proof without knowing the secret.
/// This shows the proof is zero-knowledge (transcripts can be faked).
///
/// 1. Pick random s and e
/// 2. Compute t = g^s * y^(-e) mod p
/// Returns (t, e, s) — looks like a valid proof!
///
/// Hints:
/// - Pick random s in [1, q-1]
/// - Pick random e in [1, q-1]
/// - t = mod_pow(g, s, p) * mod_pow(y, q - e, p) % p
pub fn simulate_proof(public_key: u64, params: &PokParams) -> (u64, u64, u64) {
    todo!("Simulate a proof without knowing the secret")
}

/// Exercise 7: Demonstrate that simulated proofs verify.
///
/// A simulated proof should pass verification — this is the zero-knowledge property.
/// The verifier can't tell a real proof from a simulated one.
///
/// Hints:
/// - Generate a keypair
/// - Simulate a proof using the public key
/// - Verify it passes
pub fn demonstrate_zk_property(params: &PokParams) -> bool {
    todo!("Demonstrate that simulated proofs are indistinguishable from real ones")
}

/// Exercise 8: Demonstrate knowledge extraction.
///
/// 1. Generate keypair
/// 2. Create two proofs with same commitment but different challenges
/// 3. Extract the secret
/// 4. Verify extracted secret matches original
///
/// Hints:
/// - Use a fixed nonce (not random) so both proofs share the same t
/// - Manually compute two different responses with different challenges
/// - Extract and compare
pub fn demonstrate_extraction(params: &PokParams) -> bool {
    todo!("Demonstrate knowledge extraction from two transcripts")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pok_round_trip() {
        let params = default_pok_params();
        let (sk, pk) = pok_keygen(&params);
        let proof = pok_prove(sk, &params);
        assert!(pok_verify(pk, proof, &params));
    }

    #[test]
    fn test_pok_wrong_key_fails() {
        let params = default_pok_params();
        let (sk, _pk) = pok_keygen(&params);
        let (_sk2, pk2) = pok_keygen(&params);
        let proof = pok_prove(sk, &params);
        // Wrong public key should fail
        assert!(!pok_verify(pk2, proof, &params));
    }

    #[test]
    fn test_knowledge_extraction() {
        let params = default_pok_params();
        let (sk, pk) = pok_keygen(&params);

        // Use a fixed nonce for both proofs
        let k = (rand::random::<u64>() % (params.q - 1)) + 1;
        let t = mod_pow(params.g, k, params.p);

        let e1 = 2u64;
        let e2 = 5u64;
        let s1 = (k + sk * e1) % params.q;
        let s2 = (k + sk * e2) % params.q;

        // Both should verify
        assert!(pok_verify(pk, (t, e1, s1), &params));
        assert!(pok_verify(pk, (t, e2, s2), &params));

        // Extract knowledge
        let extracted = extract_knowledge(e1, s1, e2, s2, &params);
        assert_eq!(extracted, sk);
    }

    #[test]
    fn test_simulation_produces_valid_proof() {
        let params = default_pok_params();
        let (_sk, pk) = pok_keygen(&params);
        let simulated = simulate_proof(pk, &params);
        assert!(pok_verify(pk, simulated, &params));
    }

    #[test]
    fn test_zk_property() {
        let params = default_pok_params();
        assert!(demonstrate_zk_property(&params));
    }

    #[test]
    fn test_extraction_demonstration() {
        let params = default_pok_params();
        assert!(demonstrate_extraction(&params));
    }

    #[test]
    fn test_pok_deterministic_challenge() {
        let params = default_pok_params();
        let (sk, pk) = pok_keygen(&params);
        let proof1 = pok_prove(sk, &params);
        // Same inputs should produce different proofs (random nonce)
        let proof2 = pok_prove(sk, &params);
        // Both should verify (but proofs will differ due to random k)
        assert!(pok_verify(pk, proof1, &params));
        assert!(pok_verify(pk, proof2, &params));
    }
}
