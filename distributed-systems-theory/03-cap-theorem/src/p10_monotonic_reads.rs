//! # Exercise: Monotonic Reads
//!
//! ## Theory
//!
//! Monotonic reads guarantee that once a client has read a value, it will
//! never see an older value in subsequent reads. This prevents the "time
//! travel" problem where a client reads a newer value, then on the next read
//! gets an older value from a different replica.
//!
//! This is weaker than read-your-writes: the client doesn't need to see its
//! own writes, but it must never go backward in time.
//!
//! ## Proof / Intuition
//!
//! To implement monotonic reads, the client tracks the timestamp (or version)
//! of the last value it read. On subsequent reads, if a replica returns a
//! value with an older timestamp, the client tries another replica until it
//! finds one with a timestamp at least as recent.
//!
//! If all replicas have timestamps older than the client's last read, the
//! system cannot satisfy monotonic reads and must either block or return an
//! error.
//!
//! ## Implementation Task
//!
//! Implement a `MonotonicReadClient`:
//! - Track the last-read version/timestamp for each key
//! - On read, try replicas until finding one with a sufficiently recent value
//! - Return error if no replica can satisfy the monotonicity constraint
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - Client never sees older data after seeing newer data
//! - Read progression is strictly monotonic
//! - Works correctly when replicas have different states

use std::collections::HashMap;

/// A versioned value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionedEntry {
    pub value: String,
    pub version: u64,
}

/// A single replica.
#[derive(Debug, Clone)]
pub struct Replica {
    pub id: usize,
    pub store: HashMap<String, VersionedEntry>,
}

impl Replica {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            store: HashMap::new(),
        }
    }

    pub fn put(&mut self, key: &str, value: &str, version: u64) {
        self.store.insert(
            key.to_string(),
            VersionedEntry {
                value: value.to_string(),
                version,
            },
        );
    }

    pub fn get(&self, key: &str) -> Option<&VersionedEntry> {
        self.store.get(key)
    }
}

/// Errors for monotonic read operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonotonicReadError {
    /// No replica could provide a value at least as recent as required.
    StaleData { key: String, required_version: u64 },
    /// Key not found on any replica.
    NotFound { key: String },
}

/// A client that guarantees monotonic reads.
///
/// Tracks the last-read version for each key. On each read, ensures the
/// returned value is at least as recent as the previously read value.
pub struct MonotonicReadClient {
    client_id: String,
    /// Maps key -> last-read version.
    last_read_version: HashMap<String, u64>,
}

impl MonotonicReadClient {
    pub fn new(client_id: &str) -> Self {
        Self {
            client_id: client_id.to_string(),
            last_read_version: HashMap::new(),
        }
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// Read a value with monotonic read guarantee.
    ///
    /// Tries each replica in order. If a replica has a value with a version
    /// at least as recent as the last read, returns it. Otherwise, tries
    /// the next replica.
    pub fn read(
        &mut self,
        key: &str,
        replicas: &[Replica],
    ) -> Result<String, MonotonicReadError> {
        let min_version = self.last_read_version.get(key).copied().unwrap_or(0);

        // Try each replica
        for replica in replicas {
            if let Some(entry) = replica.get(key) {
                if entry.version >= min_version {
                    // Update the last-read version
                    self.last_read_version
                        .insert(key.to_string(), entry.version);
                    return Ok(entry.value.clone());
                }
            }
        }

        // Check if any replica had the key at all
        let any_has_key = replicas.iter().any(|r| r.get(key).is_some());
        if any_has_key {
            Err(MonotonicReadError::StaleData {
                key: key.to_string(),
                required_version: min_version,
            })
        } else {
            Err(MonotonicReadError::NotFound {
                key: key.to_string(),
            })
        }
    }

    /// Get the last-read version for a key.
    pub fn last_version(&self, key: &str) -> Option<u64> {
        self.last_read_version.get(key).copied()
    }
}

/// A store that supports monotonic reads.
pub struct MonotonicStore {
    replicas: Vec<Replica>,
    /// Global version counter.
    version_counter: u64,
}

impl MonotonicStore {
    pub fn new(num_replicas: usize) -> Self {
        let replicas = (0..num_replicas).map(Replica::new).collect();
        Self {
            replicas,
            version_counter: 0,
        }
    }

    /// Write a value to all replicas with an incremented version.
    pub fn put(&mut self, key: &str, value: &str) {
        self.version_counter += 1;
        for replica in &mut self.replicas {
            replica.put(key, value, self.version_counter);
        }
    }

    /// Write to a specific replica only (simulating partial replication).
    pub fn put_on_replica(
        &mut self,
        replica_id: usize,
        key: &str,
        value: &str,
        version: u64,
    ) {
        self.replicas[replica_id].put(key, value, version);
    }

    pub fn replicas(&self) -> &[Replica] {
        &self.replicas
    }

    pub fn current_version(&self) -> u64 {
        self.version_counter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_never_sees_older_data() {
        let mut store = MonotonicStore::new(3);
        let mut client = MonotonicReadClient::new("c1");

        // Write and read v1
        store.put("x", "v1");
        let val = client.read("x", store.replicas()).unwrap();
        assert_eq!(val, "v1");

        // Write v2 to all replicas
        store.put("x", "v2");

        // Read should return v2 (or later), never v1
        let val = client.read("x", store.replicas()).unwrap();
        assert_eq!(val, "v2");
    }

    #[test]
    fn progression_is_monotonic() {
        let mut store = MonotonicStore::new(3);
        let mut client = MonotonicReadClient::new("c1");

        let mut versions_seen = Vec::new();

        for i in 1..=5 {
            store.put("k", &format!("v{i}"));
            let val = client.read("k", store.replicas()).unwrap();
            versions_seen.push(val);
        }

        // All values should be strictly increasing (each read gets the latest)
        for window in versions_seen.windows(2) {
            let v1: u64 = window[0][1..].parse().unwrap();
            let v2: u64 = window[1][1..].parse().unwrap();
            assert!(v2 >= v1, "versions should be monotonic: {v1} -> {v2}");
        }
    }

    #[test]
    fn works_with_staggered_replication() {
        let mut store = MonotonicStore::new(3);
        let mut client = MonotonicReadClient::new("c1");

        // Write v1 to all replicas
        store.put("x", "v1");
        client.read("x", store.replicas()).unwrap();

        // Write v2 only to replica 0
        store.put_on_replica(0, "x", "v2", 2);

        // Client should find v2 on replica 0
        let val = client.read("x", store.replicas()).unwrap();
        assert_eq!(val, "v2");
    }

    #[test]
    fn not_found_when_key_missing() {
        let store = MonotonicStore::new(3);
        let mut client = MonotonicReadClient::new("c1");

        let result = client.read("missing", store.replicas());
        assert!(matches!(result, Err(MonotonicReadError::NotFound { .. })));
    }

    #[test]
    fn separate_keys_tracked_independently() {
        let mut store = MonotonicStore::new(3);
        let mut client = MonotonicReadClient::new("c1");

        store.put("a", "1");
        store.put("b", "10");
        client.read("a", store.replicas()).unwrap();
        client.read("b", store.replicas()).unwrap();

        assert_eq!(client.last_version("a"), Some(1));
        assert_eq!(client.last_version("b"), Some(2));
    }
}
