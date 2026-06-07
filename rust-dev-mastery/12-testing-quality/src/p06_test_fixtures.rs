//! # Test Fixtures
//!
//! Test fixtures provide reusable setup code for tests. This lesson covers
//! builder patterns, factory functions, and test data management.

use std::collections::HashMap;

/// A builder pattern for creating test data.
pub struct UserBuilder {
    id: Option<u64>,
    name: Option<String>,
    email: Option<String>,
    role: Option<String>,
    active: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestUser {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub role: String,
    pub active: bool,
}

impl UserBuilder {
    pub fn new() -> Self {
        UserBuilder {
            id: None,
            name: None,
            email: None,
            role: None,
            active: None,
        }
    }

    pub fn id(mut self, id: u64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn email(mut self, email: &str) -> Self {
        self.email = Some(email.to_string());
        self
    }

    pub fn role(mut self, role: &str) -> Self {
        self.role = Some(role.to_string());
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = Some(active);
        self
    }

    pub fn build(self) -> TestUser {
        TestUser {
            id: self.id.unwrap_or(1),
            name: self.name.unwrap_or_else(|| "Test User".to_string()),
            email: self.email.unwrap_or_else(|| "test@example.com".to_string()),
            role: self.role.unwrap_or_else(|| "user".to_string()),
            active: self.active.unwrap_or(true),
        }
    }
}

/// Factory functions for common test objects.
pub fn admin_user() -> TestUser {
    UserBuilder::new()
        .id(1)
        .name("Admin")
        .email("admin@example.com")
        .role("admin")
        .build()
}

pub fn regular_user() -> TestUser {
    UserBuilder::new()
        .id(2)
        .name("User")
        .email("user@example.com")
        .role("user")
        .build()
}

pub fn inactive_user() -> TestUser {
    UserBuilder::new()
        .id(3)
        .name("Inactive")
        .email("inactive@example.com")
        .active(false)
        .build()
}

/// A test database fixture that provides isolated test data.
pub struct TestDatabase {
    users: HashMap<u64, TestUser>,
    next_id: u64,
}

impl TestDatabase {
    pub fn new() -> Self {
        TestDatabase {
            users: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn with_users(users: Vec<TestUser>) -> Self {
        let mut db = TestDatabase::new();
        for user in users {
            db.users.insert(user.id, user);
        }
        db.next_id = db.users.keys().max().map_or(1, |k| k + 1);
        db
    }

    pub fn insert(&mut self, user: TestUser) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.users.insert(id, user);
        id
    }

    pub fn get(&self, id: u64) -> Option<&TestUser> {
        self.users.get(&id)
    }

    pub fn get_all(&self) -> Vec<&TestUser> {
        self.users.values().collect()
    }

    pub fn count(&self) -> usize {
        self.users.len()
    }
}

/// A fixture for testing HTTP-like request handling.
pub struct RequestFixture {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub query_params: HashMap<String, String>,
}

impl RequestFixture {
    pub fn get(path: &str) -> Self {
        RequestFixture {
            method: "GET".to_string(),
            path: path.to_string(),
            headers: HashMap::new(),
            body: None,
            query_params: HashMap::new(),
        }
    }

    pub fn post(path: &str, body: &str) -> Self {
        RequestFixture {
            method: "POST".to_string(),
            path: path.to_string(),
            headers: HashMap::new(),
            body: Some(body.to_string()),
            query_params: HashMap::new(),
        }
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_query(mut self, key: &str, value: &str) -> Self {
        self.query_params.insert(key.to_string(), value.to_string());
        self
    }
}

/// A scope guard that runs cleanup code when dropped.
pub struct TestScope {
    cleanup: Option<Box<dyn FnOnce()>>,
}

impl TestScope {
    pub fn new<F: FnOnce() + 'static>(cleanup: F) -> Self {
        TestScope {
            cleanup: Some(Box::new(cleanup)),
        }
    }

    pub fn disarm(mut self) {
        self.cleanup = None;
    }
}

impl Drop for TestScope {
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}

/// A test counter for tracking invocations.
pub struct CallCounter {
    count: std::cell::Cell<usize>,
}

impl CallCounter {
    pub fn new() -> Self {
        CallCounter {
            count: std::cell::Cell::new(0),
        }
    }

    pub fn increment(&self) {
        self.count.set(self.count.get() + 1);
    }

    pub fn count(&self) -> usize {
        self.count.get()
    }

    pub fn assert_called_once(&self) {
        assert_eq!(self.count(), 1, "expected to be called exactly once");
    }

    pub fn assert_called_times(&self, n: usize) {
        assert_eq!(self.count(), n, "expected to be called {n} times");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_builder_defaults() {
        let user = UserBuilder::new().build();
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "Test User");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.role, "user");
        assert!(user.active);
    }

    #[test]
    fn test_user_builder_custom() {
        let user = UserBuilder::new()
            .id(42)
            .name("Alice")
            .email("alice@test.com")
            .role("admin")
            .active(false)
            .build();
        assert_eq!(user.id, 42);
        assert_eq!(user.name, "Alice");
        assert!(!user.active);
    }

    #[test]
    fn test_factory_functions() {
        let admin = admin_user();
        assert_eq!(admin.role, "admin");

        let user = regular_user();
        assert_eq!(user.role, "user");

        let inactive = inactive_user();
        assert!(!inactive.active);
    }

    #[test]
    fn test_database_fixture() {
        let db = TestDatabase::with_users(vec![admin_user(), regular_user()]);
        assert_eq!(db.count(), 2);
        assert!(db.get(1).is_some());
        assert!(db.get(99).is_none());
    }

    #[test]
    fn test_database_insert() {
        let mut db = TestDatabase::new();
        let id = db.insert(admin_user());
        assert_eq!(id, 1);
        assert_eq!(db.count(), 1);
    }

    #[test]
    fn test_request_fixture() {
        let req = RequestFixture::get("/api/users")
            .with_header("Authorization", "Bearer token")
            .with_query("page", "1");
        assert_eq!(req.method, "GET");
        assert_eq!(req.path, "/api/users");
        assert_eq!(req.headers.get("Authorization").unwrap(), "Bearer token");
    }

    #[test]
    fn test_request_fixture_post() {
        let req = RequestFixture::post("/api/users", r#"{"name": "Alice"}"#);
        assert_eq!(req.method, "POST");
        assert!(req.body.unwrap().contains("Alice"));
    }

    #[test]
    fn test_call_counter() {
        let counter = CallCounter::new();
        counter.increment();
        counter.increment();
        counter.increment();
        counter.assert_called_times(3);
    }

    #[test]
    fn test_call_counter_once() {
        let counter = CallCounter::new();
        counter.increment();
        counter.assert_called_once();
    }
}
