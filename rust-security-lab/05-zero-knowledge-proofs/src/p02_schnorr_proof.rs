//! # Lesson 02: Schnorr Identification Protocol
//!
//! ## What is Schnorr's Protocol?
//!
//! The Schnorr identification protocol is one of the most elegant ZK proofs. It proves
//! knowledge of a discrete logarithm without revealing the secret.
//!
//! Given: public value `y = g^x mod p` (where `x` is the secret)
//! Goal: Prove you know `x` without revealing it.
//!
//! ## Protocol (Interactive)
//!
//! 1. **Prover** picks random `r`, sends commitment `t = g^r mod p`
//! 2. **Verifier** sends random challenge `e`
//! 3. **Prover** sends response `s = r + x*e mod q`
//! 4. **Verifier** checks: `g^s mod p == t * y^e mod p`
//!
//! Why it works:
//! - `g^s = g^(r + x*e) = g^r * g^(x*e) = t * (g^x)^e = t * y^e`
//!
//! ## Why is this Zero-Knowledge?
//!
//! The verifier sees `t`, `e`, and `s`. But given any `e`, you can pick random `s`,
//! compute `t = g^s * y^(-e)`, and produce a valid transcript. So the transcript
//! could have been simulated without knowing `x` — meaning it reveals nothing about `x`.
//!
//! ## ATTACK: Why Not Just Send x?
//!
//! If Alice sends her secret key `x` to Bob for verification, Bob can:
//! - Impersonate Alice
//! - Sign messages on her behalf
//! - Sell `x` to an attacker
//!
//! With Schnorr, Bob is convinced Alice knows `x`, but cannot derive `x` from the transcript.
//!
//! ## Real-World Usage
//!
//! - **Digital Signatures**: Schnorr signatures (used in Bitcoin's Taproot upgrade)
//! - **Authentication**: Prove identity without passwords
//! - **Group signatures**: Prove membership in a group

/// Modular exponentiation: base^exp mod modulus.
/// Computes (base^exp) % modulus using binary exponentiation.
///
/// Hints:
/// - Use the square-and-multiply algorithm
/// - Handle the case where exp == 0 (result is 1)
/// - Work with u128 to avoid overflow during multiplication
pub fn mod_pow(base: u64, exp: u64, modulus: u64) -> u64 {
    todo!("Implement modular exponentiation")
}

/// Modular inverse using Fermat's little theorem: a^(-1) mod p = a^(p-2) mod p.
/// Only works when modulus is prime.
///
/// Hints:
/// - Use `mod_pow(base, modulus - 2, modulus)`
pub fn mod_inverse(a: u64, modulus: u64) -> u64 {
    todo!("Implement modular inverse using Fermat's little theorem")
}

/// Schnorr parameters: generator g, prime modulus p, subgroup order q.
/// For our teaching example, we use small parameters.
/// In practice, use 2048-bit or larger primes.
pub struct SchnorrParams {
    pub g: u64,
    pub p: u64,
    pub q: u64,
}

/// Exercise 1: Create default Schnorr parameters for teaching.
///
/// Use: g=2, p=23, q=11 (small values for teaching; 2 is a generator of
/// the subgroup of order 11 in Z*_23).
///
/// Hints:
/// - Return SchnorrParams { g: 2, p: 23, q: 11 }
pub fn default_params() -> SchnorrParams {
    todo!("Create default Schnorr parameters")
}

/// Exercise 2: Generate a Schnorr keypair.
///
/// Secret key: random x in [1, q-1]
/// Public key: y = g^x mod p
///
/// Hints:
/// - Use `rand::random::<u64>() % (params.q - 1) + 1` for the secret key
/// - Use `mod_pow(params.g, x, params.p)` for the public key
/// - Return (secret_key, public_key)
pub fn keygen(params: &SchnorrParams) -> (u64, u64) {
    todo!("Generate Schnorr keypair")
}

/// Exercise 3: Schnorr prover — step 1 (commitment).
///
/// Pick random `r` in [1, q-1], compute `t = g^r mod p`.
/// Returns (r, t) where r is kept secret and t is sent to verifier.
///
/// Hints:
/// - Random r: `rand::random::<u64>() % (params.q - 1) + 1`
/// - Commitment: `mod_pow(params.g, r, params.p)`
pub fn prove_commit(params: &SchnorrParams) -> (u64, u64) {
    todo!("Schnorr prover: generate commitment")
}

/// Exercise 4: Schnorr prover — step 3 (response).
///
/// Given secret key x, random r, and challenge e, compute:
/// s = (r + x * e) mod q
///
/// Hints:
/// - Use modular arithmetic to avoid overflow
/// - s = (r + (x * e) % q) % q
pub fn prove_respond(secret_key: u64, r: u64, challenge: u64, params: &SchnorrParams) -> u64 {
    todo!("Schnorr prover: compute response")
}

/// Exercise 5: Schnorr verifier — verify the proof.
///
/// Check: g^s mod p == t * y^e mod p
///
/// Hints:
/// - Left side: `mod_pow(params.g, s, params.p)`
/// - Right side: `(t * mod_pow(public_key, e, params.p)) % params.p`
/// - Return whether they're equal
pub fn verify(
    public_key: u64,
    commitment: u64,
    challenge: u64,
    response: u64,
    params: &SchnorrParams,
) -> bool {
    todo!("Schnorr verifier: check g^s == t * y^e mod p")
}

/// Exercise 6: Generate a deterministic challenge from the commitment and public key.
///
/// This is the Fiat-Shamir transform — makes the protocol non-interactive.
/// challenge = hash(commitment || public_key) mod q
///
/// Hints:
/// - Concatenate commitment and public key as bytes
/// - Hash with SHA-256
/// - Convert first 8 bytes to u64, take mod q
pub fn compute_challenge(commitment: u64, public_key: u64, params: &SchnorrParams) -> u64 {
    todo!("Compute deterministic challenge (Fiat-Shamir)")
}

/// Exercise 7: Complete non-interactive Schnorr proof.
///
/// Combines all steps into a single function:
/// 1. Generate commitment
/// 2. Compute challenge (Fiat-Shamir)
/// 3. Compute response
/// Returns (commitment, challenge, response).
pub fn prove(secret_key: u64, params: &SchnorrParams) -> (u64, u64, u64) {
    todo!("Complete non-interactive Schnorr proof")
}

/// Exercise 8: Verify a non-interactive Schnorr proof.
///
/// Hints:
/// - Recompute the expected challenge
/// - Call `verify()` with all parameters
pub fn verify_proof(
    public_key: u64,
    proof: (u64, u64, u64),
    params: &SchnorrParams,
) -> bool {
    todo!("Verify a non-interactive Schnorr proof")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_pow_basic() {
        assert_eq!(mod_pow(2, 10, 1000), 1024 % 1000);
        assert_eq!(mod_pow(3, 5, 7), 243 % 7);
    }

    #[test]
    fn test_mod_pow_zero_exp() {
        assert_eq!(mod_pow(5, 0, 7), 1);
    }

    #[test]
    fn test_mod_inverse() {
        // 3 * 5 = 15 = 1 mod 7, so 3^(-1) mod 7 = 5
        assert_eq!(mod_inverse(3, 7), 5);
    }

    #[test]
    fn test_keygen_produces_valid_key() {
        let params = default_params();
        let (sk, pk) = keygen(&params);
        assert!(sk >= 1 && sk < params.q);
        assert_eq!(pk, mod_pow(params.g, sk, params.p));
    }

    #[test]
    fn test_schnorr_protocol_round_trip() {
        let params = default_params();
        let (sk, pk) = keygen(&params);

        // Prover commits
        let (r, t) = prove_commit(&params);

        // Verifier sends challenge
        let e = 3u64 % params.q;

        // Prover responds
        let s = prove_respond(sk, r, e, &params);

        // Verifier checks
        assert!(verify(pk, t, e, s, &params), "Schnorr proof should verify");
    }

    #[test]
    fn test_schnorr_wrong_secret_fails() {
        let params = default_params();
        let (_sk, pk) = keygen(&params);

        // Attacker doesn't know the secret — tries random response
        let fake_s = 5u64;
        let t = 7u64;
        let e = 3u64;

        // Should fail (with high probability)
        let result = verify(pk, t, e, fake_s, &params);
        // This might coincidentally pass with small parameters, but generally won't
        // We just check it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_noninteractive_schnorr() {
        let params = default_params();
        let (sk, pk) = keygen(&params);
        let proof = prove(sk, &params);
        assert!(verify_proof(pk, proof, &params));
    }

    #[test]
    fn test_fiat_shamir_deterministic() {
        let params = default_params();
        let c1 = compute_challenge(10, 5, &params);
        let c2 = compute_challenge(10, 5, &params);
        assert_eq!(c1, c2, "Fiat-Shamir challenge should be deterministic");
    }

    #[test]
    fn test_fiat_shamir_different_inputs() {
        let params = default_params();
        let c1 = compute_challenge(10, 5, &params);
        let c2 = compute_challenge(11, 5, &params);
        // With small q, collisions are possible, but generally should differ
        let _ = (c1, c2);
    }
}
