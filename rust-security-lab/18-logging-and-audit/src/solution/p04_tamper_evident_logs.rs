//! # Lesson 04: Tamper-Evident Logs (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub data: String,
    pub prev_hash: String,
    pub hash: String,
}

/// Compute SHA-256 hash of a string, returned as lowercase hex.
pub fn compute_hash(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Compute the hash for a log entry: SHA256(data + "|" + prev_hash).
pub fn compute_entry_hash(data: &str, prev_hash: &str) -> String {
    compute_hash(&format!("{}|{}", data, prev_hash))
}

/// Create a genesis (first) entry in the chain.
pub fn create_genesis_entry(data: &str) -> LogEntry {
    let prev_hash = "GENESIS".to_string();
    let hash = compute_entry_hash(data, &prev_hash);
    LogEntry {
        data: data.to_string(),
        prev_hash,
        hash,
    }
}

/// Create a new entry that chains to the previous entry.
pub fn chain_entry(data: &str, previous: &LogEntry) -> LogEntry {
    let prev_hash = previous.hash.clone();
    let hash = compute_entry_hash(data, &prev_hash);
    LogEntry {
        data: data.to_string(),
        prev_hash,
        hash,
    }
}

/// Build a complete log chain from a list of data strings.
pub fn build_log_chain(data_entries: &[&str]) -> Vec<LogEntry> {
    if data_entries.is_empty() {
        return Vec::new();
    }

    let mut entries = Vec::with_capacity(data_entries.len());
    entries.push(create_genesis_entry(data_entries[0]));

    for i in 1..data_entries.len() {
        let prev = &entries[i - 1];
        entries.push(chain_entry(data_entries[i], prev));
    }

    entries
}

/// Verify the integrity of a log chain.
pub fn verify_chain(entries: &[LogEntry]) -> Result<(), String> {
    if entries.is_empty() {
        return Ok(());
    }

    // Check genesis
    if entries[0].prev_hash != "GENESIS" {
        return Err("First entry is not a genesis entry".to_string());
    }
    let expected_hash = compute_entry_hash(&entries[0].data, &entries[0].prev_hash);
    if entries[0].hash != expected_hash {
        return Err("Genesis entry hash mismatch".to_string());
    }

    // Check chain linkage
    for i in 1..entries.len() {
        if entries[i].prev_hash != entries[i - 1].hash {
            return Err(format!(
                "Chain broken at entry {}: prev_hash doesn't match previous entry's hash",
                i
            ));
        }
        let expected_hash = compute_entry_hash(&entries[i].data, &entries[i].prev_hash);
        if entries[i].hash != expected_hash {
            return Err(format!("Entry {} hash mismatch: data was tampered", i));
        }
    }

    Ok(())
}

/// Simulate tampering by modifying an entry's data without recomputing the hash.
pub fn tamper_entry(entries: &[LogEntry], index: usize, new_data: &str) -> Vec<LogEntry> {
    let mut tampered = entries.to_vec();
    if index < tampered.len() {
        tampered[index].data = new_data.to_string();
        // Note: hash is NOT recomputed -- this is the tamper
    }
    tampered
}

/// Build a Merkle tree from log entries and return the root hash.
pub fn merkle_root(entries: &[LogEntry]) -> String {
    if entries.is_empty() {
        return compute_hash("EMPTY_TREE");
    }

    let mut level: Vec<String> = entries.iter().map(|e| e.hash.clone()).collect();

    while level.len() > 1 {
        // If odd, duplicate the last entry
        if level.len() % 2 != 0 {
            let last = level.last().unwrap().clone();
            level.push(last);
        }

        let mut next_level = Vec::new();
        for pair in level.chunks(2) {
            let combined = format!("{}|{}", pair[0], pair[1]);
            next_level.push(compute_hash(&combined));
        }
        level = next_level;
    }

    level[0].clone()
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
