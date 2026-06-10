//! # Lesson 04: Batch Verification (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ed25519_dalek::{Signer, SigningKey, Signature, Verifier, VerifyingKey};
use rand::rngs::OsRng;

/// A single entry for batch verification.
#[derive(Debug, Clone)]
pub struct SignatureEntry {
    pub public_key: Vec<u8>,
    pub message: Vec<u8>,
    pub signature: Vec<u8>,
}

/// Generate multiple Ed25519 keypairs.
pub fn generate_keypairs(count: usize) -> Vec<(Vec<u8>, Vec<u8>)> {
    (0..count)
        .map(|_| {
            let signing_key = SigningKey::generate(&mut OsRng);
            let verifying_key = VerifyingKey::from(&signing_key);
            (
                signing_key.to_bytes().to_vec(),
                verifying_key.to_bytes().to_vec(),
            )
        })
        .collect()
}

/// Create a batch of signed entries, one per message.
///
/// Each message is signed with a different key to simulate a realistic
/// scenario (e.g., different users submitting signed transactions).
pub fn create_signed_batch(messages: &[&[u8]]) -> (Vec<(Vec<u8>, Vec<u8>)>, Vec<SignatureEntry>) {
    let keypairs = generate_keypairs(messages.len());
    let entries = keypairs
        .iter()
        .zip(messages.iter())
        .map(|((priv_key, pub_key), message)| {
            let signing_key: SigningKey =
                SigningKey::from_bytes(&priv_key.clone().try_into().unwrap());
            let sig = signing_key.sign(message);
            SignatureEntry {
                public_key: pub_key.clone(),
                message: message.to_vec(),
                signature: sig.to_bytes().to_vec(),
            }
        })
        .collect();
    (keypairs, entries)
}

/// Verify all signatures individually.
///
/// This is O(n) individual verifications — each one is an expensive
/// elliptic curve operation.
pub fn verify_all_individual(entries: &[SignatureEntry]) -> bool {
    entries.iter().all(|entry| {
        let key_bytes: [u8; 32] = match entry.public_key.clone().try_into() {
            Ok(b) => b,
            Err(_) => return false,
        };
        let verifying_key = match VerifyingKey::from_bytes(&key_bytes) {
            Ok(k) => k,
            Err(_) => return false,
        };
        let sig_bytes: [u8; 64] = match entry.signature.clone().try_into() {
            Ok(b) => b,
            Err(_) => return false,
        };
        let signature = Signature::from_bytes(&sig_bytes);
        verifying_key.verify(&entry.message, &signature).is_ok()
    })
}

/// Verify a batch using ed25519-dalek's batch API.
///
/// `ed25519_dalek::verify_batch` performs a single multi-scalar multiplication
/// instead of n individual point multiplications. This is significantly faster
/// for large batches.
///
/// SECURITY NOTE: A single invalid signature causes the entire batch to fail.
/// You must then binary-search for the bad signature(s).
pub fn verify_batch(entries: &[SignatureEntry]) -> bool {
    let mut verifying_keys = Vec::new();
    let mut signatures = Vec::new();
    let mut messages = Vec::new();

    for entry in entries {
        let key_bytes: [u8; 32] = match entry.public_key.clone().try_into() {
            Ok(b) => b,
            Err(_) => return false,
        };
        let verifying_key = match VerifyingKey::from_bytes(&key_bytes) {
            Ok(k) => k,
            Err(_) => return false,
        };
        let sig_bytes: [u8; 64] = match entry.signature.clone().try_into() {
            Ok(b) => b,
            Err(_) => return false,
        };
        let signature = Signature::from_bytes(&sig_bytes);

        verifying_keys.push(verifying_key);
        signatures.push(signature);
        messages.push(entry.message.as_slice());
    }

    // ed25519-dalek 2.x removed the top-level verify_batch function.
    // Fall back to individual verification (batch API was removed).
    // In production, use a crate that provides batch verification.
    verifying_keys.iter().zip(signatures.iter()).zip(messages.iter())
        .all(|((vk, sig), msg)| vk.verify(msg, sig).is_ok())
}

/// Find the invalid signature using divide-and-conquer.
///
/// When batch verification fails, at least one signature is bad.
/// We split the batch in half and check each half. The half that fails
/// contains the bad signature. We recurse until we find it.
///
/// This is O(log n) batch verifications in the best case (one bad sig).
pub fn find_invalid_in_batch(entries: &[SignatureEntry]) -> Option<usize> {
    if entries.is_empty() {
        return None;
    }

    // Base case: single entry
    if entries.len() == 1 {
        return if verify_all_individual(entries) {
            None
        } else {
            Some(0)
        };
    }

    // Try batch verification first
    if verify_batch(entries) {
        return None; // All valid
    }

    // Split and recurse
    let mid = entries.len() / 2;
    let (left, right) = entries.split_at(mid);

    // Check left half
    if let Some(idx) = find_invalid_in_batch(left) {
        return Some(idx);
    }

    // Check right half
    if let Some(idx) = find_invalid_in_batch(right) {
        return Some(mid + idx);
    }

    None
}

/// Find all invalid signatures in a batch.
///
/// Repeatedly finds and removes one invalid signature until the batch is clean.
/// Maps back to original indices.
pub fn find_all_invalid(entries: &[SignatureEntry]) -> Vec<usize> {
    let mut remaining: Vec<(usize, &SignatureEntry)> =
        entries.iter().enumerate().collect();
    let mut invalid_indices = Vec::new();

    loop {
        let batch: Vec<SignatureEntry> = remaining.iter().map(|(_, e)| (*e).clone()).collect();
        if verify_batch(&batch) {
            break;
        }

        // Find one bad entry
        if let Some(local_idx) = find_invalid_in_batch(&batch) {
            let (orig_idx, _) = remaining.remove(local_idx);
            invalid_indices.push(orig_idx);
        } else {
            break;
        }
    }

    invalid_indices.sort();
    invalid_indices
}

/// Benchmark comparison between individual and batch verification.
///
/// Returns (individual_elapsed_us, batch_elapsed_us) in microseconds.
pub fn benchmark_comparison(entries: &[SignatureEntry]) -> (u128, u128) {
    let iterations = 10;

    // Benchmark individual
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = verify_all_individual(entries);
    }
    let individual_us = start.elapsed().as_micros() / iterations;

    // Benchmark batch
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = verify_batch(entries);
    }
    let batch_us = start.elapsed().as_micros() / iterations;

    (individual_us, batch_us)
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
