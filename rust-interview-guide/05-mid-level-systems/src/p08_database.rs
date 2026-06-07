/// Problem: Database
///
/// Master database operations in Rust.
///
/// Key Concepts:
/// - Database connections
/// - Query execution
/// - Transactions
/// - Connection pooling
/// - ORM patterns

use std::collections::HashMap;

/// Problem 1: Database connection (simulated)
/// Simulate database connection
#[derive(Debug)]
pub struct Connection {
    pub url: String,
    pub connected: bool,
}

impl Connection {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            connected: false,
        }
    }

    pub fn connect(&mut self) -> Result<(), String> {
        self.connected = true;
        Ok(())
    }

    pub fn disconnect(&mut self) {
        self.connected = false;
    }
}

/// Problem 2: Query execution (simulated)
/// Simulate query execution
#[derive(Debug)]
pub struct QueryResult {
    pub rows: Vec<HashMap<String, String>>,
    pub affected_rows: usize,
}

pub fn execute_query(query: &str) -> QueryResult {
    let mut rows = Vec::new();
    let mut row = HashMap::new();
    row.insert("id".to_string(), "1".to_string());
    row.insert("name".to_string(), "Alice".to_string());
    row.insert("email".to_string(), "alice@example.com".to_string());
    rows.push(row);
    QueryResult {
        rows,
        affected_rows: 1,
    }
}

/// Problem 3: Transaction (simulated)
/// Simulate transaction
pub struct Transaction {
    pub active: bool,
}

impl Transaction {
    pub fn new() -> Self {
        Self { active: true }
    }

    pub fn commit(&mut self) {
        self.active = false;
    }

    pub fn rollback(&mut self) {
        self.active = false;
    }
}

/// Problem 4: Connection pool (simulated)
/// Simulate connection pool
pub struct ConnectionPool {
    pub connections: Vec<Connection>,
    pub max_size: usize,
}

impl ConnectionPool {
    pub fn new(max_size: usize) -> Self {
        Self {
            connections: Vec::new(),
            max_size,
        }
    }

    pub fn get_connection(&mut self) -> Option<&Connection> {
        if self.connections.len() < self.max_size {
            let conn = Connection::new("postgres://localhost");
            self.connections.push(conn);
            self.connections.last()
        } else {
            self.connections.first()
        }
    }
}

/// Problem 5: Query builder (simulated)
/// Build queries
pub struct QueryBuilder {
    pub table: String,
    pub conditions: Vec<String>,
    pub limit: Option<usize>,
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

    pub fn build(&self) -> String {
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

/// Problem 6: ORM pattern (simulated)
/// Simulate ORM
#[derive(Debug, Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}

impl User {
    pub fn new(id: u32, name: &str, email: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            email: email.to_string(),
        }
    }

    pub fn to_row(&self) -> HashMap<String, String> {
        let mut row = HashMap::new();
        row.insert("id".to_string(), self.id.to_string());
        row.insert("name".to_string(), self.name.clone());
        row.insert("email".to_string(), self.email.clone());
        row
    }

    pub fn from_row(row: &HashMap<String, String>) -> Option<Self> {
        Some(Self {
            id: row.get("id")?.parse().ok()?,
            name: row.get("name")?.clone(),
            email: row.get("email")?.clone(),
        })
    }
}

/// Problem 7: Migration (simulated)
/// Simulate migration
pub struct Migration {
    pub version: u32,
    pub description: String,
    pub up_sql: String,
    pub down_sql: String,
}

impl Migration {
    pub fn new(version: u32, description: &str, up_sql: &str, down_sql: &str) -> Self {
        Self {
            version,
            description: description.to_string(),
            up_sql: up_sql.to_string(),
            down_sql: down_sql.to_string(),
        }
    }
}

/// Problem 8: Database schema (simulated)
/// Define schema
pub struct Schema {
    pub tables: Vec<Table>,
}

pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
}

pub struct Column {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

impl Schema {
    pub fn new() -> Self {
        Self { tables: Vec::new() }
    }

    pub fn add_table(&mut self, table: Table) {
        self.tables.push(table);
    }
}

/// Problem 9: Prepared statement (simulated)
/// Simulate prepared statement
pub struct PreparedStatement {
    pub query: String,
    pub params: Vec<String>,
}

impl PreparedStatement {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
            params: Vec::new(),
        }
    }

    pub fn bind(&mut self, param: &str) {
        self.params.push(param.to_string());
    }

    pub fn execute(&self) -> QueryResult {
        execute_query(&self.query)
    }
}

/// Problem 10: Database error handling
/// Handle database errors
#[derive(Debug)]
pub enum DatabaseError {
    ConnectionError(String),
    QueryError(String),
    TransactionError(String),
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            DatabaseError::QueryError(msg) => write!(f, "Query error: {}", msg),
            DatabaseError::TransactionError(msg) => write!(f, "Transaction error: {}", msg),
        }
    }
}

/// Problem 11: Database result mapping
/// Map query results
pub fn map_results(result: &QueryResult) -> Vec<User> {
    result.rows.iter().filter_map(|row| User::from_row(row)).collect()
}

/// Problem 12: Database pagination
/// Implement pagination
pub fn paginate(query: &str, page: usize, per_page: usize) -> String {
    let offset = (page - 1) * per_page;
    format!("{} LIMIT {} OFFSET {}", query, per_page, offset)
}

/// Problem 13: Database indexing (simulated)
/// Simulate indexing
pub struct Index {
    pub name: String,
    pub table: String,
    pub columns: Vec<String>,
}

impl Index {
    pub fn new(name: &str, table: &str, columns: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            table: table.to_string(),
            columns,
        }
    }

    pub fn create_sql(&self) -> String {
        format!(
            "CREATE INDEX {} ON {} ({})",
            self.name,
            self.table,
            self.columns.join(", ")
        )
    }
}

/// Problem 14: Database backup (simulated)
/// Simulate backup
pub fn backup_database(url: &str) -> String {
    format!("Backing up database: {}", url)
}

/// Problem 15: Database health check
/// Check database health
pub fn health_check(url: &str) -> bool {
    // In real implementation, you'd ping the database
    url.starts_with("postgres://") || url.starts_with("mysql://")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection() {
        let mut conn = Connection::new("postgres://localhost");
        assert!(!conn.connected);
        conn.connect().unwrap();
        assert!(conn.connected);
    }

    #[test]
    fn test_execute_query() {
        let result = execute_query("SELECT * FROM users");
        assert_eq!(result.rows.len(), 1);
    }

    #[test]
    fn test_transaction() {
        let mut tx = Transaction::new();
        assert!(tx.active);
        tx.commit();
        assert!(!tx.active);
    }

    #[test]
    fn test_connection_pool() {
        let mut pool = ConnectionPool::new(5);
        let conn = pool.get_connection();
        assert!(conn.is_some());
    }

    #[test]
    fn test_query_builder() {
        let query = QueryBuilder::new("users")
            .where_clause("age > 18")
            .limit(10)
            .build();
        assert!(query.contains("users"));
        assert!(query.contains("age > 18"));
    }

    #[test]
    fn test_user_orm() {
        let user = User::new(1, "Alice", "alice@example.com");
        let row = user.to_row();
        assert_eq!(row.get("name"), Some(&"Alice".to_string()));

        let user2 = User::from_row(&row).unwrap();
        assert_eq!(user2.name, "Alice");
    }

    #[test]
    fn test_migration() {
        let migration = Migration::new(
            1,
            "Create users table",
            "CREATE TABLE users (id INT, name VARCHAR)",
            "DROP TABLE users",
        );
        assert_eq!(migration.version, 1);
    }

    #[test]
    fn test_schema() {
        let mut schema = Schema::new();
        schema.add_table(Table {
            name: "users".to_string(),
            columns: vec![
                Column {
                    name: "id".to_string(),
                    data_type: "INT".to_string(),
                    nullable: false,
                },
            ],
        });
        assert_eq!(schema.tables.len(), 1);
    }

    #[test]
    fn test_prepared_statement() {
        let mut stmt = PreparedStatement::new("SELECT * FROM users WHERE id = ?");
        stmt.bind("1");
        let result = stmt.execute();
        assert_eq!(result.rows.len(), 1);
    }

    #[test]
    fn test_database_error() {
        let error = DatabaseError::ConnectionError("Failed to connect".to_string());
        assert!(error.to_string().contains("Connection error"));
    }

    #[test]
    fn test_map_results() {
        let result = execute_query("SELECT * FROM users");
        let users = map_results(&result);
        assert_eq!(users.len(), 1);
    }

    #[test]
    fn test_paginate() {
        let query = paginate("SELECT * FROM users", 2, 10);
        assert!(query.contains("LIMIT 10"));
        assert!(query.contains("OFFSET 10"));
    }

    #[test]
    fn test_index() {
        let index = Index::new("idx_users_name", "users", vec!["name".to_string()]);
        assert!(index.create_sql().contains("idx_users_name"));
    }

    #[test]
    fn test_backup_database() {
        let result = backup_database("postgres://localhost");
        assert!(result.contains("Backing up"));
    }

    #[test]
    fn test_health_check() {
        assert!(health_check("postgres://localhost"));
        assert!(!health_check("invalid://url"));
    }
}
