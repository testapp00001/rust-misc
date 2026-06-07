//! # Mocking Patterns
//!
//! Rust's type system enables powerful mocking through trait objects and
//! dependency injection. This lesson covers patterns for testing code
//! that depends on external services.

use std::cell::RefCell;
use std::collections::HashMap;

/// Demonstrates trait-based mocking for a database.
pub trait Database {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: &str, value: &str);
    fn delete(&mut self, key: &str) -> bool;
    fn exists(&self, key: &str) -> bool;
}

/// Real implementation (not used in tests).
pub struct RealDatabase {
    connection_string: String,
}

impl RealDatabase {
    pub fn new(connection_string: &str) -> Self {
        RealDatabase {
            connection_string: connection_string.to_string(),
        }
    }
}

impl Database for RealDatabase {
    fn get(&self, _key: &str) -> Option<String> {
        // Would connect to real database
        unimplemented!("Real database not available in tests")
    }
    fn set(&mut self, _key: &str, _value: &str) {
        unimplemented!()
    }
    fn delete(&mut self, _key: &str) -> bool {
        unimplemented!()
    }
    fn exists(&self, _key: &str) -> bool {
        unimplemented!()
    }
}

/// Mock implementation for testing.
pub struct MockDatabase {
    data: HashMap<String, String>,
    /// Track method calls for verification
    calls: RefCell<Vec<String>>,
}

impl MockDatabase {
    pub fn new() -> Self {
        MockDatabase {
            data: HashMap::new(),
            calls: RefCell::new(Vec::new()),
        }
    }

    pub fn with_data(entries: Vec<(&str, &str)>) -> Self {
        let mut db = MockDatabase::new();
        for (key, value) in entries {
            db.data.insert(key.to_string(), value.to_string());
        }
        db
    }

    pub fn get_calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }

    pub fn was_called(&self, method: &str) -> bool {
        self.calls.borrow().iter().any(|c| c.contains(method))
    }
}

impl Database for MockDatabase {
    fn get(&self, key: &str) -> Option<String> {
        self.calls.borrow_mut().push(format!("get({key})"));
        self.data.get(key).cloned()
    }

    fn set(&mut self, key: &str, value: &str) {
        self.calls.borrow_mut().push(format!("set({key}, {value})"));
        self.data.insert(key.to_string(), value.to_string());
    }

    fn delete(&mut self, key: &str) -> bool {
        self.calls
            .borrow_mut()
            .push(format!("delete({key})"));
        self.data.remove(key).is_some()
    }

    fn exists(&self, key: &str) -> bool {
        self.calls
            .borrow_mut()
            .push(format!("exists({key})"));
        self.data.contains_key(key)
    }
}

/// A service that depends on a database (for demonstrating DI).
pub struct UserService {
    db: Box<dyn Database>,
}

impl UserService {
    pub fn new(db: Box<dyn Database>) -> Self {
        UserService { db }
    }

    pub fn get_user(&self, id: &str) -> Option<User> {
        let data = self.db.get(&format!("user:{id}"))?;
        Some(User {
            id: id.to_string(),
            name: data,
        })
    }

    pub fn create_user(&mut self, id: &str, name: &str) {
        self.db.set(&format!("user:{id}"), name);
    }

    pub fn delete_user(&mut self, id: &str) -> bool {
        self.db.delete(&format!("user:{id}"))
    }

    pub fn user_exists(&self, id: &str) -> bool {
        self.db.exists(&format!("user:{id}"))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: String,
    pub name: String,
}

/// Demonstrates a mock HTTP client.
pub trait HttpClient {
    fn get(&self, url: &str) -> Result<String, HttpError>;
    fn post(&self, url: &str, body: &str) -> Result<String, HttpError>;
}

#[derive(Debug, Clone)]
pub enum HttpError {
    ConnectionError,
    Timeout,
    Status(u16),
}

pub struct MockHttpClient {
    responses: RefCell<HashMap<String, Result<String, HttpError>>>,
    calls: RefCell<Vec<(String, Option<String>)>>,
}

impl MockHttpClient {
    pub fn new() -> Self {
        MockHttpClient {
            responses: RefCell::new(HashMap::new()),
            calls: RefCell::new(Vec::new()),
        }
    }

    pub fn mock_get(&self, url: &str, response: Result<String, HttpError>) {
        self.responses
            .borrow_mut()
            .insert(format!("GET:{url}"), response);
    }

    pub fn mock_post(&self, url: &str, response: Result<String, HttpError>) {
        self.responses
            .borrow_mut()
            .insert(format!("POST:{url}"), response);
    }

    pub fn get_calls(&self) -> Vec<(String, Option<String>)> {
        self.calls.borrow().clone()
    }
}

impl HttpClient for MockHttpClient {
    fn get(&self, url: &str) -> Result<String, HttpError> {
        self.calls
            .borrow_mut()
            .push((url.to_string(), None));
        self.responses
            .borrow()
            .get(&format!("GET:{url}"))
            .cloned()
            .unwrap_or(Err(HttpError::ConnectionError))
    }

    fn post(&self, url: &str, body: &str) -> Result<String, HttpError> {
        self.calls
            .borrow_mut()
            .push((url.to_string(), Some(body.to_string())));
        self.responses
            .borrow()
            .get(&format!("POST:{url}"))
            .cloned()
            .unwrap_or(Err(HttpError::ConnectionError))
    }
}

/// A service that uses the HTTP client.
pub struct ApiClient {
    client: Box<dyn HttpClient>,
    base_url: String,
}

impl ApiClient {
    pub fn new(client: Box<dyn HttpClient>, base_url: &str) -> Self {
        ApiClient {
            client,
            base_url: base_url.to_string(),
        }
    }

    pub fn fetch_data(&self, endpoint: &str) -> Result<String, HttpError> {
        let url = format!("{}/{endpoint}", self.base_url);
        self.client.get(&url)
    }

    pub fn send_data(&self, endpoint: &str, data: &str) -> Result<String, HttpError> {
        let url = format!("{}/{endpoint}", self.base_url);
        self.client.post(&url, data)
    }
}

/// Demonstrates recording and replaying calls.
pub struct CallRecorder<T> {
    inner: T,
    calls: RefCell<Vec<String>>,
}

impl<T> CallRecorder<T> {
    pub fn new(inner: T) -> Self {
        CallRecorder {
            inner,
            calls: RefCell::new(Vec::new()),
        }
    }

    pub fn calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }

    pub fn call_count(&self) -> usize {
        self.calls.borrow().len()
    }

    pub fn record(&self, method: &str) {
        self.calls.borrow_mut().push(method.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_database_get() {
        let db = MockDatabase::with_data(vec![("key1", "value1")]);
        assert_eq!(db.get("key1"), Some("value1".to_string()));
        assert_eq!(db.get("nonexistent"), None);
        assert!(db.was_called("get"));
    }

    #[test]
    fn test_mock_database_set() {
        let mut db = MockDatabase::new();
        db.set("key", "value");
        assert_eq!(db.get("key"), Some("value".to_string()));
    }

    #[test]
    fn test_mock_database_delete() {
        let mut db = MockDatabase::with_data(vec![("key", "value")]);
        assert!(db.delete("key"));
        assert!(!db.delete("nonexistent"));
    }

    #[test]
    fn test_user_service_with_mock() {
        let db = MockDatabase::with_data(vec![("user:1", "Alice")]);
        let service = UserService::new(Box::new(db));

        let user = service.get_user("1").unwrap();
        assert_eq!(user.name, "Alice");
        assert!(service.user_exists("1"));
        assert!(!service.user_exists("999"));
    }

    #[test]
    fn test_user_service_create() {
        let db = MockDatabase::new();
        let mut service = UserService::new(Box::new(db));

        service.create_user("1", "Bob");
        let user = service.get_user("1").unwrap();
        assert_eq!(user.name, "Bob");
    }

    #[test]
    fn test_mock_http_client() {
        let client = MockHttpClient::new();
        client.mock_get(
            "https://api.example.com/data",
            Ok(r#"{"status": "ok"}"#.to_string()),
        );

        let api = ApiClient::new(Box::new(client), "https://api.example.com");
        let result = api.fetch_data("data").unwrap();
        assert!(result.contains("ok"));
    }

    #[test]
    fn test_mock_http_client_error() {
        let client = MockHttpClient::new();
        client.mock_get("https://api.example.com/fail", Err(HttpError::Timeout));

        let api = ApiClient::new(Box::new(client), "https://api.example.com");
        let result = api.fetch_data("fail");
        assert!(matches!(result, Err(HttpError::Timeout)));
    }

    #[test]
    fn test_mock_http_client_tracks_calls() {
        let client = MockHttpClient::new();
        client.mock_get("https://api.example.com/a", Ok("a".to_string()));
        client.mock_get("https://api.example.com/b", Ok("b".to_string()));

        let api = ApiClient::new(Box::new(client), "https://api.example.com");
        let _ = api.fetch_data("a");
        let _ = api.fetch_data("b");

        // Note: we can't check calls because client was moved
        // In real code, you'd use Rc<MockHttpClient> or similar
    }

    #[test]
    fn test_call_recorder() {
        let db = MockDatabase::new();
        let recorder = CallRecorder::new(db);
        recorder.record("test_call");
        recorder.record("another_call");
        assert_eq!(recorder.call_count(), 2);
    }
}
