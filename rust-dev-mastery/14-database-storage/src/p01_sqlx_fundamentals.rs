//! # SQLx Fundamentals
//!
//! SQLx is an async, pure-Rust SQL toolkit with compile-time query checking.
//! This lesson covers the core patterns for database interaction without an ORM,
//! focusing on type-safe queries, result mapping, and error handling.
//!
//! ## Key Concepts
//! - `sqlx::query!` and `sqlx::query_as!` for compile-time checked queries
//! - Dynamic queries with `sqlx::query` and `sqlx::query_as`
//! - Type mapping between SQL and Rust types
//! - Error handling with `sqlx::Error`
//! - `FromRow` derive for automatic mapping
//! - Scalar, fetch, and execute operations

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// 1. Database Models
// ---------------------------------------------------------------------------

/// A user record as stored in the database.
/// In production with compile-time checked queries, you'd use `#[derive(FromRow)]`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Input for creating a new user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
}

/// Input for updating a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub active: Option<bool>,
}

// ---------------------------------------------------------------------------
// 2. Query Builder Pattern
// ---------------------------------------------------------------------------

/// A SQL query builder for constructing dynamic queries safely.
/// This pattern is used when compile-time checking isn't available.
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    sql: String,
    params: Vec<QueryParam>,
}

#[derive(Debug, Clone)]
pub enum QueryParam {
    Text(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
}

impl QueryParam {
    /// Get the SQL placeholder representation.
    pub fn placeholder(index: usize) -> String {
        format!("${index}")
    }
}

impl QueryBuilder {
    pub fn new(sql: impl Into<String>) -> Self {
        Self {
            sql: sql.into(),
            params: Vec::new(),
        }
    }

    pub fn bind_text(mut self, value: impl Into<String>) -> Self {
        self.params.push(QueryParam::Text(value.into()));
        self
    }

    pub fn bind_int(mut self, value: i64) -> Self {
        self.params.push(QueryParam::Integer(value));
        self
    }

    pub fn bind_float(mut self, value: f64) -> Self {
        self.params.push(QueryParam::Float(value));
        self
    }

    pub fn bind_bool(mut self, value: bool) -> Self {
        self.params.push(QueryParam::Boolean(value));
        self
    }

    pub fn bind_null(mut self) -> Self {
        self.params.push(QueryParam::Null);
        self
    }

    pub fn sql(&self) -> &str {
        &self.sql
    }

    pub fn params(&self) -> &[QueryParam] {
        &self.params
    }

    pub fn param_count(&self) -> usize {
        self.params.len()
    }
}

// ---------------------------------------------------------------------------
// 3. Common Query Patterns
// ---------------------------------------------------------------------------

/// Patterns for common SQL operations.
pub struct QueryPatterns;

impl QueryPatterns {
    /// Generate a SELECT query with pagination.
    pub fn select_paginated(
        table: &str,
        columns: &[&str],
        page: u32,
        per_page: u32,
    ) -> QueryBuilder {
        let cols = columns.join(", ");
        let offset = (page.saturating_sub(1)) * per_page;
        QueryBuilder::new(format!(
            "SELECT {cols} FROM {table} LIMIT {} OFFSET {}",
            per_page, offset
        ))
    }

    /// Generate a SELECT with WHERE clause.
    pub fn select_where(
        table: &str,
        columns: &[&str],
        conditions: &[(&str, &str)],
    ) -> (String, Vec<String>) {
        let cols = columns.join(", ");
        let mut where_parts = Vec::new();
        let mut params = Vec::new();

        for (i, (col, op)) in conditions.iter().enumerate() {
            where_parts.push(format!("{col} {} ${}", op, i + 1));
            params.push(format!("param_{}", i + 1));
        }

        let where_clause = where_parts.join(" AND ");
        let sql = format!("SELECT {cols} FROM {table} WHERE {where_clause}");
        (sql, params)
    }

    /// Generate an INSERT query.
    pub fn insert(table: &str, columns: &[&str]) -> String {
        let cols = columns.join(", ");
        let placeholders: Vec<String> = (1..=columns.len())
            .map(|i| format!("${i}"))
            .collect();
        let vals = placeholders.join(", ");
        format!("INSERT INTO {table} ({cols}) VALUES ({vals}) RETURNING *")
    }

    /// Generate an UPDATE query.
    pub fn update(table: &str, columns: &[&str], id_column: &str) -> String {
        let set_parts: Vec<String> = columns
            .iter()
            .enumerate()
            .map(|(i, col)| format!("{col} = ${}", i + 1))
            .collect();
        let set_clause = set_parts.join(", ");
        let id_param = columns.len() + 1;
        format!(
            "UPDATE {table} SET {set_clause} WHERE {id_column} = ${id_param} RETURNING *"
        )
    }

    /// Generate a DELETE query.
    pub fn delete(table: &str, id_column: &str) -> String {
        format!("DELETE FROM {table} WHERE {id_column} = $1")
    }

    /// Generate a COUNT query.
    pub fn count(table: &str, conditions: Option<&str>) -> String {
        match conditions {
            Some(where_clause) => format!("SELECT COUNT(*) FROM {table} WHERE {where_clause}"),
            None => format!("SELECT COUNT(*) FROM {table}"),
        }
    }
}

// ---------------------------------------------------------------------------
// 4. Result Mapping
// ---------------------------------------------------------------------------

/// Demonstrates how SQLx maps database rows to Rust types.
/// In real SQLx, this uses the `FromRow` derive macro.
pub trait RowMapper<T> {
    fn map_row(row: &DatabaseRow) -> Result<T, MappingError>;
}

/// A simplified database row representation.
#[derive(Debug, Clone)]
pub struct DatabaseRow {
    columns: Vec<ColumnInfo>,
    values: Vec<DatabaseValue>,
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub type_oid: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseValue {
    Null,
    Bool(bool),
    Int2(i16),
    Int4(i32),
    Int8(i64),
    Float4(f32),
    Float8(f64),
    Text(String),
    Bytes(Vec<u8>),
}

impl DatabaseRow {
    pub fn new(columns: Vec<ColumnInfo>, values: Vec<DatabaseValue>) -> Self {
        Self { columns, values }
    }

    pub fn get<T: FromDatabaseValue>(&self, column_name: &str) -> Result<T, MappingError> {
        let index = self
            .columns
            .iter()
            .position(|c| c.name == column_name)
            .ok_or_else(|| MappingError::ColumnNotFound(column_name.into()))?;

        let value = &self.values[index];
        T::from_db_value(value).map_err(|_| MappingError::TypeMismatch {
            column: column_name.into(),
            expected: std::any::type_name::<T>().to_string(),
            actual: format!("{value:?}"),
        })
    }

    pub fn get_optional<T: FromDatabaseValue>(
        &self,
        column_name: &str,
    ) -> Result<Option<T>, MappingError> {
        let index = self
            .columns
            .iter()
            .position(|c| c.name == column_name)
            .ok_or_else(|| MappingError::ColumnNotFound(column_name.into()))?;

        match &self.values[index] {
            DatabaseValue::Null => Ok(None),
            value => T::from_db_value(value)
                .map(Some)
                .map_err(|_| MappingError::TypeMismatch {
                    column: column_name.into(),
                    expected: std::any::type_name::<T>().to_string(),
                    actual: format!("{value:?}"),
                }),
        }
    }
}

/// Trait for types that can be converted from a database value.
pub trait FromDatabaseValue: Sized {
    fn from_db_value(value: &DatabaseValue) -> Result<Self, ()>;
}

impl FromDatabaseValue for i64 {
    fn from_db_value(value: &DatabaseValue) -> Result<Self, ()> {
        match value {
            DatabaseValue::Int8(v) => Ok(*v),
            DatabaseValue::Int4(v) => Ok(*v as i64),
            DatabaseValue::Int2(v) => Ok(*v as i64),
            _ => Err(()),
        }
    }
}

impl FromDatabaseValue for String {
    fn from_db_value(value: &DatabaseValue) -> Result<Self, ()> {
        match value {
            DatabaseValue::Text(s) => Ok(s.clone()),
            _ => Err(()),
        }
    }
}

impl FromDatabaseValue for bool {
    fn from_db_value(value: &DatabaseValue) -> Result<Self, ()> {
        match value {
            DatabaseValue::Bool(b) => Ok(*b),
            _ => Err(()),
        }
    }
}

impl FromDatabaseValue for f64 {
    fn from_db_value(value: &DatabaseValue) -> Result<Self, ()> {
        match value {
            DatabaseValue::Float8(f) => Ok(*f),
            DatabaseValue::Float4(f) => Ok(*f as f64),
            _ => Err(()),
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Error Types
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("connection error: {0}")]
    Connection(String),

    #[error("query error: {0}")]
    Query(String),

    #[error("no rows returned")]
    RowNotFound,

    #[error("mapping error: {0}")]
    Mapping(#[from] MappingError),

    #[error("constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("timeout")]
    Timeout,
}

#[derive(Debug, thiserror::Error)]
pub enum MappingError {
    #[error("column not found: {0}")]
    ColumnNotFound(String),

    #[error("type mismatch in column '{column}': expected {expected}, got {actual}")]
    TypeMismatch {
        column: String,
        expected: String,
        actual: String,
    },
}

// ---------------------------------------------------------------------------
// 6. Batch Operations
// ---------------------------------------------------------------------------

/// Builder for batch insert operations.
pub struct BatchInsertBuilder {
    table: String,
    columns: Vec<String>,
    rows: Vec<Vec<QueryParam>>,
}

impl BatchInsertBuilder {
    pub fn new(table: impl Into<String>, columns: Vec<String>) -> Self {
        Self {
            table: table.into(),
            columns,
            rows: Vec::new(),
        }
    }

    pub fn add_row(mut self, values: Vec<QueryParam>) -> Self {
        self.rows.push(values);
        self
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Generate the batch INSERT SQL.
    pub fn build_sql(&self) -> String {
        let cols = self.columns.join(", ");
        let mut value_groups = Vec::new();

        for (row_idx, row) in self.rows.iter().enumerate() {
            let placeholders: Vec<String> = (0..row.len())
                .map(|col_idx| {
                    let param_num = row_idx * self.columns.len() + col_idx + 1;
                    format!("${param_num}")
                })
                .collect();
            value_groups.push(format!("({})", placeholders.join(", ")));
        }

        format!(
            "INSERT INTO {} ({cols}) VALUES {}",
            self.table,
            value_groups.join(", ")
        )
    }

    /// Get all parameters in order.
    pub fn all_params(&self) -> Vec<&QueryParam> {
        self.rows.iter().flat_map(|row| row.iter()).collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder_basic() {
        let qb = QueryBuilder::new("SELECT * FROM users WHERE id = $1")
            .bind_int(42);

        assert_eq!(qb.sql(), "SELECT * FROM users WHERE id = $1");
        assert_eq!(qb.param_count(), 1);
    }

    #[test]
    fn test_query_builder_multiple_params() {
        let qb = QueryBuilder::new("INSERT INTO users (name, email, active) VALUES ($1, $2, $3)")
            .bind_text("Alice")
            .bind_text("alice@example.com")
            .bind_bool(true);

        assert_eq!(qb.param_count(), 3);
    }

    #[test]
    fn test_query_param_types() {
        let qb = QueryBuilder::new("SELECT $1, $2, $3, $4, $5")
            .bind_text("hello")
            .bind_int(42)
            .bind_float(3.14)
            .bind_bool(true)
            .bind_null();

        assert_eq!(qb.param_count(), 5);
    }

    #[test]
    fn test_select_paginated() {
        let qb = QueryPatterns::select_paginated("users", &["id", "name", "email"], 2, 10);
        let sql = qb.sql();
        assert!(sql.contains("SELECT id, name, email FROM users"));
        assert!(sql.contains("LIMIT 10"));
        assert!(sql.contains("OFFSET 10"));
    }

    #[test]
    fn test_select_paginated_first_page() {
        let qb = QueryPatterns::select_paginated("items", &["*"], 1, 20);
        assert!(qb.sql().contains("OFFSET 0"));
    }

    #[test]
    fn test_select_where() {
        let (sql, params) =
            QueryPatterns::select_where("users", &["*"], &[("active", "="), ("age", ">")]);
        assert!(sql.contains("WHERE active = $1 AND age > $2"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_insert_query() {
        let sql = QueryPatterns::insert("users", &["name", "email"]);
        assert!(sql.contains("INSERT INTO users (name, email)"));
        assert!(sql.contains("VALUES ($1, $2)"));
        assert!(sql.contains("RETURNING *"));
    }

    #[test]
    fn test_update_query() {
        let sql = QueryPatterns::update("users", &["name", "email"], "id");
        assert!(sql.contains("SET name = $1, email = $2"));
        assert!(sql.contains("WHERE id = $3"));
        assert!(sql.contains("RETURNING *"));
    }

    #[test]
    fn test_delete_query() {
        let sql = QueryPatterns::delete("users", "id");
        assert_eq!(sql, "DELETE FROM users WHERE id = $1");
    }

    #[test]
    fn test_count_query() {
        let sql = QueryPatterns::count("users", None);
        assert_eq!(sql, "SELECT COUNT(*) FROM users");

        let sql = QueryPatterns::count("users", Some("active = true"));
        assert_eq!(sql, "SELECT COUNT(*) FROM users WHERE active = true");
    }

    #[test]
    fn test_row_mapping() {
        let row = DatabaseRow::new(
            vec![
                ColumnInfo {
                    name: "id".into(),
                    type_oid: 20,
                },
                ColumnInfo {
                    name: "name".into(),
                    type_oid: 25,
                },
                ColumnInfo {
                    name: "active".into(),
                    type_oid: 16,
                },
            ],
            vec![
                DatabaseValue::Int8(1),
                DatabaseValue::Text("Alice".into()),
                DatabaseValue::Bool(true),
            ],
        );

        let id: i64 = row.get("id").unwrap();
        let name: String = row.get("name").unwrap();
        let active: bool = row.get("active").unwrap();

        assert_eq!(id, 1);
        assert_eq!(name, "Alice");
        assert!(active);
    }

    #[test]
    fn test_row_mapping_column_not_found() {
        let row = DatabaseRow::new(vec![], vec![]);
        let result: Result<i64, _> = row.get("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_row_mapping_type_mismatch() {
        let row = DatabaseRow::new(
            vec![ColumnInfo {
                name: "val".into(),
                type_oid: 25,
            }],
            vec![DatabaseValue::Text("not a number".into())],
        );

        let result: Result<i64, _> = row.get("val");
        assert!(result.is_err());
    }

    #[test]
    fn test_row_mapping_optional() {
        let row = DatabaseRow::new(
            vec![ColumnInfo {
                name: "val".into(),
                type_oid: 20,
            }],
            vec![DatabaseValue::Null],
        );

        let val: Option<i64> = row.get_optional("val").unwrap();
        assert!(val.is_none());
    }

    #[test]
    fn test_row_mapping_optional_with_value() {
        let row = DatabaseRow::new(
            vec![ColumnInfo {
                name: "val".into(),
                type_oid: 20,
            }],
            vec![DatabaseValue::Int8(42)],
        );

        let val: Option<i64> = row.get_optional("val").unwrap();
        assert_eq!(val, Some(42));
    }

    #[test]
    fn test_from_db_value_i64() {
        assert_eq!(i64::from_db_value(&DatabaseValue::Int8(42)).unwrap(), 42);
        assert_eq!(i64::from_db_value(&DatabaseValue::Int4(10)).unwrap(), 10);
        assert_eq!(i64::from_db_value(&DatabaseValue::Int2(5)).unwrap(), 5);
        assert!(i64::from_db_value(&DatabaseValue::Text("nope".into())).is_err());
    }

    #[test]
    fn test_from_db_value_string() {
        assert_eq!(
            String::from_db_value(&DatabaseValue::Text("hello".into())).unwrap(),
            "hello"
        );
        assert!(String::from_db_value(&DatabaseValue::Int8(1)).is_err());
    }

    #[test]
    fn test_from_db_value_bool() {
        assert!(bool::from_db_value(&DatabaseValue::Bool(true)).unwrap());
        assert!(!bool::from_db_value(&DatabaseValue::Bool(false)).unwrap());
        assert!(bool::from_db_value(&DatabaseValue::Int8(1)).is_err());
    }

    #[test]
    fn test_from_db_value_f64() {
        let val = f64::from_db_value(&DatabaseValue::Float8(3.14)).unwrap();
        assert!((val - 3.14).abs() < f64::EPSILON);

        let val = f64::from_db_value(&DatabaseValue::Float4(2.5)).unwrap();
        assert!((val - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_batch_insert_builder() {
        let builder = BatchInsertBuilder::new(
            "users",
            vec!["name".into(), "email".into()],
        )
        .add_row(vec![
            QueryParam::Text("Alice".into()),
            QueryParam::Text("alice@test.com".into()),
        ])
        .add_row(vec![
            QueryParam::Text("Bob".into()),
            QueryParam::Text("bob@test.com".into()),
        ]);

        assert_eq!(builder.row_count(), 2);

        let sql = builder.build_sql();
        assert!(sql.contains("INSERT INTO users (name, email)"));
        assert!(sql.contains("($1, $2)"));
        assert!(sql.contains("($3, $4)"));

        let params = builder.all_params();
        assert_eq!(params.len(), 4);
    }

    #[test]
    fn test_database_error_display() {
        let err = DatabaseError::Connection("refused".into());
        assert_eq!(err.to_string(), "connection error: refused");

        let err = DatabaseError::RowNotFound;
        assert_eq!(err.to_string(), "no rows returned");

        let err = DatabaseError::ConstraintViolation("unique email".into());
        assert!(err.to_string().contains("unique email"));
    }

    #[test]
    fn test_mapping_error_display() {
        let err = MappingError::ColumnNotFound("missing_col".into());
        assert!(err.to_string().contains("missing_col"));

        let err = MappingError::TypeMismatch {
            column: "age".into(),
            expected: "i64".into(),
            actual: "Text".into(),
        };
        assert!(err.to_string().contains("age"));
    }

    #[test]
    fn test_query_builder_clone() {
        let qb = QueryBuilder::new("SELECT 1").bind_int(42);
        let qb2 = qb.clone();
        assert_eq!(qb.sql(), qb2.sql());
        assert_eq!(qb.param_count(), qb2.param_count());
    }
}
