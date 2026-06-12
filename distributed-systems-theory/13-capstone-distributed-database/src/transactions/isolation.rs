//! Transaction isolation levels and transaction state tracking.
//!
//! Different isolation levels provide varying guarantees about the visibility of
//! concurrent transactions' writes. This module defines the levels and a simplified
//! `Transaction` that tracks its read and write sets for validation purposes.

use std::fmt;

/// Supported transaction isolation levels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IsolationLevel {
    /// Only committed data is visible; non-repeatable reads are possible.
    ReadCommitted,
    /// Transactions see a consistent snapshot taken at start time.
    SnapshotIsolation,
    /// Fully serializable execution; the strongest guarantee.
    Serializable,
}

impl fmt::Display for IsolationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IsolationLevel::ReadCommitted => write!(f, "ReadCommitted"),
            IsolationLevel::SnapshotIsolation => write!(f, "SnapshotIsolation"),
            IsolationLevel::Serializable => write!(f, "Serializable"),
        }
    }
}

/// A simplified transaction that tracks its read set, write set, and isolation level.
#[derive(Debug)]
pub struct Transaction {
    /// Unique identifier for this transaction.
    pub id: u64,
    /// The isolation level governing this transaction.
    isolation_level: IsolationLevel,
    /// Keys that have been read during this transaction.
    pub read_set: Vec<String>,
    /// Key-value pairs written during this transaction.
    pub write_set: Vec<(String, String)>,
    /// Whether the transaction is still active (not yet committed or aborted).
    active: bool,
}

impl Transaction {
    /// Create a new active transaction with the given id and isolation level.
    ///
    /// # Arguments
    ///
    /// * `id` - A unique transaction identifier.
    /// * `isolation_level` - The isolation level for this transaction.
    pub fn new(id: u64, isolation_level: IsolationLevel) -> Self {
        Self {
            id,
            isolation_level,
            read_set: Vec::new(),
            write_set: Vec::new(),
            active: true,
        }
    }

    /// Record a read of the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key that was read.
    pub fn read(&mut self, key: &str) {
        self.read_set.push(key.to_string());
    }

    /// Record a write of the given key-value pair.
    ///
    /// # Arguments
    ///
    /// * `key` - The key being written.
    /// * `value` - The value being written.
    pub fn write(&mut self, key: String, value: String) {
        self.write_set.push((key, value));
    }

    /// Validate the transaction.
    ///
    /// In a real implementation this would check for conflicts against concurrent
    /// transactions. Here it always returns `true`.
    pub fn validate(&self) -> bool {
        true
    }

    /// Commit the transaction.
    ///
    /// Validates the transaction and marks it as inactive (committed). Panics if
    /// validation fails, which in this simplified version cannot happen.
    pub fn commit(&mut self) {
        assert!(self.validate(), "transaction validation failed");
        self.active = false;
    }

    /// Return `true` if the transaction is still active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Return a reference to the transaction's isolation level.
    pub fn get_level(&self) -> &IsolationLevel {
        &self.isolation_level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_transaction() {
        let tx = Transaction::new(1, IsolationLevel::ReadCommitted);
        assert!(tx.is_active());
        assert_eq!(*tx.get_level(), IsolationLevel::ReadCommitted);
    }

    #[test]
    fn test_read_and_write() {
        let mut tx = Transaction::new(1, IsolationLevel::SnapshotIsolation);
        tx.read("key1".into());
        tx.write("key2".into(), "value2".into());
        assert_eq!(tx.read_set, vec!["key1".to_string()]);
        assert_eq!(tx.write_set, vec![("key2".into(), "value2".into())]);
    }

    #[test]
    fn test_commit() {
        let mut tx = Transaction::new(1, IsolationLevel::Serializable);
        tx.write("k".into(), "v".into());
        tx.commit();
        assert!(!tx.is_active());
    }

    #[test]
    fn test_validate_always_true() {
        let tx = Transaction::new(1, IsolationLevel::ReadCommitted);
        assert!(tx.validate());
    }

    #[test]
    fn test_isolation_level_display() {
        assert_eq!(IsolationLevel::ReadCommitted.to_string(), "ReadCommitted");
        assert_eq!(
            IsolationLevel::SnapshotIsolation.to_string(),
            "SnapshotIsolation"
        );
        assert_eq!(IsolationLevel::Serializable.to_string(), "Serializable");
    }
}
