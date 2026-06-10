//! # Lesson 04: Batch Verification
//!
//! ## What is Batch Verification?
//!
//! Standard signature verification checks one signature at a time. Batch verification
//! checks multiple signatures in a single operation that is faster than verifying each
//! individually.
//!
//! Ed25519 supports native batch verification. For n signatures, batch verification
//! is approximately 2-3x faster than n individual verifications.
//!
//! ## How Ed25519 Batch Verification Works
//!
//! Instead of checking: s_i * B == R_i + H(R_i, A_i, M_i) * A_i  for each i
//!
//! Batch verification checks:
//! sum(a_i * s_i) * B == sum(a_i * R_i) + sum(a_i * H(R_i, A_i, M_i) * A_i)
//!
//! where a_i are random coefficients. A single failure causes the batch to fail,
//! but you don't know which signature is bad — so you fall back to individual checks.
//!
//! ## When to Use Batch Verification
//!
//! - Blockchain nodes verifying many transaction signatures
//! - TLS servers verifying client certificate chains
//! - Certificate transparency log verification
//! - Any scenario with many independent signatures to verify
//!
//! ## Attack Scenario: DoS via Signature Flooding
//!
//! An attacker sends many messages with invalid signatures. Without batch verification,
//! each one requires expensive individual verification. With batch verification,
//! the batch fails fast, and you can binary-search for the bad signature.

use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

/// A single entry for batch verification.
#[derive(Debug, Clone)]
pub struct SignatureEntry {
    pub public_key: Vec<u8>,
    pub message: Vec<u8>,
    pub signature: Vec<u8>,
}

/// Exercise 1: Generate multiple keypairs.
///
/// Returns a vector of (signing_key_bytes, verifying_key_bytes) pairs.
///
/// Hints:
/// - Use a loop or iterator
/// - Generate `count` keypairs
pub fn generate_keypairs(count: usize) -> Vec<(Vec<u8>, Vec<u8>)> {
    todo!("Generate multiple Ed25519 keypairs")
}

/// Exercise 2: Create a batch of signed entries.
///
/// Each entry uses a different key to sign a different message.
///
/// Hints:
/// - Generate keypairs
/// - For each keypair, sign the corresponding message
/// - Return a vector of SignatureEntry
pub fn create_signed_batch(messages: &[&[u8]]) -> (Vec<(Vec<u8>, Vec<u8>)>, Vec<SignatureEntry>) {
    todo!("Create a batch of signed entries from multiple keys")
}

/// Exercise 3: Verify all signatures individually.
///
/// Verify each signature one by one and return true only if ALL are valid.
///
/// Hints:
/// - Iterate over entries
/// - Verify each one individually
/// - Return false if ANY fails
pub fn verify_all_individual(entries: &[SignatureEntry]) -> bool {
    todo!("Verify all signatures individually")
}

/// Exercise 4: Verify a batch using ed25519-dalek's batch API.
///
/// Use `ed25519_dalek::verify_batch` for efficient batch verification.
///
/// Hints:
/// - Collect VerifyingKey instances from each entry
/// - Collect Signature instances from each entry
/// - Collect messages as &[u8] slices
/// - Call `ed25519_dalek::verify_batch(&messages, &signatures, &verifying_keys)`
/// - Note: verify_batch takes slices of references
pub fn verify_batch(entries: &[SignatureEntry]) -> bool {
    todo!("Verify signatures using batch verification")
}

/// Exercise 5: Find the invalid signature in a batch using binary search.
///
/// When batch verification fails, we know at least one signature is bad.
/// Use a divide-and-conquer approach to find it efficiently.
///
/// Hints:
/// - Split the batch in half
/// - Verify each half with batch verification
/// - The half that fails contains the bad signature
/// - Recurse until you find the single bad entry
/// - Base case: a batch of 1 — verify it individually
pub fn find_invalid_in_batch(entries: &[SignatureEntry]) -> Option<usize> {
    todo!("Find the invalid signature using divide-and-conquer")
}

/// Exercise 6: Verify a batch with a mix of valid and invalid signatures,
/// returning a list of all invalid indices.
///
/// Hints:
/// - Use find_invalid_in_batch to find one bad signature
/// - Remove it and repeat on the remaining entries
/// - Map back to original indices
pub fn find_all_invalid(entries: &[SignatureEntry]) -> Vec<usize> {
    todo!("Find all invalid signatures in a batch")
}

/// Exercise 7: Benchmark comparison — return (individual_ms, batch_ms) for n signatures.
///
/// Times both individual and batch verification, returning the elapsed
/// milliseconds for each approach.
///
/// Hints:
/// - Use `std::time::Instant::now()` for timing
/// - Run verification multiple times for stable results
/// - This is more of a demonstration than a test
pub fn benchmark_comparison(entries: &[SignatureEntry]) -> (u128, u128) {
    todo!("Benchmark individual vs batch verification")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypairs_count() {
        let keypairs = generate_keypairs(5);
        assert_eq!(keypairs.len(), 5);
        for (priv_key, pub_key) in &keypairs {
            assert_eq!(priv_key.len(), 32);
            assert_eq!(pub_key.len(), 32);
        }
    }

    #[test]
    fn test_create_signed_batch() {
        let messages: Vec<&[u8]> = vec![b"msg1", b"msg2", b"msg3"];
        let (keypairs, entries) = create_signed_batch(&messages);
        assert_eq!(keypairs.len(), 3);
        assert_eq!(entries.len(), 3);
    }

    #[test]
    fn test_verify_all_individual_valid() {
        let messages: Vec<&[u8]> = vec![b"msg1", b"msg2", b"msg3"];
        let (_, entries) = create_signed_batch(&messages);
        assert!(verify_all_individual(&entries));
    }

    #[test]
    fn test_verify_all_individual_tampered() {
        let messages: Vec<&[u8]> = vec![b"msg1", b"msg2", b"msg3"];
        let (_, mut entries) = create_signed_batch(&messages);
        entries[1].message = b"tampered".to_vec();
        assert!(!verify_all_individual(&entries));
    }

    #[test]
    fn test_batch_verify_valid() {
        let messages: Vec<&[u8]> = vec![b"a", b"b", b"c", b"d"];
        let (_, entries) = create_signed_batch(&messages);
        assert!(verify_batch(&entries));
    }

    #[test]
    fn test_batch_verify_invalid() {
        let messages: Vec<&[u8]> = vec![b"a", b"b", b"c"];
        let (_, mut entries) = create_signed_batch(&messages);
        entries[0].signature = vec![0u8; 64];
        assert!(!verify_batch(&entries));
    }

    #[test]
    fn test_find_invalid_in_batch() {
        let messages: Vec<&[u8]> = vec![b"a", b"b", b"c", b"d", b"e"];
        let (_, mut entries) = create_signed_batch(&messages);
        entries[3].message = b"tampered".to_vec();
        let bad_idx = find_invalid_in_batch(&entries);
        assert_eq!(bad_idx, Some(3));
    }

    #[test]
    fn test_find_all_invalid() {
        let messages: Vec<&[u8]> = vec![b"a", b"b", b"c", b"d", b"e"];
        let (_, mut entries) = create_signed_batch(&messages);
        entries[1].message = b"tampered1".to_vec();
        entries[4].message = b"tampered2".to_vec();
        let bad = find_all_invalid(&entries);
        assert_eq!(bad, vec![1, 4]);
    }
}
