//! # Lesson 04: Tamper-Evident Logs
//!
//! ## The Problem
//!
//! An attacker who gains access to your log storage can:
//! - Delete log entries to cover their tracks
//! - Modify entries to frame someone else
//! - Insert false entries to create alibis
//! - Roll back the log to a pre-attack state
//!
//! Traditional logs in flat files or databases offer no protection against tampering.
//! An admin (or attacker with admin access) can freely edit any entry.
//!
//! ## Defense: Hash Chain
//!
//! A hash chain makes tampering detectable by linking each log entry to its
//! predecessor via a cryptographic hash:
//!
//! ```
//! Entry[0] = { data: "...", prev_hash: "GENESIS", hash: SHA256(data + prev_hash) }
//! Entry[1] = { data: "...", prev_hash: Entry[0].hash, hash: SHA256(data + prev_hash) }
//! Entry[2] = { data: "...", prev_hash: Entry[1].hash, hash: SHA256(data + prev_hash) }
//! ```
//!
//! If ANY entry is modified, its hash changes, which invalidates the next entry's
//! prev_hash, which cascades through the entire chain. An attacker would need to
//! recompute every subsequent hash -- and if the chain root is stored externally
//! (e.g., published to a blockchain or signed by a timestamp authority), this is
//! computationally infeasible.
//!
//! ## What You'll Implement
//!
//! 1. A `LogEntry` struct with data, prev_hash, and hash fields
//! 2. Genesis entry creation (the first entry in a chain)
//! 3. Chain extension -- add new entries that reference the previous hash
//! 4. Chain verification -- detect any tampering
//! 5. Tamper detection -- modify an entry and verify the chain breaks
//! 6. Merkle tree for efficient batch verification

use sha2::{Digest, Sha256};

/// A single entry in a tamper-evident log chain.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// The log data (timestamp, event, message, etc.)
    pub data: String,
    /// Hash of the previous entry in the chain
    pub prev_hash: String,
    /// Hash of this entry: SHA256(data + prev_hash)
    pub hash: String,
}

/// Exercise 1: Compute the SHA-256 hash of a string.
///
/// Return the hash as a lowercase hex string.
///
/// Hints:
/// - Create hasher: `Sha256::new()`
/// - Feed data: `hasher.update(data.as_bytes())`
/// - Get result: `hasher.finalize()`
/// - Convert to hex: `hex::encode(result)`
pub fn compute_hash(data: &str) -> String {
    todo!("Implement SHA-256 hashing")
}

/// Exercise 2: Compute the hash for a log entry.
///
/// The hash is SHA256(data + "|" + prev_hash).
/// The `|` separator prevents ambiguity between data and prev_hash.
///
/// Hints:
/// - Concatenate: `format!("{}|{}", data, prev_hash)`
/// - Hash the concatenated string
pub fn compute_entry_hash(data: &str, prev_hash: &str) -> String {
    todo!("Implement entry hash computation")
}

/// Exercise 3: Create a genesis (first) entry in the chain.
///
/// The genesis entry uses "GENESIS" as the prev_hash.
///
/// Hints:
/// - Use `compute_entry_hash(data, "GENESIS")`
pub fn create_genesis_entry(data: &str) -> LogEntry {
    todo!("Implement genesis entry creation")
}

/// Exercise 4: Create a new entry that chains to the previous entry.
///
/// The new entry's prev_hash is the previous entry's hash.
///
/// Hints:
/// - `prev_hash = previous.hash.clone()`
/// - `hash = compute_entry_hash(data, &prev_hash)`
pub fn chain_entry(data: &str, previous: &LogEntry) -> LogEntry {
    todo!("Implement chain entry creation")
}

/// Exercise 5: Build a complete log chain from a list of data strings.
///
/// The first data string becomes the genesis entry.
/// Each subsequent string chains to the previous entry.
///
/// Hints:
/// - Create genesis from first element
/// - Iterate over remaining elements, chaining each
/// - Return Vec<LogEntry>
pub fn build_log_chain(data_entries: &[&str]) -> Vec<LogEntry> {
    todo!("Implement log chain building")
}

/// Exercise 6: Verify the integrity of a log chain.
///
/// Check:
/// 1. The first entry's prev_hash is "GENESIS"
/// 2. Each entry's hash matches recomputed hash
/// 3. Each entry's prev_hash matches the previous entry's hash
///
/// Return Ok(()) if valid, Err(description) if tampered.
///
/// Hints:
/// - Check genesis: `entries[0].prev_hash == "GENESIS"`
/// - Recompute each hash and compare
/// - Check prev_hash linkage
pub fn verify_chain(entries: &[LogEntry]) -> Result<(), String> {
    todo!("Implement chain verification")
}

/// Exercise 7: Simulate tampering by modifying an entry's data.
///
/// Create a new chain where the entry at `index` has its data replaced,
/// but the hash is NOT recomputed (simulating an attacker who doesn't
/// have the hashing key or doesn't know to recompute).
///
/// Hints:
/// - Clone the entries
/// - Modify `entries[index].data`
/// - Leave hash unchanged (this is the tamper)
pub fn tamper_entry(entries: &[LogEntry], index: usize, new_data: &str) -> Vec<LogEntry> {
    todo!("Implement tamper simulation")
}

/// Exercise 8: Build a Merkle tree from log entries.
///
/// A Merkle tree allows efficient verification of individual entries.
/// Hash pairs of entries bottom-up until you reach a single root hash.
///
/// For simplicity, if the number of entries at a level is odd, duplicate
/// the last entry.
///
/// Return the root hash as a hex string.
///
/// Hints:
/// - Start with entry hashes
/// - Pair and hash: `hash(left + "|" + right)`
/// - If odd count, duplicate last
/// - Repeat until one hash remains
pub fn merkle_root(entries: &[LogEntry]) -> String {
    todo!("Implement Merkle tree root computation")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash_deterministic() {
        let h1 = compute_hash("hello");
        let h2 = compute_hash("hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_compute_hash_different_inputs() {
        let h1 = compute_hash("hello");
        let h2 = compute_hash("world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_genesis_entry() {
        let entry = create_genesis_entry("First log entry");
        assert_eq!(entry.prev_hash, "GENESIS");
        assert!(!entry.hash.is_empty());
    }

    #[test]
    fn test_chain_entry_links() {
        let genesis = create_genesis_entry("First");
        let second = chain_entry("Second", &genesis);
        assert_eq!(second.prev_hash, genesis.hash);
    }

    #[test]
    fn test_build_log_chain() {
        let entries = build_log_chain(&["First", "Second", "Third"]);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].prev_hash, "GENESIS");
        assert_eq!(entries[1].prev_hash, entries[0].hash);
        assert_eq!(entries[2].prev_hash, entries[1].hash);
    }

    #[test]
    fn test_verify_valid_chain() {
        let entries = build_log_chain(&["A", "B", "C"]);
        assert!(verify_chain(&entries).is_ok());
    }

    #[test]
    fn test_detect_tamper() {
        let entries = build_log_chain(&["A", "B", "C"]);
        let tampered = tamper_entry(&entries, 1, "MODIFIED");
        assert!(verify_chain(&tampered).is_err());
    }

    #[test]
    fn test_merkle_root_deterministic() {
        let entries = build_log_chain(&["A", "B", "C", "D"]);
        let r1 = merkle_root(&entries);
        let r2 = merkle_root(&entries);
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_merkle_root_different_data() {
        let e1 = build_log_chain(&["A", "B"]);
        let e2 = build_log_chain(&["A", "C"]);
        assert_ne!(merkle_root(&e1), merkle_root(&e2));
    }
}
