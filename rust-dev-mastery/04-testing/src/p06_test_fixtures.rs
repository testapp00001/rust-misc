//! # Lesson 6: Test Fixtures
//!
//! Test fixtures provide consistent, reusable test environments.
//! This lesson covers setup/teardown patterns, temporary directories,
//! test data builders, and fixture composition.

use std::collections::HashMap;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Test data builders
// ---------------------------------------------------------------------------

/// A builder for creating test User objects.
pub struct UserBuilder {
    id: String,
    name: String,
    email: String,
    role: String,
    active: bool,
    metadata: HashMap<String, String>,
}

impl UserBuilder {
    pub fn new() -> Self {
        Self {
            id: "user-1".into(),
            name: "Test User".into(),
            email: "test@example.com".into(),
            role: "user".into(),
            active: true,
            metadata: HashMap::new(),
        }
    }

    pub fn id(mut self, id: &str) -> Self {
        self.id = id.into();
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = name.into();
        self
    }

    pub fn email(mut self, email: &str) -> Self {
        self.email = email.into();
        self
    }

    pub fn role(mut self, role: &str) -> Self {
        self.role = role.into();
        self
    }

    pub fn inactive(mut self) -> Self {
        self.active = false;
        self
    }

    pub fn metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> TestUser {
        TestUser {
            id: self.id,
            name: self.name,
            email: self.email,
            role: self.role,
            active: self.active,
            metadata: self.metadata,
        }
    }
}

impl Default for UserBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub active: bool,
    pub metadata: HashMap<String, String>,
}

/// A builder for creating test configuration objects.
pub struct ConfigBuilder {
    database_url: String,
    port: u16,
    max_connections: u32,
    log_level: String,
    features: Vec<String>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            database_url: "sqlite://:memory:".into(),
            port: 8080,
            max_connections: 10,
            log_level: "debug".into(),
            features: Vec::new(),
        }
    }

    pub fn database_url(mut self, url: &str) -> Self {
        self.database_url = url.into();
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    pub fn log_level(mut self, level: &str) -> Self {
        self.log_level = level.into();
        self
    }

    pub fn with_feature(mut self, feature: &str) -> Self {
        self.features.push(feature.into());
        self
    }

    pub fn build(self) -> TestConfig {
        TestConfig {
            database_url: self.database_url,
            port: self.port,
            max_connections: self.max_connections,
            log_level: self.log_level,
            features: self.features,
        }
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct TestConfig {
    pub database_url: String,
    pub port: u16,
    pub max_connections: u32,
    pub log_level: String,
    pub features: Vec<String>,
}

// ---------------------------------------------------------------------------
// Fixture for file-based tests
// ---------------------------------------------------------------------------

/// A fixture that manages temporary files for tests.
pub struct FileFixture {
    temp_dir: tempfile::TempDir,
    files: HashMap<String, PathBuf>,
}

impl FileFixture {
    pub fn new() -> Self {
        Self {
            temp_dir: tempfile::tempdir().expect("failed to create temp dir"),
            files: HashMap::new(),
        }
    }

    pub fn path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    /// Create a file with content and track it.
    pub fn create_file(&mut self, name: &str, content: &str) -> PathBuf {
        let path = self.path().join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&path, content).expect("failed to write file");
        self.files.insert(name.to_string(), path.clone());
        path
    }

    /// Read a tracked file's content.
    pub fn read_file(&self, name: &str) -> Option<String> {
        self.files
            .get(name)
            .and_then(|path| std::fs::read_to_string(path).ok())
    }

    /// Check if a tracked file exists.
    pub fn file_exists(&self, name: &str) -> bool {
        self.files.get(name).map_or(false, |p| p.exists())
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}

impl Default for FileFixture {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Fixture for in-memory store tests
// ---------------------------------------------------------------------------

/// A fixture that manages an in-memory key-value store.
pub struct StoreFixture {
    store: HashMap<String, String>,
    access_log: Vec<AccessLogEntry>,
}

#[derive(Debug, Clone)]
pub struct AccessLogEntry {
    pub operation: String,
    pub key: String,
    pub value: Option<String>,
}

impl StoreFixture {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            access_log: Vec::new(),
        }
    }

    /// Pre-populate the store with test data.
    pub fn with_data(data: &[(&str, &str)]) -> Self {
        let mut fixture = Self::new();
        for (k, v) in data {
            fixture.set(k, v);
        }
        fixture.access_log.clear(); // Don't count setup
        fixture
    }

    pub fn get(&mut self, key: &str) -> Option<&str> {
        self.access_log.push(AccessLogEntry {
            operation: "get".into(),
            key: key.into(),
            value: self.store.get(key).cloned(),
        });
        self.store.get(key).map(|s| s.as_str())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.access_log.push(AccessLogEntry {
            operation: "set".into(),
            key: key.into(),
            value: Some(value.into()),
        });
        self.store.insert(key.into(), value.into());
    }

    pub fn delete(&mut self, key: &str) -> bool {
        let existed = self.store.remove(key).is_some();
        self.access_log.push(AccessLogEntry {
            operation: "delete".into(),
            key: key.into(),
            value: None,
        });
        existed
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn access_log(&self) -> &[AccessLogEntry] {
        &self.access_log
    }

    pub fn access_count(&self, operation: &str) -> usize {
        self.access_log
            .iter()
            .filter(|e| e.operation == operation)
            .count()
    }
}

impl Default for StoreFixture {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Composed fixture
// ---------------------------------------------------------------------------

/// A composed fixture that combines multiple fixtures.
pub struct AppFixture {
    pub files: FileFixture,
    pub store: StoreFixture,
    pub config: TestConfig,
}

impl AppFixture {
    pub fn new() -> Self {
        Self {
            files: FileFixture::new(),
            store: StoreFixture::new(),
            config: ConfigBuilder::new().build(),
        }
    }

    pub fn with_config(config: TestConfig) -> Self {
        Self {
            files: FileFixture::new(),
            store: StoreFixture::new(),
            config,
        }
    }
}

impl Default for AppFixture {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // UserBuilder tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_user_builder_default() {
        let user = UserBuilder::new().build();
        assert_eq!(user.id, "user-1");
        assert_eq!(user.name, "Test User");
        assert!(user.active);
    }

    #[test]
    fn test_user_builder_custom() {
        let user = UserBuilder::new()
            .id("user-42")
            .name("Alice")
            .email("alice@example.com")
            .role("admin")
            .metadata("department", "engineering")
            .build();

        assert_eq!(user.id, "user-42");
        assert_eq!(user.name, "Alice");
        assert_eq!(user.role, "admin");
        assert_eq!(user.metadata.get("department").unwrap(), "engineering");
    }

    #[test]
    fn test_user_builder_inactive() {
        let user = UserBuilder::new().inactive().build();
        assert!(!user.active);
    }

    // -----------------------------------------------------------------------
    // ConfigBuilder tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_config_builder_default() {
        let config = ConfigBuilder::new().build();
        assert_eq!(config.port, 8080);
        assert_eq!(config.max_connections, 10);
        assert!(config.features.is_empty());
    }

    #[test]
    fn test_config_builder_custom() {
        let config = ConfigBuilder::new()
            .port(3000)
            .max_connections(100)
            .log_level("info")
            .with_feature("auth")
            .with_feature("cache")
            .build();

        assert_eq!(config.port, 3000);
        assert_eq!(config.features.len(), 2);
    }

    // -----------------------------------------------------------------------
    // FileFixture tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_file_fixture_create_and_read() {
        let mut fixture = FileFixture::new();
        fixture.create_file("test.txt", "hello world");

        let content = fixture.read_file("test.txt").unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_file_fixture_exists() {
        let mut fixture = FileFixture::new();
        fixture.create_file("test.txt", "content");

        assert!(fixture.file_exists("test.txt"));
        assert!(!fixture.file_exists("missing.txt"));
    }

    #[test]
    fn test_file_fixture_count() {
        let mut fixture = FileFixture::new();
        assert_eq!(fixture.file_count(), 0);

        fixture.create_file("a.txt", "a");
        fixture.create_file("b.txt", "b");
        assert_eq!(fixture.file_count(), 2);
    }

    #[test]
    fn test_file_fixture_read_missing() {
        let fixture = FileFixture::new();
        assert!(fixture.read_file("missing.txt").is_none());
    }

    // -----------------------------------------------------------------------
    // StoreFixture tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_store_fixture_basic() {
        let mut store = StoreFixture::new();
        store.set("key", "value");
        assert_eq!(store.get("key"), Some("value"));
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_store_fixture_with_data() {
        let store = StoreFixture::with_data(&[("a", "1"), ("b", "2")]);
        assert_eq!(store.len(), 2);
        // Access log should be empty (setup doesn't count)
        assert!(store.access_log().is_empty());
    }

    #[test]
    fn test_store_fixture_access_log() {
        let mut store = StoreFixture::new();
        store.set("a", "1");
        store.get("a");
        store.get("b");
        store.delete("a");

        assert_eq!(store.access_log().len(), 4);
        assert_eq!(store.access_count("get"), 2);
        assert_eq!(store.access_count("set"), 1);
        assert_eq!(store.access_count("delete"), 1);
    }

    #[test]
    fn test_store_fixture_delete() {
        let mut store = StoreFixture::with_data(&[("key", "value")]);
        assert!(store.delete("key"));
        assert!(!store.delete("key"));
        assert_eq!(store.len(), 0);
    }

    // -----------------------------------------------------------------------
    // Composed fixture tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_app_fixture() {
        let mut app = AppFixture::new();

        app.files.create_file("config.json", "{}");
        app.store.set("key", "value");

        assert_eq!(app.files.file_count(), 1);
        assert_eq!(app.store.len(), 1);
        assert_eq!(app.config.port, 8080);
    }

    #[test]
    fn test_app_fixture_with_config() {
        let config = ConfigBuilder::new()
            .port(3000)
            .with_feature("test")
            .build();
        let app = AppFixture::with_config(config);
        assert_eq!(app.config.port, 3000);
        assert!(app.config.features.contains(&"test".to_string()));
    }

    // -----------------------------------------------------------------------
    // Builder pattern usage in tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_multiple_users() {
        let users: Vec<TestUser> = (1..=5)
            .map(|i| {
                UserBuilder::new()
                    .id(&format!("user-{}", i))
                    .name(&format!("User {}", i))
                    .email(&format!("user{}@example.com", i))
                    .build()
            })
            .collect();

        assert_eq!(users.len(), 5);
        assert_eq!(users[0].id, "user-1");
        assert_eq!(users[4].id, "user-5");
    }

    #[test]
    fn test_fixture_reuse() {
        let mut fixture = FileFixture::new();
        // First test
        fixture.create_file("test1.txt", "first");
        assert_eq!(fixture.read_file("test1.txt").unwrap(), "first");

        // Second test (same fixture)
        fixture.create_file("test2.txt", "second");
        assert_eq!(fixture.file_count(), 2);
    }
}
