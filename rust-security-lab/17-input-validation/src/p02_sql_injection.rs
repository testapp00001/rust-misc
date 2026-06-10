//! # Lesson 02: SQL Injection
//!
//! ## The Problem
//!
//! SQL injection occurs when user input is concatenated directly into SQL
//! queries without proper escaping or parameterization. It has been the
//! #1 web vulnerability for over two decades.
//!
//! ```ignore
//! // VULNERABLE: String concatenation
//! let query = format!("SELECT * FROM users WHERE name = '{}'", user_input);
//!
//! // If user_input is:  ' OR '1'='1' --
//! // The query becomes: SELECT * FROM users WHERE name = '' OR '1'='1' --'
//! // This returns ALL users!
//! ```
//!
//! ## Attack Techniques
//!
//! 1. **Classic injection**: `' OR '1'='1' --` to bypass authentication
//! 2. **UNION injection**: `UNION SELECT password FROM users--` to exfiltrate data
//! 3. **Blind injection**: `' AND (SELECT CASE WHEN (1=1) THEN 1 ELSE 0 END)--`
//!    to extract data bit by bit
//! 4. **Stacked queries**: `; DROP TABLE users;--` to destroy data
//!
//! ## Defense: Parameterized Queries
//!
//! The only reliable defense is to **never** interpolate user input into SQL.
//! Instead, use parameterized queries (also called prepared statements) where
//! the database engine handles escaping:
//!
//! ```ignore
//! // SAFE: Parameterized query
//! let query = "SELECT * FROM users WHERE name = ?";
//! db.execute(query, &[&user_input]);
//! ```
//!
//! ## In This Exercise
//!
//! We simulate parameterized query behavior in pure Rust. You will build a
//! query builder that separates SQL structure from user values, and a
//! sanitizer that rejects dangerous patterns.

/// Represents a SQL value that can be safely embedded.
#[derive(Debug, Clone, PartialEq)]
pub enum SqlValue {
    Null,
    Integer(i64),
    Text(String),
    Boolean(bool),
}

impl SqlValue {
    /// Convert the value to its SQL literal representation.
    ///
    /// For Text values, this must properly escape single quotes by doubling them.
    /// For example, `O'Brien` becomes `'O''Brien'`.
    pub fn to_sql_literal(&self) -> String {
        todo!("Implement SQL literal conversion with proper escaping")
    }
}

/// A parameterized SQL query builder that separates structure from values.
///
/// # Example
/// ```ignore
/// let (query, params) = QueryBuilder::select("users")
///     .column("*")
///     .where_clause("name", "=", SqlValue::Text("alice".into()))
///     .build();
/// // query:  "SELECT * FROM users WHERE name = ?"
/// // params: [SqlValue::Text("alice")]
/// ```
pub struct QueryBuilder {
    table: String,
    columns: Vec<String>,
    conditions: Vec<(String, String, SqlValue)>,
    limit: Option<usize>,
}

impl QueryBuilder {
    /// Create a new SELECT query builder for the given table.
    ///
    /// Table name must be alphanumeric (with underscores) -- reject anything else.
    pub fn select(table: &str) -> Result<Self, String> {
        todo!("Create QueryBuilder, validating the table name")
    }

    /// Add a column to select. Column names must be alphanumeric or `*`.
    pub fn column(mut self, name: &str) -> Result<Self, String> {
        todo!("Validate and add column name")
    }

    /// Add a WHERE condition.
    ///
    /// The column name is validated (alphanumeric/underscore only).
    /// The operator must be one of: =, !=, <, >, <=, >=, LIKE
    /// The value is stored as a parameter (never interpolated).
    pub fn where_clause(
        mut self,
        column: &str,
        operator: &str,
        value: SqlValue,
    ) -> Result<Self, String> {
        todo!("Validate column and operator, store value as parameter")
    }

    /// Add a LIMIT clause.
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    /// Build the final query string and parameter list.
    ///
    /// Returns (query_string, parameters).
    pub fn build(self) -> (String, Vec<SqlValue>) {
        todo!("Assemble the SQL query with ? placeholders and collect params")
    }
}

/// Detect potential SQL injection in user input.
///
/// Returns `true` if the input contains common SQL injection patterns.
/// This is a DENYLIST approach -- it is NOT sufficient as the sole defense,
/// but is useful as an additional layer.
///
/// Patterns to detect (case-insensitive):
/// - SQL keywords: `SELECT`, `INSERT`, `UPDATE`, `DELETE`, `DROP`, `UNION`,
///   `WHERE`, `OR`, `AND`, `EXEC`, `EXECUTE`
/// - SQL comment sequences: `--`, `/*`
/// - Stacked query separator: `;`
/// - Common injection strings: `' OR '`, `" OR "`, `' AND '`
pub fn detect_sql_injection(input: &str) -> bool {
    todo!("Check input against SQL injection patterns")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_value_escape_quotes() {
        let val = SqlValue::Text("O'Brien".to_string());
        assert_eq!(val.to_sql_literal(), "'O''Brien'");
    }

    #[test]
    fn test_sql_value_null() {
        assert_eq!(SqlValue::Null.to_sql_literal(), "NULL");
    }

    #[test]
    fn test_sql_value_integer() {
        assert_eq!(SqlValue::Integer(42).to_sql_literal(), "42");
    }

    #[test]
    fn test_query_builder_simple() {
        let (query, params) = QueryBuilder::select("users")
            .unwrap()
            .column("*")
            .unwrap()
            .where_clause("name", "=", SqlValue::Text("alice".into()))
            .unwrap()
            .build();
        assert!(query.contains("SELECT * FROM users"));
        assert!(query.contains("WHERE name = ?"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_query_builder_rejects_bad_table_name() {
        assert!(QueryBuilder::select("users; DROP TABLE users--").is_err());
    }

    #[test]
    fn test_query_builder_rejects_bad_operator() {
        let result = QueryBuilder::select("users")
            .unwrap()
            .where_clause("name", "; DROP", SqlValue::Text("x".into()));
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_injection_union() {
        assert!(detect_sql_injection("' UNION SELECT * FROM passwords--"));
    }

    #[test]
    fn test_detect_injection_or_true() {
        assert!(detect_sql_injection("' OR '1'='1'"));
    }

    #[test]
    fn test_detect_injection_clean_input() {
        assert!(!detect_sql_injection("alice@example.com"));
    }
}
