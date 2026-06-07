//! # Database Testing
//!
//! Testing database code requires special strategies to avoid polluting production
//! data and to ensure test isolation. This lesson covers test database setup,
//! fixtures, cleanup, and transactional testing.
//!
//! ## Key Concepts
//! - Test database provisioning
//! - Fixture management
//! - Transactional test isolation
//! - Test cleanup strategies
//! - Mock vs real database testing
//! - Snapshot testing for database state

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// 1. Test Database Configuration
// ---------------------------------------------------------------------------

/// Configuration for a test database.
#[derive(Debug, Clone)]
pub struct TestDbConfig {
    pub url: String,
    pub max_connections: u32,
    pub create_if_missing: bool,
    pub cleanup_on_drop: bool,
}

impl Default for TestDbConfig {
    fn default() -> Self {
        Self {
            url: ":memory:".into(),
            max_connections: 1,
            create_if_missing: true,
            cleanup_on_drop: true,
        }
    }
}

impl TestDbConfig {
    pub fn in_memory() -> Self {
        Self::default()
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }
}

// ---------------------------------------------------------------------------
// 2. Test Fixtures
// ---------------------------------------------------------------------------

/// A fixture that inserts test data and cleans it up after the test.
#[derive(Debug)]
pub struct Fixture {
    pub name: String,
    pub setup_sql: Vec<String>,
    pub teardown_sql: Vec<String>,
    pub data: Vec<HashMap<String, String>>,
}

impl Fixture {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            setup_sql: Vec::new(),
            teardown_sql: Vec::new(),
            data: Vec::new(),
        }
    }

    pub fn with_setup(mut self, sql: impl Into<String>) -> Self {
        self.setup_sql.push(sql.into());
        self
    }

    pub fn with_teardown(mut self, sql: impl Into<String>) -> Self {
        self.teardown_sql.push(sql.into());
        self
    }

    pub fn with_row(mut self, row: HashMap<String, String>) -> Self {
        self.data.push(row);
        self
    }

    /// Generate INSERT statements for the fixture data.
    pub fn insert_statements(&self, table: &str) -> Vec<String> {
        self.data
            .iter()
            .map(|row| {
                let cols: Vec<&str> = row.keys().map(|k| k.as_str()).collect();
                let vals: Vec<String> = row.values().map(|v| format!("'{v}'")).collect();
                format!(
                    "INSERT INTO {table} ({}) VALUES ({});",
                    cols.join(", "),
                    vals.join(", ")
                )
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// 3. Fixture Builder
// ---------------------------------------------------------------------------

/// Build fixtures for common test scenarios.
pub struct FixtureBuilder;

impl FixtureBuilder {
    /// Create a users fixture with sample data.
    pub fn users() -> Fixture {
        let mut fixture = Fixture::new("users")
            .with_setup("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, username TEXT, email TEXT, active BOOLEAN);")
            .with_teardown("DROP TABLE IF EXISTS users;");

        let mut row1 = HashMap::new();
        row1.insert("id".into(), "1".into());
        row1.insert("username".into(), "alice".into());
        row1.insert("email".into(), "alice@example.com".into());
        row1.insert("active".into(), "true".into());
        fixture = fixture.with_row(row1);

        let mut row2 = HashMap::new();
        row2.insert("id".into(), "2".into());
        row2.insert("username".into(), "bob".into());
        row2.insert("email".into(), "bob@example.com".into());
        row2.insert("active".into(), "true".into());
        fixture = fixture.with_row(row2);

        let mut row3 = HashMap::new();
        row3.insert("id".into(), "3".into());
        row3.insert("username".into(), "charlie".into());
        row3.insert("email".into(), "charlie@example.com".into());
        row3.insert("active".into(), "false".into());
        fixture = fixture.with_row(row3);

        fixture
    }

    /// Create a products fixture.
    pub fn products() -> Fixture {
        let mut fixture = Fixture::new("products")
            .with_setup("CREATE TABLE IF NOT EXISTS products (id INTEGER PRIMARY KEY, name TEXT, price INTEGER);")
            .with_teardown("DROP TABLE IF EXISTS products;");

        let mut row = HashMap::new();
        row.insert("id".into(), "1".into());
        row.insert("name".into(), "Widget".into());
        row.insert("price".into(), "999".into());
        fixture = fixture.with_row(row);

        fixture
    }
}

// ---------------------------------------------------------------------------
// 4. Test Database Manager
// ---------------------------------------------------------------------------

/// Manages test database lifecycle.
#[derive(Debug)]
pub struct TestDatabase {
    config: TestDbConfig,
    fixtures: Vec<Fixture>,
    setup_complete: bool,
}

impl TestDatabase {
    pub fn new(config: TestDbConfig) -> Self {
        Self {
            config,
            fixtures: Vec::new(),
            setup_complete: false,
        }
    }

    pub fn with_fixture(mut self, fixture: Fixture) -> Self {
        self.fixtures.push(fixture);
        self
    }

    /// Set up the test database with all fixtures.
    pub fn setup(&mut self) -> Result<(), TestDbError> {
        if self.setup_complete {
            return Ok(());
        }

        // Run setup SQL
        for fixture in &self.fixtures {
            for sql in &fixture.setup_sql {
                self.execute_sql(sql)?;
            }
            for table_sql in fixture.insert_statements(&fixture.name) {
                self.execute_sql(&table_sql)?;
            }
        }

        self.setup_complete = true;
        Ok(())
    }

    /// Tear down the test database.
    pub fn teardown(&mut self) -> Result<(), TestDbError> {
        for fixture in &self.fixtures {
            for sql in &fixture.teardown_sql {
                self.execute_sql(sql)?;
            }
        }
        self.setup_complete = false;
        Ok(())
    }

    fn execute_sql(&self, _sql: &str) -> Result<(), TestDbError> {
        // In a real implementation, this would execute against SQLite
        Ok(())
    }

    pub fn is_setup(&self) -> bool {
        self.setup_complete
    }

    pub fn fixture_count(&self) -> usize {
        self.fixtures.len()
    }
}

// ---------------------------------------------------------------------------
// 5. Transactional Test Runner
// ---------------------------------------------------------------------------

/// Runs tests within a transaction that is rolled back after each test,
/// ensuring complete isolation.
pub struct TransactionalTestRunner {
    transaction_active: bool,
    snapshot_id: Option<String>,
}

impl TransactionalTestRunner {
    pub fn new() -> Self {
        Self {
            transaction_active: false,
            snapshot_id: None,
        }
    }

    /// Begin a test transaction.
    pub fn begin(&mut self) -> Result<(), TestDbError> {
        if self.transaction_active {
            return Err(TestDbError::TransactionAlreadyActive);
        }
        self.transaction_active = true;
        self.snapshot_id = Some(format!("snap_{}", self.snapshot_id_count()));
        Ok(())
    }

    /// Rollback the test transaction (called after each test).
    pub fn rollback(&mut self) -> Result<(), TestDbError> {
        if !self.transaction_active {
            return Err(TestDbError::NoActiveTransaction);
        }
        self.transaction_active = false;
        self.snapshot_id = None;
        Ok(())
    }

    /// Run a test function within a transaction.
    pub fn run_test<F>(&mut self, test_fn: F) -> Result<(), TestDbError>
    where
        F: FnOnce() -> Result<(), String>,
    {
        self.begin()?;
        let result = test_fn();
        self.rollback()?;

        result.map_err(TestDbError::TestFailed)
    }

    pub fn is_active(&self) -> bool {
        self.transaction_active
    }

    fn snapshot_id_count(&self) -> u64 {
        // Simple counter for snapshot IDs
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}

// ---------------------------------------------------------------------------
// 6. Test Assertions
// /// Database-specific test assertions.
pub struct DbAssertions;

impl DbAssertions {
    /// Assert that a table has the expected number of rows.
    pub fn assert_row_count(table: &str, expected: usize, actual: usize) {
        assert_eq!(
            actual, expected,
            "table '{table}' expected {expected} rows, got {actual}"
        );
    }

    /// Assert that a query returns results.
    pub fn assert_has_results(table: &str, count: usize) {
        assert!(count > 0, "table '{table}' should have results");
    }

    /// Assert that two database states are equal.
    pub fn assert_state_equal(
        label: &str,
        expected: &HashMap<String, Vec<HashMap<String, String>>>,
        actual: &HashMap<String, Vec<HashMap<String, String>>>,
    ) {
        for (table, expected_rows) in expected {
            let actual_rows = actual.get(table);
            assert!(
                actual_rows.is_some(),
                "{label}: table '{table}' missing from actual state"
            );
            let actual_rows = actual_rows.unwrap();
            assert_eq!(
                actual_rows.len(),
                expected_rows.len(),
                "{label}: table '{table}' row count mismatch"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 7. Test Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum TestDbError {
    #[error("setup failed: {0}")]
    SetupFailed(String),

    #[error("teardown failed: {0}")]
    TeardownFailed(String),

    #[error("transaction already active")]
    TransactionAlreadyActive,

    #[error("no active transaction")]
    NoActiveTransaction,

    #[error("test failed: {0}")]
    TestFailed(String),
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_db_config_default() {
        let config = TestDbConfig::default();
        assert_eq!(config.url, ":memory:");
        assert!(config.cleanup_on_drop);
    }

    #[test]
    fn test_test_db_config_in_memory() {
        let config = TestDbConfig::in_memory();
        assert_eq!(config.url, ":memory:");
    }

    #[test]
    fn test_fixture_creation() {
        let fixture = Fixture::new("test")
            .with_setup("CREATE TABLE test (id INT);")
            .with_teardown("DROP TABLE test;");

        assert_eq!(fixture.name, "test");
        assert_eq!(fixture.setup_sql.len(), 1);
        assert_eq!(fixture.teardown_sql.len(), 1);
    }

    #[test]
    fn test_fixture_insert_statements() {
        let mut row = HashMap::new();
        row.insert("name".into(), "Alice".into());
        row.insert("email".into(), "alice@test.com".into());

        let fixture = Fixture::new("users").with_row(row);
        let stmts = fixture.insert_statements("users");

        assert_eq!(stmts.len(), 1);
        assert!(stmts[0].contains("INSERT INTO users"));
        assert!(stmts[0].contains("Alice"));
    }

    #[test]
    fn test_fixture_builder_users() {
        let fixture = FixtureBuilder::users();
        assert_eq!(fixture.name, "users");
        assert_eq!(fixture.data.len(), 3);
        assert!(!fixture.setup_sql.is_empty());
    }

    #[test]
    fn test_fixture_builder_products() {
        let fixture = FixtureBuilder::products();
        assert_eq!(fixture.name, "products");
        assert_eq!(fixture.data.len(), 1);
    }

    #[test]
    fn test_test_database_setup() {
        let config = TestDbConfig::in_memory();
        let fixture = Fixture::new("test")
            .with_setup("CREATE TABLE test (id INT);");

        let mut db = TestDatabase::new(config).with_fixture(fixture);
        assert!(!db.is_setup());

        db.setup().unwrap();
        assert!(db.is_setup());
        assert_eq!(db.fixture_count(), 1);
    }

    #[test]
    fn test_test_database_teardown() {
        let config = TestDbConfig::in_memory();
        let fixture = Fixture::new("test")
            .with_setup("CREATE TABLE test (id INT);")
            .with_teardown("DROP TABLE test;");

        let mut db = TestDatabase::new(config).with_fixture(fixture);
        db.setup().unwrap();
        db.teardown().unwrap();
        assert!(!db.is_setup());
    }

    #[test]
    fn test_test_database_idempotent_setup() {
        let config = TestDbConfig::in_memory();
        let mut db = TestDatabase::new(config);

        db.setup().unwrap();
        db.setup().unwrap(); // should not fail
    }

    #[test]
    fn test_transactional_runner() {
        let mut runner = TransactionalTestRunner::new();
        assert!(!runner.is_active());

        runner.begin().unwrap();
        assert!(runner.is_active());

        runner.rollback().unwrap();
        assert!(!runner.is_active());
    }

    #[test]
    fn test_transactional_runner_double_begin() {
        let mut runner = TransactionalTestRunner::new();
        runner.begin().unwrap();
        assert!(runner.begin().is_err());
    }

    #[test]
    fn test_transactional_runner_rollback_without_begin() {
        let mut runner = TransactionalTestRunner::new();
        assert!(runner.rollback().is_err());
    }

    #[test]
    fn test_transactional_runner_run_test() {
        let mut runner = TransactionalTestRunner::new();

        let result = runner.run_test(|| Ok(()));
        assert!(result.is_ok());
        assert!(!runner.is_active());
    }

    #[test]
    fn test_transactional_runner_run_test_failure() {
        let mut runner = TransactionalTestRunner::new();

        let result = runner.run_test(|| Err("test failed".into()));
        assert!(result.is_err());
        // Transaction should still be rolled back
        assert!(!runner.is_active());
    }

    #[test]
    fn test_db_assertions_row_count() {
        DbAssertions::assert_row_count("users", 3, 3);
    }

    #[test]
    fn test_db_assertions_has_results() {
        DbAssertions::assert_has_results("users", 5);
    }

    #[test]
    fn test_db_assertions_state_equal() {
        let mut expected = HashMap::new();
        expected.insert("users".into(), vec![HashMap::new()]);

        let mut actual = HashMap::new();
        actual.insert("users".into(), vec![HashMap::new()]);

        DbAssertions::assert_state_equal("test", &expected, &actual);
    }

    #[test]
    fn test_test_db_error_display() {
        let err = TestDbError::SetupFailed("table exists".into());
        assert!(err.to_string().contains("table exists"));

        let err = TestDbError::TransactionAlreadyActive;
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_fixture_with_multiple_rows() {
        let mut fixture = Fixture::new("items");

        for i in 0..5 {
            let mut row = HashMap::new();
            row.insert("id".into(), i.to_string());
            row.insert("name".into(), format!("item_{i}"));
            fixture = fixture.with_row(row);
        }

        let stmts = fixture.insert_statements("items");
        assert_eq!(stmts.len(), 5);
    }
}
