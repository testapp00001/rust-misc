//! # Refactoring Patterns
//!
//! Safe refactoring patterns for improving Rust code structure. These patterns
//! maintain behavior while improving design.
//!
//! ## Key Patterns:
//!
//! - **Extract Function**: Break long functions into smaller ones
//! - **Extract Trait**: Define behavior interfaces
//! - **Introduce Newtype**: Add type safety with wrapper types
//! - **Replace Enum with Trait**: When behavior varies more than data

use std::collections::HashMap;

/// Newtype pattern: wrapper for type safety.
/// Prevents mixing up different types with the same underlying representation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrderId(pub u64);

#[derive(Debug, Clone, PartialEq)]
pub struct Email(pub String);

impl Email {
    pub fn new(email: &str) -> Result<Self, String> {
        if email.contains('@') {
            Ok(Self(email.into()))
        } else {
            Err("Invalid email".into())
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Trait extraction pattern: define behavior interfaces.
pub trait Repository<T> {
    fn find_by_id(&self, id: u64) -> Option<T>;
    fn save(&mut self, id: u64, item: T);
    fn delete(&mut self, id: u64) -> bool;
    fn count(&self) -> usize;
}

/// In-memory repository implementation.
pub struct InMemoryRepository<T: Clone> {
    data: HashMap<u64, T>,
}

impl<T: Clone> InMemoryRepository<T> {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl<T: Clone> Repository<T> for InMemoryRepository<T> {
    fn find_by_id(&self, id: u64) -> Option<T> {
        self.data.get(&id).cloned()
    }

    fn save(&mut self, id: u64, item: T) {
        self.data.insert(id, item);
    }

    fn delete(&mut self, id: u64) -> bool {
        self.data.remove(&id).is_some()
    }

    fn count(&self) -> usize {
        self.data.len()
    }
}

/// Builder pattern for complex construction.
pub struct QueryBuilder {
    table: String,
    conditions: Vec<String>,
    order_by: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

impl QueryBuilder {
    pub fn new(table: &str) -> Self {
        Self {
            table: table.into(),
            conditions: Vec::new(),
            order_by: None,
            limit: None,
            offset: None,
        }
    }

    pub fn where_clause(mut self, condition: &str) -> Self {
        self.conditions.push(condition.into());
        self
    }

    pub fn order_by(mut self, field: &str) -> Self {
        self.order_by = Some(field.into());
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn build(self) -> String {
        let mut query = format!("SELECT * FROM {}", self.table);

        if !self.conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&self.conditions.join(" AND "));
        }

        if let Some(order) = self.order_by {
            query.push_str(&format!(" ORDER BY {}", order));
        }

        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        query
    }
}

/// Extract function pattern: helper for complex operations.
pub struct DataProcessor;

impl DataProcessor {
    /// Before refactoring: one large function.
    pub fn process_raw(data: &[i32]) -> (i32, f64, i32) {
        let sum: i32 = data.iter().sum();
        let avg = sum as f64 / data.len() as f64;
        let max = *data.iter().max().unwrap_or(&0);
        (sum, avg, max)
    }

    /// After refactoring: extracted helper functions.
    pub fn process_refactored(data: &[i32]) -> (i32, f64, i32) {
        let sum = Self::calculate_sum(data);
        let avg = Self::calculate_average(data);
        let max = Self::find_max(data);
        (sum, avg, max)
    }

    fn calculate_sum(data: &[i32]) -> i32 {
        data.iter().sum()
    }

    fn calculate_average(data: &[i32]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        Self::calculate_sum(data) as f64 / data.len() as f64
    }

    fn find_max(data: &[i32]) -> i32 {
        *data.iter().max().unwrap_or(&0)
    }
}

/// Replace conditionals with polymorphism.
pub trait Shape {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
    fn name(&self) -> &str;
}

pub struct Circle {
    pub radius: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }

    fn perimeter(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }

    fn name(&self) -> &str {
        "Circle"
    }
}

pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }

    fn perimeter(&self) -> f64 {
        2.0 * (self.width + self.height)
    }

    fn name(&self) -> &str {
        "Rectangle"
    }
}

/// Refactoring: Extract interface from concrete type.
pub struct UserService {
    users: HashMap<u64, String>,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    pub fn add_user(&mut self, id: u64, name: &str) {
        self.users.insert(id, name.into());
    }

    pub fn get_user(&self, id: u64) -> Option<&str> {
        self.users.get(&id).map(|s| s.as_str())
    }
}

/// After refactoring: trait-based interface.
pub trait UserProvider {
    fn get_user(&self, id: u64) -> Option<String>;
}

impl UserProvider for UserService {
    fn get_user(&self, id: u64) -> Option<String> {
        self.users.get(&id).cloned()
    }
}

/// Refactoring metrics.
pub struct RefactoringMetrics {
    pub before_complexity: usize,
    pub after_complexity: usize,
    pub before_lines: usize,
    pub after_lines: usize,
}

impl RefactoringMetrics {
    pub fn complexity_reduction(&self) -> f64 {
        if self.before_complexity == 0 {
            return 0.0;
        }
        ((self.before_complexity - self.after_complexity) as f64
            / self.before_complexity as f64)
            * 100.0
    }

    pub fn line_reduction(&self) -> f64 {
        if self.before_lines == 0 {
            return 0.0;
        }
        ((self.before_lines - self.after_lines) as f64 / self.before_lines as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_newtype_user_id() {
        let id = UserId(42);
        assert_eq!(id.0, 42);
    }

    #[test]
    fn test_newtype_prevents_mixing() {
        let user_id = UserId(1);
        let order_id = OrderId(1);
        // These are different types even though both wrap u64
        assert_ne!(format!("{:?}", user_id), format!("{:?}", order_id));
    }

    #[test]
    fn test_email_validation() {
        assert!(Email::new("user@example.com").is_ok());
        assert!(Email::new("invalid").is_err());
    }

    #[test]
    fn test_in_memory_repository() {
        let mut repo = InMemoryRepository::new();
        repo.save(1, "Alice".to_string());
        repo.save(2, "Bob".to_string());

        assert_eq!(repo.count(), 2);
        assert_eq!(repo.find_by_id(1), Some("Alice".to_string()));
        assert!(repo.delete(1));
        assert_eq!(repo.count(), 1);
    }

    #[test]
    fn test_query_builder() {
        let query = QueryBuilder::new("users")
            .where_clause("age > 18")
            .where_clause("active = true")
            .order_by("name")
            .limit(10)
            .offset(20)
            .build();

        assert!(query.contains("SELECT * FROM users"));
        assert!(query.contains("WHERE age > 18 AND active = true"));
        assert!(query.contains("ORDER BY name"));
        assert!(query.contains("LIMIT 10"));
        assert!(query.contains("OFFSET 20"));
    }

    #[test]
    fn test_query_builder_minimal() {
        let query = QueryBuilder::new("orders").build();
        assert_eq!(query, "SELECT * FROM orders");
    }

    #[test]
    fn test_data_processor_equivalence() {
        let data = vec![1, 2, 3, 4, 5];
        let raw = DataProcessor::process_raw(&data);
        let refactored = DataProcessor::process_refactored(&data);
        assert_eq!(raw, refactored);
    }

    #[test]
    fn test_data_processor_empty() {
        let data = vec![];
        let (sum, avg, max) = DataProcessor::process_refactored(&data);
        assert_eq!(sum, 0);
        assert_eq!(avg, 0.0);
        assert_eq!(max, 0);
    }

    #[test]
    fn test_shape_polymorphism() {
        let shapes: Vec<Box<dyn Shape>> = vec![
            Box::new(Circle { radius: 5.0 }),
            Box::new(Rectangle {
                width: 4.0,
                height: 6.0,
            }),
        ];

        let total_area: f64 = shapes.iter().map(|s| s.area()).sum();
        let circle_area = std::f64::consts::PI * 25.0;
        let rect_area = 24.0;
        assert!((total_area - (circle_area + rect_area)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_circle() {
        let circle = Circle { radius: 1.0 };
        assert!((circle.area() - std::f64::consts::PI).abs() < f64::EPSILON);
        assert_eq!(circle.name(), "Circle");
    }

    #[test]
    fn test_rectangle() {
        let rect = Rectangle {
            width: 3.0,
            height: 4.0,
        };
        assert!((rect.area() - 12.0).abs() < f64::EPSILON);
        assert!((rect.perimeter() - 14.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_user_service() {
        let mut service = UserService::new();
        service.add_user(1, "Alice");
        assert_eq!(service.get_user(1), Some("Alice"));
        assert_eq!(service.get_user(2), None);
    }

    #[test]
    fn test_user_provider_trait() {
        let mut service = UserService::new();
        service.add_user(1, "Bob");

        let provider: &dyn UserProvider = &service;
        assert_eq!(provider.get_user(1), Some("Bob".to_string()));
    }

    #[test]
    fn test_refactoring_metrics() {
        let metrics = RefactoringMetrics {
            before_complexity: 20,
            after_complexity: 10,
            before_lines: 100,
            after_lines: 60,
        };
        assert!((metrics.complexity_reduction() - 50.0).abs() < f64::EPSILON);
        assert!((metrics.line_reduction() - 40.0).abs() < f64::EPSILON);
    }
}
