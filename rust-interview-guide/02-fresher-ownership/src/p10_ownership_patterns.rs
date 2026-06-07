/// Problem: Ownership Patterns
///
/// Common ownership patterns in Rust.
///
/// Key Concepts:
/// - Builder pattern
/// - RAII (Resource Acquisition Is Initialization)
/// - Cow (Clone on Write)
/// - Interior mutability patterns
/// - Ownership transfer patterns

use std::borrow::Cow;

/// Problem 1: Builder pattern
/// Use builder pattern with ownership
#[derive(Debug)]
pub struct QueryBuilder {
    table: String,
    conditions: Vec<String>,
    limit: Option<usize>,
}

impl QueryBuilder {
    pub fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            conditions: Vec::new(),
            limit: None,
        }
    }

    pub fn where_clause(mut self, condition: &str) -> Self {
        self.conditions.push(condition.to_string());
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn build(self) -> String {
        let mut query = format!("SELECT * FROM {}", self.table);
        if !self.conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&self.conditions.join(" AND "));
        }
        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        query
    }
}

/// Problem 2: RAII pattern
/// Resource management with ownership
pub struct FileHandle {
    pub path: String,
}

impl FileHandle {
    pub fn open(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}

impl Drop for FileHandle {
    fn drop(&mut self) {
        // Cleanup would happen here
    }
}

/// Problem 3: Cow pattern
/// Clone on Write for efficiency
pub fn process_data(data: Cow<str>) -> Cow<str> {
    if data.contains("error") {
        Cow::Owned(data.replace("error", "warning"))
    } else {
        data
    }
}

/// Problem 4: Ownership transfer pattern
/// Transfer ownership through function chain
pub fn process_string(s: String) -> String {
    let s = add_prefix(s, "processed_");
    let s = add_suffix(s, "_done");
    s
}

fn add_prefix(mut s: String, prefix: &str) -> String {
    s.insert_str(0, prefix);
    s
}

fn add_suffix(mut s: String, suffix: &str) -> String {
    s.push_str(suffix);
    s
}

/// Problem 5: Interior mutability pattern
/// Use RefCell for interior mutability
use std::cell::RefCell;

pub struct Cache {
    data: RefCell<std::collections::HashMap<String, String>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            data: RefCell::new(std::collections::HashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.data.borrow().get(key).cloned()
    }

    pub fn set(&self, key: &str, value: &str) {
        self.data.borrow_mut().insert(key.to_string(), value.to_string());
    }
}

/// Problem 6: Shared ownership pattern
/// Use Rc for shared ownership
use std::rc::Rc;

pub struct SharedConfig {
    pub config: Rc<std::collections::HashMap<String, String>>,
}

impl SharedConfig {
    pub fn new(config: std::collections::HashMap<String, String>) -> Self {
        Self {
            config: Rc::new(config),
        }
    }

    pub fn clone_config(&self) -> Rc<std::collections::HashMap<String, String>> {
        Rc::clone(&self.config)
    }
}

/// Problem 7: Ownership with Option
/// Use Option for optional ownership
pub struct OptionalOwner {
    pub data: Option<String>,
}

impl OptionalOwner {
    pub fn new(data: Option<String>) -> Self {
        Self { data }
    }

    pub fn take(&mut self) -> Option<String> {
        self.data.take()
    }

    pub fn replace(&mut self, data: String) -> Option<String> {
        self.data.replace(data)
    }
}

/// Problem 8: Ownership with Result
/// Use Result for fallible ownership
pub fn parse_and_own(s: &str) -> Result<String, String> {
    if s.is_empty() {
        Err("Empty string".to_string())
    } else {
        Ok(s.to_string())
    }
}

/// Problem 9: Ownership and vectors
/// Manage ownership in vectors
pub fn collect_owned(v: Vec<String>) -> Vec<String> {
    v.into_iter().filter(|s| !s.is_empty()).collect()
}

/// Problem 10: Ownership and enums
/// Enums with owned data
#[derive(Debug)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Text(String),
    List(Vec<Value>),
}

impl Value {
    pub fn to_string(&self) -> String {
        match self {
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Text(s) => s.clone(),
            Value::List(v) => {
                let items: Vec<String> = v.iter().map(|item| item.to_string()).collect();
                format!("[{}]", items.join(", "))
            }
        }
    }
}

/// Problem 11: Ownership with trait objects
/// Box<dyn Trait> owns the data
pub trait Processor {
    fn process(&self, input: &str) -> String;
}

pub struct UpperCase;
pub struct LowerCase;

impl Processor for UpperCase {
    fn process(&self, input: &str) -> String {
        input.to_uppercase()
    }
}

impl Processor for LowerCase {
    fn process(&self, input: &str) -> String {
        input.to_lowercase()
    }
}

pub fn create_processor(upper: bool) -> Box<dyn Processor> {
    if upper {
        Box::new(UpperCase)
    } else {
        Box::new(LowerCase)
    }
}

/// Problem 12: Ownership and closures
/// Closures that own their data
pub fn create_counter() -> impl FnMut() -> i32 {
    let mut count = 0;
    move || {
        count += 1;
        count
    }
}

/// Problem 13: Ownership and iterators
/// Iterators that own their data
pub fn owned_iterator(v: Vec<String>) -> impl Iterator<Item = String> {
    v.into_iter().map(|s| format!("processed_{}", s))
}

/// Problem 14: Ownership and pattern matching
/// Pattern matching with owned data
pub fn match_owned(value: Option<String>) -> String {
    match value {
        Some(s) => format!("Got: {}", s),
        None => "None".to_string(),
    }
}

/// Problem 15: Ownership and error handling
/// Error handling with owned data
pub fn handle_error(result: Result<String, String>) -> String {
    match result {
        Ok(s) => format!("Success: {}", s),
        Err(e) => format!("Error: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder() {
        let query = QueryBuilder::new("users")
            .where_clause("age > 18")
            .where_clause("active = true")
            .limit(10)
            .build();
        assert_eq!(query, "SELECT * FROM users WHERE age > 18 AND active = true LIMIT 10");
    }

    #[test]
    fn test_file_handle() {
        let _handle = FileHandle::open("test.txt");
        // Handle is dropped here
    }

    #[test]
    fn test_cow() {
        let data = Cow::Borrowed("hello");
        let result = process_data(data);
        assert_eq!(result, "hello");

        let data = Cow::Borrowed("error in code");
        let result = process_data(data);
        assert_eq!(result, "warning in code");
    }

    #[test]
    fn test_process_string() {
        assert_eq!(process_string("test".to_string()), "processed_test_done");
    }

    #[test]
    fn test_cache() {
        let cache = Cache::new();
        cache.set("key", "value");
        assert_eq!(cache.get("key"), Some("value".to_string()));
    }

    #[test]
    fn test_shared_config() {
        let mut config = std::collections::HashMap::new();
        config.insert("key".to_string(), "value".to_string());
        let shared = SharedConfig::new(config);
        let cloned = shared.clone_config();
        assert_eq!(cloned.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_optional_owner() {
        let mut owner = OptionalOwner::new(Some("data".to_string()));
        assert_eq!(owner.take(), Some("data".to_string()));
        assert_eq!(owner.take(), None);
    }

    #[test]
    fn test_parse_and_own() {
        assert_eq!(parse_and_own("hello"), Ok("hello".to_string()));
        assert_eq!(parse_and_own(""), Err("Empty string".to_string()));
    }

    #[test]
    fn test_collect_owned() {
        let v = vec!["hello".to_string(), "".to_string(), "world".to_string()];
        let result = collect_owned(v);
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_value_to_string() {
        assert_eq!(Value::Integer(42).to_string(), "42");
        assert_eq!(Value::Text("hello".to_string()).to_string(), "hello");
    }

    #[test]
    fn test_create_processor() {
        let upper = create_processor(true);
        assert_eq!(upper.process("hello"), "HELLO");

        let lower = create_processor(false);
        assert_eq!(lower.process("HELLO"), "hello");
    }

    #[test]
    fn test_create_counter() {
        let mut counter = create_counter();
        assert_eq!(counter(), 1);
        assert_eq!(counter(), 2);
    }

    #[test]
    fn test_owned_iterator() {
        let v = vec!["a".to_string(), "b".to_string()];
        let result: Vec<String> = owned_iterator(v).collect();
        assert_eq!(result, vec!["processed_a", "processed_b"]);
    }

    #[test]
    fn test_match_owned() {
        assert_eq!(match_owned(Some("hello".to_string())), "Got: hello");
        assert_eq!(match_owned(None), "None");
    }

    #[test]
    fn test_handle_error() {
        assert_eq!(handle_error(Ok("success".to_string())), "Success: success");
        assert_eq!(handle_error(Err("error".to_string())), "Error: error");
    }
}
