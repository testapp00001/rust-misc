//! # Lesson 10: Hash-Based Commitment Schemes
//!
//! ## What is a Commitment Scheme?
//!
//! A commitment scheme lets you "lock in" a value without revealing it,
//! then later reveal it with proof that you committed to that exact value.
//!
//! It's like writing a value on paper, sealing it in an envelope, and
//! showing the sealed envelope. Later, you open it to prove what you wrote.
//!
//! ## Two Phases
//!
//! 1. **Commit**: Choose a value and randomness, compute `C = H(value || randomness)`
//!    Send C to the verifier. They see C but cannot determine the value.
//!
//! 2. **Reveal**: Provide the value and randomness. Verifier computes
//!    `H(value || randomness)` and checks it matches C.
//!
//! ## Properties
//!
//! - **Binding**: Once committed, you CANNOT change the value.
//!   (Finding different value/randomness with same hash is computationally infeasible.)
//!
//! - **Hiding**: The commitment C reveals NOTHING about the value.
//!   (Without randomness, an attacker could try all possible values.)
//!   The randomness makes the commitment hiding — many (value, randomness) pairs
//!   could produce the same hash.
//!
//! ## Applications
//!
//! - **E-voting**: Voters commit to their vote, then reveal after all votes are cast
//! - **Sealed-bid auctions**: Bidders commit to their bid, then reveal simultaneously
//! - **Coin flipping protocols**: Two parties commit to random bits, then reveal
//! - **Zero-knowledge proofs**: Proving knowledge of a value without revealing it
//! - **Blockchain**: Commit-reveal for fair randomness (e.g., RANDAO in Ethereum)
//!
//! ## Attack Scenario
//!
//! Without randomness, an attacker could brute-force the commitment:
//! `commit("alice") = H("alice")`, `commit("bob") = H("bob")`, etc.
//!
//! With randomness, the attacker would need to find (value, r) such that
//! `H(value || r) = C`, which is infeasible for a cryptographic hash.
//!
//! ## Key Takeaway
//!
//! ALWAYS use random nonce in commitment schemes. Without it, the scheme
//! is not hiding (the commitment can be guessed by trying common values).

use ring::digest;

/// Exercise 1: Generate random bytes for commitment nonce.
///
/// Hints:
/// - Use `ring::rand::SystemRandom::new()` for CSPRNG
/// - Generate 32 bytes of randomness
/// - Use `ring::rand::SecureRandom::fill(&mut buffer)`
pub fn generate_nonce() -> Vec<u8> {
    todo!("Generate 32 random bytes for commitment nonce")
}

/// Exercise 2: Create a commitment to a value.
///
/// Commitment = SHA-256(value || nonce)
///
/// Returns (commitment_hash, nonce) — the nonce must be saved for reveal.
///
/// Hints:
/// - Generate a random nonce
/// - Concatenate value and nonce
/// - Hash the concatenation
/// - Return both the hash and the nonce
pub fn commit(value: &[u8]) -> (Vec<u8>, Vec<u8>) {
    todo!("Create a hash-based commitment")
}

/// Exercise 3: Verify a commitment by revealing.
///
/// Given the original commitment hash, the revealed value, and the nonce,
/// verify that the commitment matches.
///
/// Hints:
/// - Compute SHA-256(value || nonce)
/// - Compare with the commitment hash (constant-time!)
pub fn reveal(commitment: &[u8], value: &[u8], nonce: &[u8]) -> bool {
    todo!("Verify a commitment during reveal phase")
}

/// Exercise 4: Create a commitment with caller-provided nonce.
///
/// This is useful when you need deterministic commitments for testing,
/// or when the nonce is generated externally.
///
/// Hints:
/// - Compute SHA-256(value || nonce)
/// - Return just the commitment hash
pub fn commit_with_nonce(value: &[u8], nonce: &[u8]) -> Vec<u8> {
    todo!("Create commitment with provided nonce")
}

/// Exercise 5: Demonstrate the binding property.
///
/// Try to find a different value that produces the same commitment.
/// This should be computationally infeasible — the function returns false
/// to show that binding holds.
///
/// Hints:
/// - Given a commitment, try a few different values with the same nonce
/// - None should match (demonstrating binding)
pub fn demonstrate_binding(commitment: &[u8], original_value: &[u8], nonce: &[u8]) -> bool {
    todo!("Demonstrate that commitment binding holds")
}

/// Exercise 6: Demonstrate the hiding property.
///
/// Show that the commitment hash alone doesn't reveal the value.
/// This function returns the commitment — an attacker would need to
/// brute-force to find the value (infeasible for random nonce).
///
/// Hints:
/// - Just return the commitment hash
/// - The nonce is kept secret, so the commitment is hiding
pub fn get_commitment_only(value: &[u8]) -> (Vec<u8>, Vec<u8>) {
    todo!("Return commitment and nonce, demonstrating hiding")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_reveal_roundtrip() {
        let value = b"my_secret_vote";
        let (commitment, nonce) = commit(value);
        assert!(reveal(&commitment, value, &nonce), "Valid reveal should succeed");
    }

    #[test]
    fn test_wrong_value_fails() {
        let value = b"vote_for_alice";
        let (commitment, nonce) = commit(value);
        assert!(
            !reveal(&commitment, b"vote_for_bob", &nonce),
            "Wrong value should fail reveal"
        );
    }

    #[test]
    fn test_wrong_nonce_fails() {
        let value = b"vote_for_alice";
        let (commitment, nonce) = commit(value);
        let fake_nonce = generate_nonce();
        assert!(
            !reveal(&commitment, value, &fake_nonce),
            "Wrong nonce should fail reveal"
        );
    }

    #[test]
    fn test_binding_property() {
        let value = b"locked_in_value";
        let (commitment, nonce) = commit(value);
        // Try to "cheat" — find a different value with same commitment
        let can_cheat = demonstrate_binding(&commitment, value, &nonce);
        assert!(!can_cheat, "Binding: cannot change committed value");
    }

    #[test]
    fn test_hiding_property() {
        let value = b"secret_vote";
        let (commitment, _nonce) = get_commitment_only(value);
        // The commitment is 32 bytes — it doesn't reveal the value
        assert_eq!(commitment.len(), 32);
        // An attacker seeing only the commitment cannot determine the value
        // (they'd need to brute-force, which is infeasible with random nonce)
    }

    #[test]
    fn test_deterministic_with_nonce() {
        let value = b"test";
        let nonce = b"fixed_nonce_12345678901234567890";
        let c1 = commit_with_nonce(value, nonce);
        let c2 = commit_with_nonce(value, nonce);
        assert_eq!(c1, c2, "Same value + nonce should produce same commitment");
    }

    #[test]
    fn test_different_nonces_different_commitments() {
        let value = b"same_value";
        let nonce1 = generate_nonce();
        let nonce2 = generate_nonce();
        let c1 = commit_with_nonce(value, &nonce1);
        let c2 = commit_with_nonce(value, &nonce2);
        assert_ne!(c1, c2, "Different nonces should produce different commitments");
    }

    #[test]
    fn test_empty_value() {
        let value = b"";
        let (commitment, nonce) = commit(value);
        assert!(reveal(&commitment, value, &nonce));
    }

    #[test]
    fn test_commit_with_nonce_roundtrip() {
        let value = b"roundtrip";
        let nonce = generate_nonce();
        let commitment = commit_with_nonce(value, &nonce);
        assert!(reveal(&commitment, value, &nonce));
    }
}
