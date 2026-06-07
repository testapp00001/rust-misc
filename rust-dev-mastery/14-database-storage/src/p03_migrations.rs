//! # Migrations
//!
//! Database migrations provide version-controlled, repeatable schema changes.
//! This lesson covers migration file formats, migration runners, and rollback
//! strategies.
//!
//! ## Key Concepts
//! - Migration file naming and ordering
//! - Up and down migrations
//! - Migration state tracking
//! - Transactional migrations
//! - Rollback strategies
//! - Migration validation

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// 1. Migration Definition
// ---------------------------------------------------------------------------

/// A single database migration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    pub version: u32,
    pub name: String,
    pub up_sql: String,
    pub down_sql: String,
    pub checksum: String,
}

impl Migration {
    pub fn new(version: u32, name: impl Into<String>, up_sql: &str, down_sql: &str) -> Self {
        Self {
            version,
            name: name.into(),
            up_sql: up_sql.into(),
            down_sql: down_sql.into(),
            checksum: compute_checksum(up_sql),
        }
    }

    pub fn display_name(&self) -> String {
        format!("{:04}_{}", self.version, self.name)
    }
}

/// Simple checksum computation for migration integrity.
fn compute_checksum(sql: &str) -> String {
    let mut hash: u64 = 5381;
    for byte in sql.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    format!("{hash:016x}")
}

// ---------------------------------------------------------------------------
// 2. Migration Registry
// ---------------------------------------------------------------------------

/// A collection of migrations in order.
#[derive(Debug)]
pub struct MigrationRegistry {
    migrations: BTreeMap<u32, Migration>,
}

impl MigrationRegistry {
    pub fn new() -> Self {
        Self {
            migrations: BTreeMap::new(),
        }
    }

    /// Add a migration to the registry.
    pub fn add(&mut self, migration: Migration) -> Result<(), MigrationError> {
        if self.migrations.contains_key(&migration.version) {
            return Err(MigrationError::DuplicateVersion(migration.version));
        }
        self.migrations
            .insert(migration.version, migration);
        Ok(())
    }

    /// Get all migrations in version order.
    pub fn all(&self) -> Vec<&Migration> {
        self.migrations.values().collect()
    }

    /// Get migrations that need to be applied (version > last_applied).
    pub fn pending(&self, last_applied: u32) -> Vec<&Migration> {
        self.migrations
            .values()
            .filter(|m| m.version > last_applied)
            .collect()
    }

    /// Get a specific migration by version.
    pub fn get(&self, version: u32) -> Option<&Migration> {
        self.migrations.get(&version)
    }

    /// Get the latest version in the registry.
    pub fn latest_version(&self) -> Option<u32> {
        self.migrations.keys().next_back().copied()
    }

    /// Validate all migrations have unique versions and valid checksums.
    pub fn validate(&self) -> Result<(), MigrationError> {
        for migration in self.migrations.values() {
            let expected = compute_checksum(&migration.up_sql);
            if migration.checksum != expected {
                return Err(MigrationError::ChecksumMismatch {
                    version: migration.version,
                    expected,
                    actual: migration.checksum.clone(),
                });
            }
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.migrations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.migrations.is_empty()
    }
}

// ---------------------------------------------------------------------------
// 3. Migration State
// ---------------------------------------------------------------------------

/// Tracks which migrations have been applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationState {
    pub applied: Vec<AppliedMigration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedMigration {
    pub version: u32,
    pub name: String,
    pub checksum: String,
    pub applied_at: String,
    pub execution_time_ms: u64,
}

impl MigrationState {
    pub fn new() -> Self {
        Self {
            applied: Vec::new(),
        }
    }

    pub fn last_applied_version(&self) -> u32 {
        self.applied.iter().map(|m| m.version).max().unwrap_or(0)
    }

    pub fn is_applied(&self, version: u32) -> bool {
        self.applied.iter().any(|m| m.version == version)
    }

    pub fn record(&mut self, migration: &AppliedMigration) {
        if !self.is_applied(migration.version) {
            self.applied.push(migration.clone());
            self.applied.sort_by_key(|m| m.version);
        }
    }

    pub fn remove(&mut self, version: u32) {
        self.applied.retain(|m| m.version != version);
    }
}

// ---------------------------------------------------------------------------
// 4. Migration Runner
// ---------------------------------------------------------------------------

/// Runs migrations and tracks their execution.
pub struct MigrationRunner {
    registry: MigrationRegistry,
    state: MigrationState,
    dry_run: bool,
    log: Vec<String>,
}

impl MigrationRunner {
    pub fn new(registry: MigrationRegistry, state: MigrationState) -> Self {
        Self {
            registry,
            state,
            dry_run: false,
            log: Vec::new(),
        }
    }

    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Run all pending migrations.
    pub fn migrate_up(&mut self) -> Result<Vec<u32>, MigrationError> {
        self.registry.validate()?;

        let last = self.state.last_applied_version();
        let pending = self.registry.pending(last);

        if pending.is_empty() {
            self.log.push("no pending migrations".into());
            return Ok(vec![]);
        }

        let mut applied = Vec::new();
        for migration in pending {
            self.log
                .push(format!("applying migration {}: {}", migration.version, migration.name));

            if !self.dry_run {
                let record = AppliedMigration {
                    version: migration.version,
                    name: migration.name.clone(),
                    checksum: migration.checksum.clone(),
                    applied_at: "2024-01-01T00:00:00Z".into(),
                    execution_time_ms: 0,
                };
                self.state.record(&record);
            }

            applied.push(migration.version);
        }

        Ok(applied)
    }

    /// Rollback the last N migrations.
    pub fn migrate_down(&mut self, count: usize) -> Result<Vec<u32>, MigrationError> {
        let mut to_rollback: Vec<u32> = self
            .state
            .applied
            .iter()
            .rev()
            .take(count)
            .map(|m| m.version)
            .collect();

        if to_rollback.is_empty() {
            self.log.push("no migrations to rollback".into());
            return Ok(vec![]);
        }

        let mut rolled_back = Vec::new();
        for version in &to_rollback {
            let migration = self
                .registry
                .get(*version)
                .ok_or(MigrationError::NotFound(*version))?;

            self.log.push(format!(
                "rolling back migration {}: {}",
                migration.version, migration.name
            ));

            if !self.dry_run {
                self.state.remove(*version);
            }

            rolled_back.push(*version);
        }

        Ok(rolled_back)
    }

    /// Rollback to a specific version (inclusive).
    pub fn migrate_to(&mut self, target_version: u32) -> Result<Vec<u32>, MigrationError> {
        let current = self.state.last_applied_version();
        if target_version >= current {
            return Ok(vec![]);
        }

        let count = self
            .state
            .applied
            .iter()
            .filter(|m| m.version > target_version)
            .count();

        self.migrate_down(count)
    }

    pub fn state(&self) -> &MigrationState {
        &self.state
    }

    pub fn log(&self) -> &[String] {
        &self.log
    }
}

// ---------------------------------------------------------------------------
// 5. Migration Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("duplicate migration version: {0}")]
    DuplicateVersion(u32),

    #[error("migration {0} not found")]
    NotFound(u32),

    #[error("checksum mismatch for migration {version}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        version: u32,
        expected: String,
        actual: String,
    },

    #[error("migration {version} failed: {reason}")]
    ExecutionFailed { version: u32, reason: String },

    #[error("cannot rollback: {0}")]
    RollbackFailed(String),
}

// ---------------------------------------------------------------------------
// 6. Migration Generator
// ---------------------------------------------------------------------------

/// Generates migration file content.
pub struct MigrationGenerator;

impl MigrationGenerator {
    /// Generate a CREATE TABLE migration.
    pub fn create_table(table: &str, columns: &[(String, String)]) -> (String, String) {
        let cols: Vec<String> = columns
            .iter()
            .map(|(name, dtype)| format!("    {name} {dtype}"))
            .collect();

        let up = format!(
            "CREATE TABLE {table} (\n{},\n    created_at TIMESTAMP DEFAULT NOW(),\n    updated_at TIMESTAMP DEFAULT NOW()\n);",
            cols.join(",\n")
        );
        let down = format!("DROP TABLE IF EXISTS {table};");

        (up, down)
    }

    /// Generate an ADD COLUMN migration.
    pub fn add_column(table: &str, column: &str, dtype: &str, default: Option<&str>) -> (String, String) {
        let default_clause = default
            .map(|d| format!(" DEFAULT {d}"))
            .unwrap_or_default();

        let up = format!("ALTER TABLE {table} ADD COLUMN {column} {dtype}{default_clause};");
        let down = format!("ALTER TABLE {table} DROP COLUMN IF EXISTS {column};");

        (up, down)
    }

    /// Generate an ADD INDEX migration.
    pub fn add_index(table: &str, columns: &[&str], unique: bool) -> (String, String) {
        let unique_str = if unique { "UNIQUE " } else { "" };
        let col_str = columns.join("_");
        let col_list = columns.join(", ");
        let index_name = format!("idx_{table}_{col_str}");

        let up = format!("CREATE {unique_str}INDEX {index_name} ON {table} ({col_list});");
        let down = format!("DROP INDEX IF EXISTS {index_name};");

        (up, down)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_creation() {
        let m = Migration::new(1, "create_users", "CREATE TABLE users();", "DROP TABLE users;");
        assert_eq!(m.version, 1);
        assert_eq!(m.name, "create_users");
        assert_eq!(m.display_name(), "0001_create_users");
        assert!(!m.checksum.is_empty());
    }

    #[test]
    fn test_migration_registry() {
        let mut registry = MigrationRegistry::new();
        registry
            .add(Migration::new(1, "m1", "SQL1", "ROLLBACK1"))
            .unwrap();
        registry
            .add(Migration::new(2, "m2", "SQL2", "ROLLBACK2"))
            .unwrap();

        assert_eq!(registry.len(), 2);
        assert_eq!(registry.latest_version(), Some(2));
    }

    #[test]
    fn test_migration_registry_duplicate() {
        let mut registry = MigrationRegistry::new();
        registry
            .add(Migration::new(1, "m1", "SQL", "ROLLBACK"))
            .unwrap();
        let result = registry.add(Migration::new(1, "m1_dup", "SQL2", "ROLLBACK2"));
        assert!(result.is_err());
    }

    #[test]
    fn test_migration_pending() {
        let mut registry = MigrationRegistry::new();
        registry
            .add(Migration::new(1, "m1", "SQL1", "ROLL1"))
            .unwrap();
        registry
            .add(Migration::new(2, "m2", "SQL2", "ROLL2"))
            .unwrap();
        registry
            .add(Migration::new(3, "m3", "SQL3", "ROLL3"))
            .unwrap();

        let pending = registry.pending(1);
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0].version, 2);
        assert_eq!(pending[1].version, 3);
    }

    #[test]
    fn test_migration_state() {
        let mut state = MigrationState::new();
        assert_eq!(state.last_applied_version(), 0);

        state.record(&AppliedMigration {
            version: 1,
            name: "m1".into(),
            checksum: "abc".into(),
            applied_at: "now".into(),
            execution_time_ms: 10,
        });

        assert_eq!(state.last_applied_version(), 1);
        assert!(state.is_applied(1));
        assert!(!state.is_applied(2));
    }

    #[test]
    fn test_migration_state_remove() {
        let mut state = MigrationState::new();
        state.record(&AppliedMigration {
            version: 1,
            name: "m1".into(),
            checksum: "abc".into(),
            applied_at: "now".into(),
            execution_time_ms: 10,
        });
        state.record(&AppliedMigration {
            version: 2,
            name: "m2".into(),
            checksum: "def".into(),
            applied_at: "now".into(),
            execution_time_ms: 10,
        });

        state.remove(2);
        assert_eq!(state.last_applied_version(), 1);
        assert!(!state.is_applied(2));
    }

    #[test]
    fn test_migration_runner_up() {
        let mut registry = MigrationRegistry::new();
        registry
            .add(Migration::new(1, "m1", "SQL1", "ROLL1"))
            .unwrap();
        registry
            .add(Migration::new(2, "m2", "SQL2", "ROLL2"))
            .unwrap();

        let state = MigrationState::new();
        let mut runner = MigrationRunner::new(registry, state);
        let applied = runner.migrate_up().unwrap();

        assert_eq!(applied, vec![1, 2]);
        assert_eq!(runner.state().last_applied_version(), 2);
    }

    #[test]
    fn test_migration_runner_down() {
        let mut registry = MigrationRegistry::new();
        registry
            .add(Migration::new(1, "m1", "SQL1", "ROLL1"))
            .unwrap();
        registry
            .add(Migration::new(2, "m2", "SQL2", "ROLL2"))
            .unwrap();

        let state = MigrationState::new();
        let mut runner = MigrationRunner::new(registry, state);
        runner.migrate_up().unwrap();

        let rolled_back = runner.migrate_down(1).unwrap();
        assert_eq!(rolled_back, vec![2]);
        assert_eq!(runner.state().last_applied_version(), 1);
    }

    #[test]
    fn test_migration_runner_to_version() {
        let mut registry = MigrationRegistry::new();
        for i in 1..=5 {
            registry
                .add(Migration::new(i, format!("m{i}"), &format!("SQL{i}"), &format!("ROLL{i}")))
                .unwrap();
        }

        let state = MigrationState::new();
        let mut runner = MigrationRunner::new(registry, state);
        runner.migrate_up().unwrap();
        assert_eq!(runner.state().last_applied_version(), 5);

        runner.migrate_to(2).unwrap();
        assert_eq!(runner.state().last_applied_version(), 2);
    }

    #[test]
    fn test_migration_runner_dry_run() {
        let mut registry = MigrationRegistry::new();
        registry
            .add(Migration::new(1, "m1", "SQL1", "ROLL1"))
            .unwrap();

        let state = MigrationState::new();
        let mut runner = MigrationRunner::new(registry, state).with_dry_run(true);
        let applied = runner.migrate_up().unwrap();

        assert_eq!(applied, vec![1]);
        // State should NOT have been updated in dry run
        assert_eq!(runner.state().last_applied_version(), 0);
    }

    #[test]
    fn test_migration_generator_create_table() {
        let columns = vec![
            ("name".into(), "VARCHAR(255) NOT NULL".into()),
            ("email".into(), "VARCHAR(255) UNIQUE".into()),
        ];
        let (up, down) = MigrationGenerator::create_table("users", &columns);

        assert!(up.contains("CREATE TABLE users"));
        assert!(up.contains("name VARCHAR(255) NOT NULL"));
        assert!(up.contains("email VARCHAR(255) UNIQUE"));
        assert!(up.contains("created_at"));
        assert!(down.contains("DROP TABLE IF EXISTS users"));
    }

    #[test]
    fn test_migration_generator_add_column() {
        let (up, down) =
            MigrationGenerator::add_column("users", "phone", "VARCHAR(20)", Some("NULL"));
        assert!(up.contains("ALTER TABLE users ADD COLUMN phone VARCHAR(20) DEFAULT NULL"));
        assert!(down.contains("ALTER TABLE users DROP COLUMN IF EXISTS phone"));
    }

    #[test]
    fn test_migration_generator_add_index() {
        let (up, down) = MigrationGenerator::add_index("users", &["email"], true);
        assert!(up.contains("CREATE UNIQUE INDEX"));
        assert!(up.contains("idx_users_email"));
        assert!(down.contains("DROP INDEX IF EXISTS"));
    }

    #[test]
    fn test_migration_registry_validation() {
        let mut registry = MigrationRegistry::new();
        let m = Migration::new(1, "test", "CREATE TABLE test();", "DROP TABLE test;");
        registry.add(m).unwrap();
        assert!(registry.validate().is_ok());
    }

    #[test]
    fn test_checksum_consistency() {
        let m1 = Migration::new(1, "test", "SQL", "ROLL");
        let m2 = Migration::new(1, "test", "SQL", "ROLL");
        assert_eq!(m1.checksum, m2.checksum);
    }
}
