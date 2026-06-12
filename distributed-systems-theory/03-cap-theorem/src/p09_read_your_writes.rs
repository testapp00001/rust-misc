//! # Exercise: Read-Your-Writes Consistency
//!
//! ## Theory
//!
//! Read-your-writes (RYW) is a session consistency guarantee: a client is
//! always guaranteed to see the effects of its own writes. After a client
//! writes a value, subsequent reads from that client will return at least
//! that value (or a more recent one).
//!
//! RYW is weaker than linearizability because other clients may not see the
//! write immediately. It is useful for user-facing applications where a user
//! should always see their own changes (e.g., after editing a profile).
//!
//! ## Proof / Intuition
//!
//! The simplest way to implement RYW is to pin a client's reads to the same
//! replica that processed its writes. Since that replica has the latest write,
//! the client is guaranteed to see it.
//!
//! The trade-off: other clients reading from different replicas may see stale
//! data. The system is not globally consistent, but each client's session is
//! consistent.
//!
//! ## Implementation Task
//!
//! Implement a `SessionStickyClient`:
//! - Pin each client to a specific replica
//! - Writes go to the pinned replica
//! - Reads come from the pinned replica
//! - Guarantee: client always sees its own writes
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - Client always sees its own writes
//! - Other clients may not see writes immediately
//! - Writes are eventually replicated

use std::collections::HashMap;

/// A replica node.
#[derive(Debug, Clone)]
pub struct Replica {
    pub id: usize,
    pub store: HashMap<String, String>,
}

impl Replica {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            store: HashMap::new(),
        }
    }

    pub fn put(&mut self, key: &str, value: &str) {
        self.store.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.store.get(key).map(|s| s.as_str())
    }
}

/// A session-sticky client that is pinned to a specific replica.
///
/// All reads and writes go to the same replica, guaranteeing
/// read-your-writes consistency.
#[derive(Debug)]
pub struct SessionStickyClient {
    /// Unique client identifier.
    pub client_id: String,
    /// The replica this client is pinned to.
    pinned_replica_id: usize,
}

impl SessionStickyClient {
    /// Create a new session-sticky client pinned to a replica.
    pub fn new(client_id: &str, pinned_replica_id: usize) -> Self {
        Self {
            client_id: client_id.to_string(),
            pinned_replica_id,
        }
    }

    /// Return the ID of the replica this client is pinned to.
    pub fn pinned_replica(&self) -> usize {
        self.pinned_replica_id
    }
}

/// A replicated store that supports session-sticky clients.
pub struct SessionStore {
    replicas: Vec<Replica>,
    /// Client ID -> pinned replica index.
    client_pins: HashMap<String, usize>,
}

impl SessionStore {
    pub fn new(num_replicas: usize) -> Self {
        let replicas = (0..num_replicas).map(Replica::new).collect();
        Self {
            replicas,
            client_pins: HashMap::new(),
        }
    }

    /// Register a client and pin it to a specific replica.
    pub fn register_client(&mut self, client_id: &str, replica_id: usize) {
        assert!(
            replica_id < self.replicas.len(),
            "replica {replica_id} does not exist"
        );
        self.client_pins
            .insert(client_id.to_string(), replica_id);
    }

    /// Write a value through a session-sticky client.
    /// The write goes to the client's pinned replica.
    pub fn write(
        &mut self,
        client_id: &str,
        key: &str,
        value: &str,
    ) {
        let &replica_id = self
            .client_pins
            .get(client_id)
            .expect("client not registered");
        self.replicas[replica_id].put(key, value);
    }

    /// Read a value through a session-sticky client.
    /// The read comes from the client's pinned replica.
    pub fn read(&self, client_id: &str, key: &str) -> Option<String> {
        let &replica_id = self
            .client_pins
            .get(client_id)
            .expect("client not registered");
        self.replicas[replica_id].get(key).map(|s| s.to_string())
    }

    /// Read from a specific replica (for testing).
    pub fn read_from_replica(
        &self,
        replica_id: usize,
        key: &str,
    ) -> Option<String> {
        self.replicas[replica_id]
            .get(key)
            .map(|s| s.to_string())
    }

    /// Replicate a key-value pair to all replicas (background sync).
    pub fn replicate(&mut self, key: &str, value: &str) {
        for replica in &mut self.replicas {
            replica.put(key, value);
        }
    }

    pub fn replicas(&self) -> &[Replica] {
        &self.replicas
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_sees_own_write() {
        let mut store = SessionStore::new(3);
        store.register_client("alice", 0);

        store.write("alice", "name", "Alice");
        let result = store.read("alice", "name").unwrap();
        assert_eq!(result, "Alice");
    }

    #[test]
    fn other_client_may_not_see_write_immediately() {
        let mut store = SessionStore::new(3);
        store.register_client("alice", 0);
        store.register_client("bob", 1);

        // Alice writes to replica 0
        store.write("alice", "x", "from_alice");

        // Bob reads from replica 1 (different replica)
        let result = store.read("bob", "x");
        assert!(
            result.is_none(),
            "bob should not see alice's write until replicated"
        );
    }

    #[test]
    fn after_replication_both_see_value() {
        let mut store = SessionStore::new(3);
        store.register_client("alice", 0);
        store.register_client("bob", 1);

        store.write("alice", "x", "hello");
        store.replicate("x", "hello");

        assert_eq!(store.read("alice", "x").unwrap(), "hello");
        assert_eq!(store.read("bob", "x").unwrap(), "hello");
    }

    #[test]
    fn multiple_writes_visible_in_order() {
        let mut store = SessionStore::new(3);
        store.register_client("alice", 0);

        store.write("alice", "counter", "1");
        store.write("alice", "counter", "2");
        store.write("alice", "counter", "3");

        assert_eq!(store.read("alice", "counter").unwrap(), "3");
    }

    #[test]
    fn different_clients_on_different_replicas() {
        let mut store = SessionStore::new(3);
        store.register_client("c1", 0);
        store.register_client("c2", 1);
        store.register_client("c3", 2);

        store.write("c1", "a", "1");
        store.write("c2", "b", "2");
        store.write("c3", "c", "3");

        // Each client sees its own writes
        assert_eq!(store.read("c1", "a").unwrap(), "1");
        assert_eq!(store.read("c2", "b").unwrap(), "2");
        assert_eq!(store.read("c3", "c").unwrap(), "3");
    }
}
