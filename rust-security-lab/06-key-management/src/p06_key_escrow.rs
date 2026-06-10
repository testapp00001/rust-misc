//! # Lesson 06: Key Escrow and Split Knowledge
//!
//! ## What Is Key Escrow?
//!
//! Key escrow is the practice of entrusting a cryptographic key (or components of it)
//! to one or more third parties, called "escrow agents." The key can be recovered only
//! when a sufficient number of agents cooperate.
//!
//! ## Why Key Escrow?
//!
//! 1. **Disaster recovery**: If a key holder dies or leaves the company, data is not lost
//! 2. **Legal compliance**: Some regulations require ability to decrypt on lawful order
//! 3. **Split knowledge**: No single person ever holds the full key
//!
//! ## Attack Scenario: Single Point of Trust
//!
//! If one person holds all keys, they are:
//! - A single point of failure (bus factor = 1)
//! - A high-value target for coercion or bribery
//! - A potential insider threat
//!
//! Split knowledge via Shamir's Secret Sharing or M-of-N escrow mitigates this.
//!
//! ## Shamir's Secret Sharing (Simplified)
//!
//! ```text
//! secret = 42
//! Generate a random polynomial: f(x) = 42 + 7x + 3x^2
//! Share 1: f(1) = 42 + 7 + 3 = 52
//! Share 2: f(2) = 42 + 14 + 12 = 68
//! Share 3: f(3) = 42 + 21 + 27 = 90
//!
//! Any 2 of 3 shares can reconstruct the polynomial (and thus the secret).
//! 1 share alone reveals nothing about the secret.
//! ```
//!
//! For this lesson, we'll use a simplified XOR-based split (not true Shamir's, but
//! illustrates the concept). Real systems should use the `ssss` or `threshold-secret-sharing` crate.
//!
//! ## Security Notes
//!
//! - Escrow agents must be independent (different organizations, jurisdictions)
//! - Access to escrowed keys must be audited and logged
//! - Escrow recovery procedures should be tested regularly
//! - Consider time-locks: escrowed key is only recoverable after a delay

use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};

/// An escrow share — one piece of a split key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowShare {
    pub agent_id: String,
    pub share_data: Vec<u8>,
    pub share_index: u8,
}

/// Metadata about an escrowed key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowRecord {
    pub key_id: String,
    pub threshold: usize,     // M in M-of-N
    pub total_shares: usize,  // N in M-of-N
    pub shares: Vec<EscrowShare>,
    pub created_at: u64,
}

/// Exercise 1: Split a key into N shares using XOR (simplified).
///
/// For a simplified 2-of-N scheme:
/// - Generate N-1 random shares
/// - The Nth share = key XOR (share1 XOR share2 XOR ... XOR shareN-1)
/// - Any 2 shares can recover the key (one real share + the composite share, or
///   any subset of 2 that includes enough info)
///
/// For simplicity, implement as: generate (N-1) random shares, last share = key XOR all others.
/// This means you need ALL shares to recover (N-of-N), but it illustrates the concept.
///
/// Hints:
/// - Use `ring::SystemRandom` for random shares
/// - XOR all random shares together, then XOR with the key to get the last share
/// - Each share is the same length as the key
pub fn split_key(key: &[u8], agent_ids: &[&str], current_timestamp: u64) -> EscrowRecord {
    todo!("Split a key into N XOR-based shares")
}

/// Exercise 2: Reconstruct a key from all its shares.
///
/// XOR all shares together to recover the original key.
///
/// Hints:
/// - XOR all share_data vectors element-wise
/// - The result is the original key
pub fn reconstruct_key(record: &EscrowRecord) -> Vec<u8> {
    todo!("Reconstruct key by XORing all shares")
}

/// Exercise 3: Verify that a subset of shares cannot recover the key.
///
/// With the XOR-based N-of-N scheme, fewer than N shares should NOT reveal the key.
/// Demonstrate this by trying to "recover" with fewer shares.
///
/// Returns true if partial shares produce incorrect key (as expected).
pub fn demonstrate_partial_insufficiency(record: &EscrowRecord, subset_size: usize) -> bool {
    todo!("Show that partial shares cannot recover the key")
}

/// Exercise 4: Create a 2-of-N threshold scheme using additive sharing.
///
/// For educational purposes, implement a simple 2-of-N scheme:
/// - share_0 = random bytes
/// - share_1 = key XOR share_0
/// - remaining shares = random (decoys)
///
/// Any 2 shares that include share_0 and share_1 can recover the key.
/// The first two agents listed are the real shareholders.
///
/// Hints:
/// - Generate one random share of key length
/// - Second share = key XOR first share
/// - Additional shares are random noise (for plausible deniability / mixing)
pub fn split_key_threshold(
    key: &[u8],
    agent_ids: &[&str],
    current_timestamp: u64,
) -> EscrowRecord {
    todo!("Create a 2-of-N threshold escrow")
}

/// Exercise 5: Reconstruct from threshold shares.
///
/// Given a 2-of-N escrow record, recover the key using the first two shares.
///
/// Hints:
/// - share_0 XOR share_1 = key (since share_1 = key XOR share_0)
pub fn reconstruct_from_threshold(record: &EscrowRecord) -> Vec<u8> {
    todo!("Reconstruct key from first two threshold shares")
}

/// Exercise 6: Audit log for escrow access.
///
/// Simulate an audit trail: each time a key is accessed, log (agent_id, timestamp, action).
/// Return the list of audit entries.
///
/// Hints:
/// - Create a simple struct for audit entries
/// - Record each share access attempt
pub fn audit_escrow_access(record: &EscrowRecord, accessing_agents: &[&str], timestamp: u64) -> Vec<(String, u64, String)> {
    todo!("Generate audit log entries for escrow access")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> Vec<u8> {
        vec![0xAB; 32]
    }

    #[test]
    fn test_split_creates_correct_number_of_shares() {
        let record = split_key(&test_key(), &["alice", "bob", "carol"], 1000);
        assert_eq!(record.shares.len(), 3);
        assert_eq!(record.total_shares, 3);
    }

    #[test]
    fn test_split_share_lengths() {
        let key = test_key();
        let record = split_key(&key, &["alice", "bob"], 1000);
        for share in &record.shares {
            assert_eq!(share.share_data.len(), key.len(), "Each share must be same length as key");
        }
    }

    #[test]
    fn test_reconstruct_recovers_key() {
        let key = test_key();
        let record = split_key(&key, &["alice", "bob", "carol"], 1000);
        let recovered = reconstruct_key(&record);
        assert_eq!(recovered, key, "Reconstructed key must match original");
    }

    #[test]
    fn test_partial_shares_insufficient() {
        let key = test_key();
        let record = split_key(&key, &["alice", "bob", "carol", "dave"], 1000);
        let insufficient = demonstrate_partial_insufficiency(&record, 2);
        assert!(insufficient, "Partial shares should NOT recover the correct key");
    }

    #[test]
    fn test_threshold_split_and_reconstruct() {
        let key = test_key();
        let record = split_key_threshold(&key, &["alice", "bob", "carol"], 1000);
        let recovered = reconstruct_from_threshold(&record);
        assert_eq!(recovered, key, "Threshold reconstruction must recover the key");
    }

    #[test]
    fn test_threshold_share_count() {
        let record = split_key_threshold(&test_key(), &["alice", "bob", "carol", "dave"], 1000);
        assert_eq!(record.shares.len(), 4);
        assert_eq!(record.threshold, 2);
    }

    #[test]
    fn test_audit_log_entries() {
        let record = split_key(&test_key(), &["alice", "bob"], 1000);
        let audit = audit_escrow_access(&record, &["alice", "bob"], 2000);
        assert_eq!(audit.len(), 2);
        assert_eq!(audit[0].0, "alice");
        assert_eq!(audit[1].0, "bob");
    }

    #[test]
    fn test_different_keys_produce_different_shares() {
        let key1 = vec![0xAA; 32];
        let key2 = vec![0xBB; 32];
        let record1 = split_key(&key1, &["alice", "bob"], 1000);
        let record2 = split_key(&key2, &["alice", "bob"], 1000);
        assert_ne!(record1.shares[0].share_data, record2.shares[0].share_data);
    }
}
