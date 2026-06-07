//! # Transaction Patterns
//!
//! Transactions ensure data consistency by grouping operations that must all
//! succeed or all fail. This lesson covers transaction lifecycle, savepoints,
//! isolation levels, and retry patterns for handling serialization failures.
//!
//! ## Key Concepts
//! - BEGIN, COMMIT, ROLLBACK semantics
//! - Savepoints for partial rollback
//! - Isolation levels and their trade-offs
//! - Retry on serialization failure
//! - Deadlock detection and handling
//! - Transaction-scoped resources

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// 1. Transaction State Machine
// ---------------------------------------------------------------------------

/// States of a database transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    /// No active transaction.
    Idle,
    /// Transaction is active.
    Active,
    /// Transaction has been committed.
    Committed,
    /// Transaction has been rolled back.
    RolledBack,
    /// Transaction failed due to an error.
    Failed,
}

/// Represents an isolation level for transactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    /// Lowest isolation, allows dirty reads.
    ReadUncommitted,
    /// Prevents dirty reads.
    ReadCommitted,
    /// Prevents dirty and non-repeatable reads.
    RepeatableRead,
    /// Highest isolation, fully serializable.
    Serializable,
}

impl IsolationLevel {
    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::ReadUncommitted => "READ UNCOMMITTED",
            Self::ReadCommitted => "READ COMMITTED",
            Self::RepeatableRead => "REPEATABLE READ",
            Self::Serializable => "SERIALIZABLE",
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Transaction Manager
// ---------------------------------------------------------------------------

/// A savepoint within a transaction.
#[derive(Debug, Clone)]
pub struct Savepoint {
    pub name: String,
    pub created_at: Instant,
    pub state: TransactionState,
}

/// Manages a database transaction with savepoint support.
#[derive(Debug)]
pub struct Transaction {
    id: u64,
    isolation: IsolationLevel,
    state: TransactionState,
    savepoints: Vec<Savepoint>,
    operations: Vec<String>,
    started_at: Instant,
    read_only: bool,
}

impl Transaction {
    pub fn new(id: u64, isolation: IsolationLevel) -> Self {
        Self {
            id,
            isolation,
            state: TransactionState::Active,
            savepoints: Vec::new(),
            operations: Vec::new(),
            started_at: Instant::now(),
            read_only: false,
        }
    }

    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }

    pub fn state(&self) -> TransactionState {
        self.state
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn isolation(&self) -> IsolationLevel {
        self.isolation
    }

    pub fn duration(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Record a SQL operation within this transaction.
    pub fn execute(&mut self, sql: &str) -> Result<(), TransactionError> {
        if self.state != TransactionState::Active {
            return Err(TransactionError::NotActive);
        }
        if self.read_only && !sql.trim_start().to_uppercase().starts_with("SELECT") {
            return Err(TransactionError::ReadOnlyViolation);
        }
        self.operations.push(sql.into());
        Ok(())
    }

    /// Create a savepoint.
    pub fn savepoint(&mut self, name: impl Into<String>) -> Result<(), TransactionError> {
        if self.state != TransactionState::Active {
            return Err(TransactionError::NotActive);
        }
        let name = name.into();
        self.savepoints.push(Savepoint {
            name: name.clone(),
            created_at: Instant::now(),
            state: TransactionState::Active,
        });
        self.operations.push(format!("SAVEPOINT {name}"));
        Ok(())
    }

    /// Rollback to a savepoint.
    pub fn rollback_to_savepoint(&mut self, name: &str) -> Result<(), TransactionError> {
        if self.state != TransactionState::Active {
            return Err(TransactionError::NotActive);
        }

        let sp = self
            .savepoints
            .iter_mut()
            .find(|sp| sp.name == name)
            .ok_or_else(|| TransactionError::SavepointNotFound(name.into()))?;

        sp.state = TransactionState::RolledBack;
        self.operations
            .push(format!("ROLLBACK TO SAVEPOINT {name}"));
        Ok(())
    }

    /// Release a savepoint.
    pub fn release_savepoint(&mut self, name: &str) -> Result<(), TransactionError> {
        if self.state != TransactionState::Active {
            return Err(TransactionError::NotActive);
        }

        let sp = self
            .savepoints
            .iter_mut()
            .find(|sp| sp.name == name)
            .ok_or_else(|| TransactionError::SavepointNotFound(name.into()))?;

        sp.state = TransactionState::Committed;
        self.operations.push(format!("RELEASE SAVEPOINT {name}"));
        Ok(())
    }

    /// Commit the transaction.
    pub fn commit(mut self) -> Result<Vec<String>, TransactionError> {
        if self.state != TransactionState::Active {
            return Err(TransactionError::NotActive);
        }
        self.state = TransactionState::Committed;
        self.operations.push("COMMIT".into());
        Ok(self.operations)
    }

    /// Rollback the transaction.
    pub fn rollback(mut self) -> Vec<String> {
        self.state = TransactionState::RolledBack;
        self.operations.push("ROLLBACK".into());
        self.operations
    }

    /// Mark the transaction as failed.
    pub fn fail(&mut self, reason: &str) {
        self.state = TransactionState::Failed;
        self.operations
            .push(format!("-- FAILED: {reason}"));
    }

    pub fn operations(&self) -> &[String] {
        &self.operations
    }

    pub fn savepoint_count(&self) -> usize {
        self.savepoints.len()
    }
}

// ---------------------------------------------------------------------------
// 3. Transaction Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
    #[error("transaction is not active")]
    NotActive,

    #[error("savepoint not found: {0}")]
    SavepointNotFound(String),

    #[error("read-only transaction cannot execute write operations")]
    ReadOnlyViolation,

    #[error("serialization failure, retry recommended")]
    SerializationFailure,

    #[error("deadlock detected")]
    DeadlockDetected,

    #[error("transaction timed out after {0:?}")]
    Timeout(Duration),
}

// ---------------------------------------------------------------------------
// 4. Retry Logic
// ---------------------------------------------------------------------------

/// Configuration for transaction retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub retryable_errors: Vec<RetryableError>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RetryableError {
    SerializationFailure,
    DeadlockDetected,
    ConnectionLost,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            retryable_errors: vec![
                RetryableError::SerializationFailure,
                RetryableError::DeadlockDetected,
            ],
        }
    }
}

impl RetryConfig {
    pub fn should_retry(&self, error: &RetryableError, attempt: u32) -> bool {
        attempt < self.max_attempts && self.retryable_errors.contains(error)
    }

    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let delay = self.base_delay * 2u32.saturating_pow(attempt);
        delay.min(self.max_delay)
    }
}

/// Execute a transaction with automatic retry on serialization failures.
pub fn with_retry<F, T, E>(
    config: &RetryConfig,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Debug,
{
    let mut last_error = None;

    for attempt in 0..config.max_attempts {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                if attempt < config.max_attempts - 1 {
                    let delay = config.delay_for_attempt(attempt);
                    std::thread::sleep(delay);
                }
            }
        }
    }

    Err(last_error.unwrap())
}

// ---------------------------------------------------------------------------
// 5. Transaction Log
// ---------------------------------------------------------------------------

/// Records transaction activity for auditing and debugging.
#[derive(Debug)]
pub struct TransactionLog {
    entries: Vec<TransactionLogEntry>,
}

#[derive(Debug, Clone)]
pub struct TransactionLogEntry {
    pub transaction_id: u64,
    pub operation: String,
    pub timestamp: Instant,
    pub duration: Duration,
    pub success: bool,
}

impl TransactionLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn record(&mut self, entry: TransactionLogEntry) {
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &[TransactionLogEntry] {
        &self.entries
    }

    pub fn successful_count(&self) -> usize {
        self.entries.iter().filter(|e| e.success).count()
    }

    pub fn failed_count(&self) -> usize {
        self.entries.iter().filter(|e| !e.success).count()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_lifecycle() {
        let tx = Transaction::new(1, IsolationLevel::ReadCommitted);
        assert_eq!(tx.state(), TransactionState::Active);

        let ops = tx.commit().unwrap();
        assert!(ops.contains(&"COMMIT".to_string()));
    }

    #[test]
    fn test_transaction_rollback() {
        let tx = Transaction::new(1, IsolationLevel::ReadCommitted);
        let ops = tx.rollback();
        assert!(ops.contains(&"ROLLBACK".to_string()));
    }

    #[test]
    fn test_transaction_execute() {
        let mut tx = Transaction::new(1, IsolationLevel::ReadCommitted);
        tx.execute("INSERT INTO users (name) VALUES ('Alice')").unwrap();
        tx.execute("INSERT INTO users (name) VALUES ('Bob')").unwrap();
        assert_eq!(tx.operations().len(), 2);
    }

    #[test]
    fn test_transaction_read_only_violation() {
        let mut tx = Transaction::new(1, IsolationLevel::ReadCommitted).read_only();
        let result = tx.execute("INSERT INTO users (name) VALUES ('Alice')");
        assert!(result.is_err());
    }

    #[test]
    fn test_transaction_read_only_select_ok() {
        let mut tx = Transaction::new(1, IsolationLevel::ReadCommitted).read_only();
        assert!(tx.execute("SELECT * FROM users").is_ok());
    }

    #[test]
    fn test_savepoint() {
        let mut tx = Transaction::new(1, IsolationLevel::ReadCommitted);
        tx.savepoint("sp1").unwrap();
        assert_eq!(tx.savepoint_count(), 1);

        tx.execute("INSERT INTO users (name) VALUES ('temp')").unwrap();
        tx.rollback_to_savepoint("sp1").unwrap();

        let sp = tx.savepoints.iter().find(|s| s.name == "sp1").unwrap();
        assert_eq!(sp.state, TransactionState::RolledBack);
    }

    #[test]
    fn test_savepoint_release() {
        let mut tx = Transaction::new(1, IsolationLevel::ReadCommitted);
        tx.savepoint("sp1").unwrap();
        tx.release_savepoint("sp1").unwrap();

        let sp = tx.savepoints.iter().find(|s| s.name == "sp1").unwrap();
        assert_eq!(sp.state, TransactionState::Committed);
    }

    #[test]
    fn test_savepoint_not_found() {
        let mut tx = Transaction::new(1, IsolationLevel::ReadCommitted);
        let result = tx.rollback_to_savepoint("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_transaction_not_active_after_commit() {
        let tx = Transaction::new(1, IsolationLevel::ReadCommitted);
        let _ = tx.commit();
        // Can't test further since commit consumes self
    }

    #[test]
    fn test_isolation_level_sql() {
        assert_eq!(IsolationLevel::ReadUncommitted.as_sql(), "READ UNCOMMITTED");
        assert_eq!(IsolationLevel::ReadCommitted.as_sql(), "READ COMMITTED");
        assert_eq!(IsolationLevel::RepeatableRead.as_sql(), "REPEATABLE READ");
        assert_eq!(IsolationLevel::Serializable.as_sql(), "SERIALIZABLE");
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.base_delay, Duration::from_millis(100));
    }

    #[test]
    fn test_retry_config_should_retry() {
        let config = RetryConfig::default();
        assert!(config.should_retry(&RetryableError::SerializationFailure, 0));
        assert!(config.should_retry(&RetryableError::SerializationFailure, 2));
        assert!(!config.should_retry(&RetryableError::SerializationFailure, 3));
    }

    #[test]
    fn test_retry_config_delay_exponential() {
        let config = RetryConfig {
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(2),
            ..Default::default()
        };

        assert_eq!(config.delay_for_attempt(0), Duration::from_millis(100));
        assert_eq!(config.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(config.delay_for_attempt(2), Duration::from_millis(400));
        // Capped at max_delay
        assert_eq!(config.delay_for_attempt(10), Duration::from_millis(2000));
    }

    #[test]
    fn test_with_retry_success() {
        let config = RetryConfig::default();
        let result = with_retry(&config, || -> Result<i32, String> { Ok(42) });
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_with_retry_eventual_success() {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(1),
            ..Default::default()
        };

        let attempts = Arc::new(Mutex::new(0));
        let attempts_clone = attempts.clone();

        let result = with_retry(&config, || -> Result<i32, String> {
            let mut a = attempts_clone.lock().unwrap();
            *a += 1;
            if *a < 3 {
                Err("not yet".into())
            } else {
                Ok(42)
            }
        });

        assert_eq!(result.unwrap(), 42);
        assert_eq!(*attempts.lock().unwrap(), 3);
    }

    #[test]
    fn test_transaction_log() {
        let mut log = TransactionLog::new();
        log.record(TransactionLogEntry {
            transaction_id: 1,
            operation: "INSERT".into(),
            timestamp: Instant::now(),
            duration: Duration::from_millis(5),
            success: true,
        });
        log.record(TransactionLogEntry {
            transaction_id: 2,
            operation: "UPDATE".into(),
            timestamp: Instant::now(),
            duration: Duration::from_millis(10),
            success: false,
        });

        assert_eq!(log.entries().len(), 2);
        assert_eq!(log.successful_count(), 1);
        assert_eq!(log.failed_count(), 1);
    }

    #[test]
    fn test_transaction_multiple_savepoints() {
        let mut tx = Transaction::new(1, IsolationLevel::ReadCommitted);
        tx.savepoint("sp1").unwrap();
        tx.savepoint("sp2").unwrap();
        tx.savepoint("sp3").unwrap();
        assert_eq!(tx.savepoint_count(), 3);

        tx.rollback_to_savepoint("sp2").unwrap();
        let sp2 = tx.savepoints.iter().find(|s| s.name == "sp2").unwrap();
        assert_eq!(sp2.state, TransactionState::RolledBack);
    }

    #[test]
    fn test_transaction_error_display() {
        let err = TransactionError::NotActive;
        assert!(!err.to_string().is_empty());

        let err = TransactionError::SavepointNotFound("sp1".into());
        assert!(err.to_string().contains("sp1"));

        let err = TransactionError::Timeout(Duration::from_secs(5));
        assert!(err.to_string().contains("5s"));
    }
}
