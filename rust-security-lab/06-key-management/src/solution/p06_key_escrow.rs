//! # Lesson 06: Key Escrow and Split Knowledge (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

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
    pub threshold: usize,
    pub total_shares: usize,
    pub shares: Vec<EscrowShare>,
    pub created_at: u64,
}

/// Split a key into N shares using XOR.
///
/// The last share is the "composite" share: it equals key XOR (all other shares XORed).
/// This means ALL N shares are needed to reconstruct (N-of-N scheme).
pub fn split_key(key: &[u8], agent_ids: &[&str], current_timestamp: u64) -> EscrowRecord {
    let rng = SystemRandom::new();
    let n = agent_ids.len();
    let mut shares = Vec::with_capacity(n);

    // Generate N-1 random shares
    let mut xor_accumulator = vec![0u8; key.len()];
    for (i, agent_id) in agent_ids.iter().enumerate().take(n - 1) {
        let mut share_data = vec![0u8; key.len()];
        rng.fill(&mut share_data).expect("Failed to generate share");
        for (a, s) in xor_accumulator.iter_mut().zip(share_data.iter()) {
            *a ^= s;
        }
        shares.push(EscrowShare {
            agent_id: agent_id.to_string(),
            share_data,
            share_index: i as u8,
        });
    }

    // Last share = key XOR xor_accumulator
    let last_share: Vec<u8> = key.iter().zip(xor_accumulator.iter()).map(|(k, x)| k ^ x).collect();
    shares.push(EscrowShare {
        agent_id: agent_ids[n - 1].to_string(),
        share_data: last_share,
        share_index: (n - 1) as u8,
    });

    EscrowRecord {
        key_id: format!("escrow-{}", current_timestamp),
        threshold: n, // N-of-N for XOR scheme
        total_shares: n,
        shares,
        created_at: current_timestamp,
    }
}

/// Reconstruct a key by XORing all shares.
pub fn reconstruct_key(record: &EscrowRecord) -> Vec<u8> {
    let share_len = record.shares[0].share_data.len();
    let mut result = vec![0u8; share_len];
    for share in &record.shares {
        for (r, s) in result.iter_mut().zip(share.share_data.iter()) {
            *r ^= s;
        }
    }
    result
}

/// Demonstrate that partial shares cannot recover the key.
///
/// XOR only a subset of shares — the result will NOT match the original key.
/// Returns true if the partial reconstruction produces the wrong key.
pub fn demonstrate_partial_insufficiency(record: &EscrowRecord, subset_size: usize) -> bool {
    let share_len = record.shares[0].share_data.len();
    let mut partial = vec![0u8; share_len];
    for share in record.shares.iter().take(subset_size) {
        for (p, s) in partial.iter_mut().zip(share.share_data.iter()) {
            *p ^= s;
        }
    }
    // Full reconstruction for comparison
    let full = reconstruct_key(record);
    partial != full // true means partial was insufficient
}

/// Create a 2-of-N threshold scheme.
///
/// Share 0 = random, Share 1 = key XOR share_0, remaining shares = random decoys.
/// Shares 0 and 1 together can recover the key.
pub fn split_key_threshold(
    key: &[u8],
    agent_ids: &[&str],
    current_timestamp: u64,
) -> EscrowRecord {
    let rng = SystemRandom::new();
    let n = agent_ids.len();
    let mut shares = Vec::with_capacity(n);

    // Share 0: random
    let mut share_0_data = vec![0u8; key.len()];
    rng.fill(&mut share_0_data).expect("Failed to generate share 0");

    // Share 1: key XOR share_0
    let share_1_data: Vec<u8> = key.iter().zip(share_0_data.iter()).map(|(k, s)| k ^ s).collect();

    shares.push(EscrowShare {
        agent_id: agent_ids[0].to_string(),
        share_data: share_0_data,
        share_index: 0,
    });
    shares.push(EscrowShare {
        agent_id: agent_ids[1].to_string(),
        share_data: share_1_data,
        share_index: 1,
    });

    // Remaining shares are decoys
    for (i, agent_id) in agent_ids.iter().enumerate().skip(2) {
        let mut decoy = vec![0u8; key.len()];
        rng.fill(&mut decoy).expect("Failed to generate decoy share");
        shares.push(EscrowShare {
            agent_id: agent_id.to_string(),
            share_data: decoy,
            share_index: i as u8,
        });
    }

    EscrowRecord {
        key_id: format!("threshold-{}", current_timestamp),
        threshold: 2,
        total_shares: n,
        shares,
        created_at: current_timestamp,
    }
}

/// Reconstruct from the first two shares of a threshold scheme.
pub fn reconstruct_from_threshold(record: &EscrowRecord) -> Vec<u8> {
    let share_0 = &record.shares[0].share_data;
    let share_1 = &record.shares[1].share_data;
    share_0.iter().zip(share_1.iter()).map(|(a, b)| a ^ b).collect()
}

/// Audit log for escrow access.
///
/// Returns a list of (agent_id, timestamp, action) tuples.
pub fn audit_escrow_access(
    record: &EscrowRecord,
    accessing_agents: &[&str],
    timestamp: u64,
) -> Vec<(String, u64, String)> {
    accessing_agents
        .iter()
        .map(|agent| {
            (
                agent.to_string(),
                timestamp,
                format!("Accessed share for key {}", record.key_id),
            )
        })
        .collect()
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
            assert_eq!(share.share_data.len(), key.len());
        }
    }

    #[test]
    fn test_reconstruct_recovers_key() {
        let key = test_key();
        let record = split_key(&key, &["alice", "bob", "carol"], 1000);
        let recovered = reconstruct_key(&record);
        assert_eq!(recovered, key);
    }

    #[test]
    fn test_partial_shares_insufficient() {
        let key = test_key();
        let record = split_key(&key, &["alice", "bob", "carol", "dave"], 1000);
        let insufficient = demonstrate_partial_insufficiency(&record, 2);
        assert!(insufficient);
    }

    #[test]
    fn test_threshold_split_and_reconstruct() {
        let key = test_key();
        let record = split_key_threshold(&key, &["alice", "bob", "carol"], 1000);
        let recovered = reconstruct_from_threshold(&record);
        assert_eq!(recovered, key);
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
