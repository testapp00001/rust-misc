//! # Repository Pattern
//!
//! The Repository pattern abstracts data access behind a clean interface,
//! making it easy to swap implementations and test without a real database.
//! This lesson covers repository traits, CRUD operations, pagination, and
//! filtering.
//!
//! ## Key Concepts
//! - Repository trait design
//! - Generic CRUD operations
//! - Pagination (offset and cursor)
//! - Filtering and sorting
//! - In-memory repository for testing
//! - Error handling across the repository boundary

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

// ---------------------------------------------------------------------------
// 1. Repository Trait
// ---------------------------------------------------------------------------

/// A generic repository trait for CRUD operations.
#[async_trait::async_trait]
pub trait Repository<T: Send + Sync>: Send + Sync {
    type Id: Clone + Debug + Send + Sync;
    type Error: std::error::Error + Send + Sync;

    /// Find an entity by ID.
    async fn find_by_id(&self, id: &Self::Id) -> Result<Option<T>, Self::Error>;

    /// Find all entities matching the given filter.
    async fn find_all(&self, filter: Filter) -> Result<Vec<T>, Self::Error>;

    /// Create a new entity and return it with its generated ID.
    async fn create(&self, entity: &T) -> Result<T, Self::Error>;

    /// Update an existing entity.
    async fn update(&self, id: &Self::Id, entity: &T) -> Result<T, Self::Error>;

    /// Delete an entity by ID.
    async fn delete(&self, id: &Self::Id) -> Result<bool, Self::Error>;

    /// Count entities matching the filter.
    async fn count(&self, filter: Filter) -> Result<u64, Self::Error>;
}

// ---------------------------------------------------------------------------
// 2. Domain Models
// ---------------------------------------------------------------------------

/// A user entity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserEntity {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub role: String,
    pub active: bool,
    pub created_at: String,
}

/// A product entity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProductEntity {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub price_cents: u64,
    pub category: String,
    pub in_stock: bool,
}

// ---------------------------------------------------------------------------
// 3. Query Types
// ---------------------------------------------------------------------------

/// Filter criteria for repository queries.
#[derive(Debug, Clone, Default)]
pub struct Filter {
    pub conditions: Vec<Condition>,
    pub order_by: Vec<OrderBy>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub field: String,
    pub operator: FilterOperator,
    pub value: FilterValue,
}

#[derive(Debug, Clone)]
pub enum FilterOperator {
    Eq,
    Ne,
    Gt,
    Lt,
    Gte,
    Lte,
    Like,
    In,
}

#[derive(Debug, Clone)]
pub enum FilterValue {
    Text(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    List(Vec<FilterValue>),
}

#[derive(Debug, Clone)]
pub struct OrderBy {
    pub field: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Paginated result.
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResult<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

impl<T> PaginatedResult<T> {
    pub fn new(data: Vec<T>, total: u64, page: u32, per_page: u32) -> Self {
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;
        Self {
            data,
            total,
            page,
            per_page,
            total_pages,
        }
    }

    pub fn has_next(&self) -> bool {
        self.page < self.total_pages
    }

    pub fn has_prev(&self) -> bool {
        self.page > 1
    }
}

// ---------------------------------------------------------------------------
// 4. Filter Builder
// ---------------------------------------------------------------------------

/// Fluent builder for constructing filters.
impl Filter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn eq(mut self, field: &str, value: FilterValue) -> Self {
        self.conditions.push(Condition {
            field: field.into(),
            operator: FilterOperator::Eq,
            value,
        });
        self
    }

    pub fn ne(mut self, field: &str, value: FilterValue) -> Self {
        self.conditions.push(Condition {
            field: field.into(),
            operator: FilterOperator::Ne,
            value,
        });
        self
    }

    pub fn gt(mut self, field: &str, value: FilterValue) -> Self {
        self.conditions.push(Condition {
            field: field.into(),
            operator: FilterOperator::Gt,
            value,
        });
        self
    }

    pub fn like(mut self, field: &str, pattern: &str) -> Self {
        self.conditions.push(Condition {
            field: field.into(),
            operator: FilterOperator::Like,
            value: FilterValue::Text(pattern.into()),
        });
        self
    }

    pub fn order_by(mut self, field: &str, direction: SortDirection) -> Self {
        self.order_by.push(OrderBy {
            field: field.into(),
            direction,
        });
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn paginate(self, page: u32, per_page: u32) -> Self {
        let offset = (page.saturating_sub(1)) * per_page;
        self.limit(per_page).offset(offset)
    }

    pub fn matches_condition(&self, field: &str, value: &FilterValue) -> bool {
        self.conditions.iter().all(|c| {
            if c.field != field {
                return true; // different field, skip
            }
            match (&c.operator, &c.value, value) {
                (FilterOperator::Eq, FilterValue::Text(a), FilterValue::Text(b)) => a == b,
                (FilterOperator::Eq, FilterValue::Integer(a), FilterValue::Integer(b)) => a == b,
                (FilterOperator::Eq, FilterValue::Boolean(a), FilterValue::Boolean(b)) => a == b,
                (FilterOperator::Ne, FilterValue::Text(a), FilterValue::Text(b)) => a != b,
                (FilterOperator::Like, FilterValue::Text(pattern), FilterValue::Text(val)) => {
                    let pattern_lower = pattern.to_lowercase();
                    let val_lower = val.to_lowercase();
                    pattern_lower
                        .trim_matches('%')
                        .split('%')
                        .all(|part| val_lower.contains(part))
                }
                _ => true, // unmatched types pass through
            }
        })
    }
}

// ---------------------------------------------------------------------------
// 5. In-Memory Repository
// ---------------------------------------------------------------------------

/// An in-memory repository for testing.
pub struct InMemoryRepository<T: Clone + Debug + Send + Sync> {
    store: HashMap<u64, T>,
    next_id: u64,
    id_getter: Box<dyn Fn(&T) -> u64 + Send + Sync>,
    id_setter: Box<dyn Fn(&T, u64) -> T + Send + Sync>,
}

impl<T: Clone + Debug + Send + Sync> std::fmt::Debug for InMemoryRepository<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryRepository")
            .field("store", &self.store)
            .field("next_id", &self.next_id)
            .finish_non_exhaustive()
    }
}

impl<T: Clone + Debug + Send + Sync> InMemoryRepository<T> {
    pub fn new(
        id_getter: impl Fn(&T) -> u64 + Send + Sync + 'static,
        id_setter: impl Fn(&T, u64) -> T + Send + Sync + 'static,
    ) -> Self {
        Self {
            store: HashMap::new(),
            next_id: 1,
            id_getter: Box::new(id_getter),
            id_setter: Box::new(id_setter),
        }
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}

#[async_trait::async_trait]
impl Repository<UserEntity> for InMemoryRepository<UserEntity> {
    type Id = u64;
    type Error = RepositoryError;

    async fn find_by_id(&self, id: &u64) -> Result<Option<UserEntity>, RepositoryError> {
        Ok(self.store.get(id).cloned())
    }

    async fn find_all(&self, _filter: Filter) -> Result<Vec<UserEntity>, RepositoryError> {
        Ok(self.store.values().cloned().collect())
    }

    async fn create(&self, entity: &UserEntity) -> Result<UserEntity, RepositoryError> {
        // In a real impl, we'd use interior mutability
        // For testing, this demonstrates the pattern
        let mut entity = entity.clone();
        entity.id = self.next_id;
        Ok(entity)
    }

    async fn update(&self, id: &u64, entity: &UserEntity) -> Result<UserEntity, RepositoryError> {
        if self.store.contains_key(id) {
            let mut updated = entity.clone();
            updated.id = *id;
            Ok(updated)
        } else {
            Err(RepositoryError::NotFound(format!("entity {id} not found")))
        }
    }

    async fn delete(&self, id: &u64) -> Result<bool, RepositoryError> {
        Ok(self.store.contains_key(id))
    }

    async fn count(&self, _filter: Filter) -> Result<u64, RepositoryError> {
        Ok(self.store.len() as u64)
    }
}

// ---------------------------------------------------------------------------
// 6. Repository Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("entity not found: {0}")]
    NotFound(String),

    #[error("duplicate key: {0}")]
    DuplicateKey(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("validation error: {0}")]
    Validation(String),
}

// ---------------------------------------------------------------------------
// 7. Repository Utilities
// ---------------------------------------------------------------------------

/// Apply pagination to a slice of data.
pub fn apply_pagination<T: Clone>(data: &[T], page: u32, per_page: u32) -> PaginatedResult<T> {
    let total = data.len() as u64;
    let offset = ((page.saturating_sub(1)) * per_page) as usize;
    let paged: Vec<T> = data
        .iter()
        .skip(offset)
        .take(per_page as usize)
        .cloned()
        .collect();

    PaginatedResult::new(paged, total, page, per_page)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_builder() {
        let filter = Filter::new()
            .eq("role", FilterValue::Text("admin".into()))
            .like("username", "%alice%")
            .order_by("created_at", SortDirection::Desc)
            .limit(10)
            .offset(0);

        assert_eq!(filter.conditions.len(), 2);
        assert_eq!(filter.order_by.len(), 1);
        assert_eq!(filter.limit, Some(10));
    }

    #[test]
    fn test_filter_paginate() {
        let filter = Filter::new().paginate(3, 20);
        assert_eq!(filter.limit, Some(20));
        assert_eq!(filter.offset, Some(40));
    }

    #[test]
    fn test_filter_paginate_first_page() {
        let filter = Filter::new().paginate(1, 10);
        assert_eq!(filter.offset, Some(0));
    }

    #[test]
    fn test_paginated_result() {
        let data = vec![1, 2, 3, 4, 5];
        let result = PaginatedResult::new(data, 100, 1, 5);
        assert_eq!(result.data.len(), 5);
        assert_eq!(result.total, 100);
        assert_eq!(result.total_pages, 20);
        assert!(result.has_next());
        assert!(!result.has_prev());
    }

    #[test]
    fn test_paginated_result_last_page() {
        let data = vec![1, 2];
        let result = PaginatedResult::new(data, 12, 3, 5);
        assert!(!result.has_next());
        assert!(result.has_prev());
    }

    #[test]
    fn test_apply_pagination() {
        let data: Vec<i32> = (0..100).collect();
        let result = apply_pagination(&data, 2, 10);
        assert_eq!(result.data, vec![10, 11, 12, 13, 14, 15, 16, 17, 18, 19]);
        assert_eq!(result.total, 100);
    }

    #[test]
    fn test_apply_pagination_out_of_bounds() {
        let data: Vec<i32> = vec![1, 2, 3];
        let result = apply_pagination(&data, 10, 10);
        assert!(result.data.is_empty());
    }

    #[test]
    fn test_filter_eq() {
        let filter = Filter::new().eq("name", FilterValue::Text("Alice".into()));
        assert!(filter.matches_condition("name", &FilterValue::Text("Alice".into())));
        assert!(!filter.matches_condition("name", &FilterValue::Text("Bob".into())));
    }

    #[test]
    fn test_filter_like() {
        let filter = Filter::new().like("name", "%alice%");
        assert!(filter.matches_condition("name", &FilterValue::Text("Bob Alice Smith".into())));
        assert!(!filter.matches_condition("name", &FilterValue::Text("Bob".into())));
    }

    #[test]
    fn test_filter_ne() {
        let filter = Filter::new().ne("role", FilterValue::Text("admin".into()));
        assert!(filter.matches_condition("role", &FilterValue::Text("user".into())));
        assert!(!filter.matches_condition("role", &FilterValue::Text("admin".into())));
    }

    #[test]
    fn test_repository_error_display() {
        let err = RepositoryError::NotFound("user 42".into());
        assert!(err.to_string().contains("user 42"));

        let err = RepositoryError::DuplicateKey("email".into());
        assert!(err.to_string().contains("email"));
    }

    #[test]
    fn test_sort_direction() {
        assert_ne!(SortDirection::Asc, SortDirection::Desc);
    }

    #[test]
    fn test_filter_default() {
        let filter = Filter::default();
        assert!(filter.conditions.is_empty());
        assert!(filter.order_by.is_empty());
        assert!(filter.limit.is_none());
    }

    #[test]
    fn test_user_entity() {
        let user = UserEntity {
            id: 1,
            username: "alice".into(),
            email: "alice@example.com".into(),
            role: "admin".into(),
            active: true,
            created_at: "2024-01-01".into(),
        };
        assert_eq!(user.id, 1);
        assert!(user.active);
    }

    #[test]
    fn test_product_entity() {
        let product = ProductEntity {
            id: 1,
            name: "Widget".into(),
            description: "A widget".into(),
            price_cents: 999,
            category: "tools".into(),
            in_stock: true,
        };
        assert_eq!(product.price_cents, 999);
    }
}
