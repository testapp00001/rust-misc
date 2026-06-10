//! # Lesson 10: Hash-Based Commitment Schemes (Reference Solution)

use ring::digest;

/// Generate 32 random bytes for a commitment nonce.
///
/// Uses the OS CSPRNG (via ring) to ensure the nonce is unpredictable.
/// This is critical for the hiding property — a guessable nonce would
/// allow an attacker to brute-force the committed value.
pub fn generate_nonce() -> Vec<u8> {
    let rng = ring::rand::SystemRandom::new();
    let mut nonce = vec![0u8; 32];
    rng.fill(&mut nonce).expect("Failed to generate random nonce");
    nonce
}

/// Create a commitment: C = H(value || nonce).
///
/// Returns (commitment_hash, nonce). The nonce must be kept secret
/// until reveal time to preserve the hiding property.
///
/// The commitment is:
/// - **Binding**: Cannot change value after committing (hash collision resistance)
/// - **Hiding**: Commitment reveals nothing about value (random nonce)
pub fn commit(value: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let nonce = generate_nonce();
    let commitment = commit_with_nonce(value, &nonce);
    (commitment, nonce)
}

/// Verify a commitment during reveal phase.
///
/// Recompute H(value || nonce) and compare with the commitment.
/// Uses constant-time comparison to prevent timing attacks.
pub fn reveal(commitment: &[u8], value: &[u8], nonce: &[u8]) -> bool {
    let computed = commit_with_nonce(value, nonce);
    ring::constant_time::verify_slices_are_equal(&computed, commitment).is_ok()
}

/// Create a commitment with a caller-provided nonce.
///
/// H(value || nonce) using SHA-256.
pub fn commit_with_nonce(value: &[u8], nonce: &[u8]) -> Vec<u8> {
    let mut ctx = digest::Context::new(&digest::SHA256);
    ctx.update(value);
    ctx.update(nonce);
    ctx.finish().as_ref().to_vec()
}

/// Demonstrate the binding property of the commitment scheme.
///
/// Returns true only if we can find a different value that produces the
/// same commitment (which should be computationally infeasible).
///
/// We try a few plausible "cheating" values — none should match.
pub fn demonstrate_binding(commitment: &[u8], _original_value: &[u8], nonce: &[u8]) -> bool {
    // Try to cheat with various values
    let cheating_attempts: Vec<&[u8]> = vec![
        b"cheat1",
        b"cheat2",
        b"different_value",
        b"",
        b"a",
    ];

    for attempt in &cheating_attempts {
        if reveal(commitment, attempt, nonce) {
            return true; // Found a collision (should never happen)
        }
    }
    false // Binding holds — no collision found
}

/// Return commitment and nonce, demonstrating the hiding property.
///
/// The commitment hash alone (without the nonce) reveals nothing about the value.
/// An attacker would need to try all possible (value, nonce) pairs to find a match,
/// which is infeasible for a 256-bit hash with a 256-bit random nonce.
pub fn get_commitment_only(value: &[u8]) -> (Vec<u8>, Vec<u8>) {
    commit(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_reveal_roundtrip() {
        let value = b"my_secret_vote";
        let (commitment, nonce) = commit(value);
        assert!(reveal(&commitment, value, &nonce));
    }

    #[test]
    fn test_wrong_value_fails() {
        let value = b"vote_for_alice";
        let (commitment, nonce) = commit(value);
        assert!(!reveal(&commitment, b"vote_for_bob", &nonce));
    }

    #[test]
    fn test_wrong_nonce_fails() {
        let value = b"vote_for_alice";
        let (commitment, nonce) = commit(value);
        let fake_nonce = generate_nonce();
        assert!(!reveal(&commitment, value, &fake_nonce));
    }

    #[test]
    fn test_binding_property() {
        let value = b"locked_in_value";
        let (commitment, nonce) = commit(value);
        let can_cheat = demonstrate_binding(&commitment, value, &nonce);
        assert!(!can_cheat);
    }

    #[test]
    fn test_hiding_property() {
        let value = b"secret_vote";
        let (commitment, _nonce) = get_commitment_only(value);
        assert_eq!(commitment.len(), 32);
    }

    #[test]
    fn test_deterministic_with_nonce() {
        let value = b"test";
        let nonce = b"fixed_nonce_12345678901234567890";
        let c1 = commit_with_nonce(value, nonce);
        let c2 = commit_with_nonce(value, nonce);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_different_nonces_different_commitments() {
        let value = b"same_value";
        let nonce1 = generate_nonce();
        let nonce2 = generate_nonce();
        let c1 = commit_with_nonce(value, &nonce1);
        let c2 = commit_with_nonce(value, &nonce2);
        assert_ne!(c1, c2);
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
