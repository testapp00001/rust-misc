//! # API Review Checklist
//!
//! A systematic approach to reviewing Rust APIs for correctness, ergonomics,
//! and future-proofing. This module provides a checklist and demonstrates
//! common pitfalls and best practices.
//!
//! ## Key Concepts
//! - **API review checklist**: Systematic evaluation of API quality
//! - **Common pitfalls**: Anti-patterns to avoid
//! - **Ergonomic APIs**: Making the easy thing easy and the hard thing possible
//! - **API design principles**: Consistency, discoverability, least surprise

use std::collections::HashMap;

/// An API review checklist item.
#[derive(Debug, Clone, PartialEq)]
pub enum ChecklistItem {
    /// Naming follows Rust conventions (snake_case for functions, CamelCase for types)
    NamingConventions,
    /// Error types are well-documented and non-exhaustive where appropriate
    ErrorDesign,
    /// Public items have doc comments with examples
    Documentation,
    /// Builder pattern used for types with 3+ constructor parameters
    BuilderPattern,
    /// Traits are object-safe if they need to be used as dyn Trait
    ObjectSafety,
    /// #[non_exhaustive] on enums/structs that may grow
    NonExhaustive,
    /// Implements standard traits (Debug, Clone, etc.) where appropriate
    StandardTraits,
    /// Uses impl Into<String> for string parameters
    StringParameters,
    /// Returns impl Iterator instead of Vec where possible
    IteratorReturns,
    /// Uses const generics or const fn for compile-time optimization
    ConstGenerics,
}

/// A review result with issues and suggestions.
#[derive(Debug, Clone)]
pub struct ReviewResult {
    pub passed: Vec<ChecklistItem>,
    pub warnings: Vec<(ChecklistItem, String)>,
    pub errors: Vec<(ChecklistItem, String)>,
}

impl ReviewResult {
    pub fn new() -> Self {
        ReviewResult {
            passed: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn pass(&mut self, item: ChecklistItem) {
        self.passed.push(item);
    }

    pub fn warn(&mut self, item: ChecklistItem, message: impl Into<String>) {
        self.warnings.push((item, message.into()));
    }

    pub fn error(&mut self, item: ChecklistItem, message: impl Into<String>) {
        self.errors.push((item, message.into()));
    }

    pub fn is_passing(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn summary(&self) -> String {
        format!(
            "Passed: {}, Warnings: {}, Errors: {}",
            self.passed.len(),
            self.warnings.len(),
            self.errors.len()
        )
    }
}

/// Demonstrates common API design pitfalls and their fixes.
pub mod pitfall_examples {
    /// PITFALL: Using String when &str would suffice.
    /// FIX: Use impl Into<String> for owned parameters, &str for borrowed.
    pub fn greet(name: &str) -> String {
        format!("Hello, {name}!")
    }

    /// PITFALL: Returning Vec when an iterator would be more flexible.
    /// FIX: Return impl Iterator<Item = T>.
    pub fn range(start: u32, end: u32) -> impl Iterator<Item = u32> {
        start..end
    }

    /// PITFALL: Using i32 for sizes (should be usize).
    /// FIX: Use usize for sizes, indices, and counts.
    pub fn process_batch(items: &[String], batch_size: usize) -> Vec<&[String]> {
        items.chunks(batch_size).collect()
    }
}

/// Demonstrates ergonomic API patterns.
pub mod ergonomic_patterns {
    use super::*;

    /// Pattern: Accept impl Into<T> for flexible input.
    pub fn create_user(
        name: impl Into<String>,
        email: impl Into<String>,
        age: u32,
    ) -> User {
        User {
            name: name.into(),
            email: email.into(),
            age,
        }
    }

    #[derive(Debug, Clone)]
    pub struct User {
        pub name: String,
        pub email: String,
        pub age: u32,
    }

    /// Pattern: Return Result with descriptive error variants.
    pub fn parse_age(input: &str) -> Result<u32, ParseError> {
        let age: u32 = input
            .parse()
            .map_err(|_| ParseError::NotANumber(input.to_string()))?;

        if age > 150 {
            return Err(ParseError::UnrealisticAge(age));
        }

        Ok(age)
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum ParseError {
        NotANumber(String),
        UnrealisticAge(u32),
    }

    impl std::fmt::Display for ParseError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ParseError::NotANumber(s) => write!(f, "Not a valid number: {s}"),
                ParseError::UnrealisticAge(a) => write!(f, "Unrealistic age: {a}"),
            }
        }
    }

    impl std::error::Error for ParseError {}

    /// Pattern: Builder for complex construction.
    pub struct QueryBuilder {
        table: Option<String>,
        conditions: Vec<String>,
        limit: Option<usize>,
    }

    impl QueryBuilder {
        pub fn new() -> Self {
            QueryBuilder {
                table: None,
                conditions: Vec::new(),
                limit: None,
            }
        }

        pub fn from(mut self, table: impl Into<String>) -> Self {
            self.table = Some(table.into());
            self
        }

        pub fn where_clause(mut self, condition: impl Into<String>) -> Self {
            self.conditions.push(condition.into());
            self
        }

        pub fn limit(mut self, n: usize) -> Self {
            self.limit = Some(n);
            self
        }

        pub fn build(self) -> Result<String, String> {
            let table = self.table.ok_or("Table is required")?;
            let mut query = format!("SELECT * FROM {table}");
            if !self.conditions.is_empty() {
                query.push_str(&format!(" WHERE {}", self.conditions.join(" AND ")));
            }
            if let Some(limit) = self.limit {
                query.push_str(&format!(" LIMIT {limit}"));
            }
            Ok(query)
        }
    }
}

/// A type-safe configuration system that demonstrates multiple API design principles.
#[derive(Debug, Clone)]
pub struct Config {
    values: HashMap<String, ConfigValue>,
}

#[derive(Debug, Clone)]
pub enum ConfigValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<String>),
}

impl Config {
    pub fn new() -> Self {
        Config {
            values: HashMap::new(),
        }
    }

    pub fn set_string(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.values
            .insert(key.into(), ConfigValue::String(value.into()));
    }

    pub fn set_int(&mut self, key: impl Into<String>, value: i64) {
        self.values.insert(key.into(), ConfigValue::Int(value));
    }

    pub fn set_bool(&mut self, key: impl Into<String>, value: bool) {
        self.values.insert(key.into(), ConfigValue::Bool(value));
    }

    pub fn get_string(&self, key: &str) -> Option<&str> {
        match self.values.get(key) {
            Some(ConfigValue::String(s)) => Some(s),
            _ => None,
        }
    }

    pub fn get_int(&self, key: &str) -> Option<i64> {
        match self.values.get(key) {
            Some(ConfigValue::Int(n)) => Some(*n),
            _ => None,
        }
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.values.get(key) {
            Some(ConfigValue::Bool(b)) => Some(*b),
            _ => None,
        }
    }

    pub fn get_or_string<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get_string(key).unwrap_or(default)
    }

    pub fn get_or_int(&self, key: &str, default: i64) -> i64 {
        self.get_int(key).unwrap_or(default)
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.values.keys().map(|s| s.as_str())
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// An API design principle validator.
pub struct ApiValidator;

impl ApiValidator {
    /// Checks if a function name follows Rust conventions.
    pub fn is_valid_function_name(name: &str) -> bool {
        !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit())
            && !name.starts_with('_')
            && !name.ends_with('_')
    }

    /// Checks if a type name follows Rust conventions.
    pub fn is_valid_type_name(name: &str) -> bool {
        !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_lowercase() || c.is_ascii_digit())
            && name
                .chars()
                .next()
                .map_or(false, |c| c.is_ascii_uppercase())
    }

    /// Checks if an error message is user-friendly.
    pub fn is_good_error_message(msg: &str) -> bool {
        !msg.is_empty()
            && msg.len() >= 10
            && msg.len() <= 200
            && msg.chars().next().map_or(false, |c| c.is_uppercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ergonomic_patterns::*;
    use pitfall_examples;

    #[test]
    fn test_review_result() {
        let mut result = ReviewResult::new();
        result.pass(ChecklistItem::NamingConventions);
        result.pass(ChecklistItem::Documentation);
        result.warn(
            ChecklistItem::ObjectSafety,
            "Consider making trait object-safe",
        );
        result.error(
            ChecklistItem::ErrorDesign,
            "Error type should be #[non_exhaustive]",
        );

        assert!(!result.is_passing());
        assert!(result.summary().contains("Passed: 2"));
    }

    #[test]
    fn test_pitfall_greet() {
        assert_eq!(pitfall_examples::greet("Alice"), "Hello, Alice!");
    }

    #[test]
    fn test_pitfall_range() {
        let items: Vec<u32> = pitfall_examples::range(0, 5).collect();
        assert_eq!(items, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_pitfall_process_batch() {
        let items: Vec<String> = (0..10).map(|i| format!("item-{i}")).collect();
        let batches = pitfall_examples::process_batch(&items, 3);
        assert_eq!(batches.len(), 4); // 3+3+3+1
    }

    #[test]
    fn test_create_user() {
        let user = create_user("Alice", "alice@example.com", 30);
        assert_eq!(user.name, "Alice");
        assert_eq!(user.email, "alice@example.com");
        assert_eq!(user.age, 30);
    }

    #[test]
    fn test_parse_age_valid() {
        assert_eq!(parse_age("25"), Ok(25));
    }

    #[test]
    fn test_parse_age_invalid() {
        assert_eq!(
            parse_age("abc"),
            Err(ParseError::NotANumber("abc".into()))
        );
        assert_eq!(parse_age("200"), Err(ParseError::UnrealisticAge(200)));
    }

    #[test]
    fn test_query_builder() {
        let query = QueryBuilder::new()
            .from("users")
            .where_clause("age > 18")
            .where_clause("active = true")
            .limit(10)
            .build()
            .unwrap();

        assert!(query.contains("FROM users"));
        assert!(query.contains("WHERE age > 18 AND active = true"));
        assert!(query.contains("LIMIT 10"));
    }

    #[test]
    fn test_query_builder_no_table() {
        let result = QueryBuilder::new().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_basic() {
        let mut config = Config::new();
        config.set_string("host", "localhost");
        config.set_int("port", 8080);
        config.set_bool("tls", true);

        assert_eq!(config.get_string("host"), Some("localhost"));
        assert_eq!(config.get_int("port"), Some(8080));
        assert_eq!(config.get_bool("tls"), Some(true));
    }

    #[test]
    fn test_config_defaults() {
        let config = Config::new();
        assert_eq!(config.get_or_string("host", "default"), "default");
        assert_eq!(config.get_or_int("port", 3000), 3000);
    }

    #[test]
    fn test_config_keys() {
        let mut config = Config::new();
        config.set_string("a", "1");
        config.set_string("b", "2");

        let mut keys: Vec<&str> = config.keys().collect();
        keys.sort();
        assert_eq!(keys, vec!["a", "b"]);
    }

    #[test]
    fn test_api_validator_function_names() {
        assert!(ApiValidator::is_valid_function_name("get_value"));
        assert!(ApiValidator::is_valid_function_name("process"));
        assert!(!ApiValidator::is_valid_function_name("GetValue")); // CamelCase
        assert!(!ApiValidator::is_valid_function_name("_private")); // starts with _
        assert!(!ApiValidator::is_valid_function_name("")); // empty
    }

    #[test]
    fn test_api_validator_type_names() {
        assert!(ApiValidator::is_valid_type_name("MyStruct"));
        assert!(ApiValidator::is_valid_type_name("Config"));
        assert!(!ApiValidator::is_valid_type_name("myStruct")); // lowercase start
        assert!(!ApiValidator::is_valid_type_name("")); // empty
    }

    #[test]
    fn test_good_error_message() {
        assert!(ApiValidator::is_good_error_message("Connection refused by server"));
        assert!(!ApiValidator::is_good_error_message("")); // empty
        assert!(!ApiValidator::is_good_error_message("err")); // too short
        assert!(!ApiValidator::is_good_error_message("lowercase start")); // lowercase
    }
}
