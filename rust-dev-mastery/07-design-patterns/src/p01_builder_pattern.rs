//! # Builder Pattern
//!
//! The builder pattern constructs complex objects step by step. In Rust, the
//! typestate builder variant ensures at compile time that all required fields
//! are set before the object can be built.
//!
//! ## Key Concepts
//! - **Classic builder**: Optional fields with defaults, `.build()` validates
//! - **Typestate builder**: Type-level encoding of which fields have been set
//! - **Required vs optional**: The type system enforces required fields

/// A classic builder with runtime validation.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
    pub tls_enabled: bool,
    pub timeout_ms: u64,
    pub worker_threads: usize,
}

pub struct ServerConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    max_connections: u32,
    tls_enabled: bool,
    timeout_ms: u64,
    worker_threads: usize,
}

impl ServerConfigBuilder {
    pub fn new() -> Self {
        ServerConfigBuilder {
            host: None,
            port: None,
            max_connections: 100,
            tls_enabled: false,
            timeout_ms: 30000,
            worker_threads: 4,
        }
    }

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    pub fn tls_enabled(mut self, enabled: bool) -> Self {
        self.tls_enabled = enabled;
        self
    }

    pub fn timeout_ms(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    pub fn worker_threads(mut self, threads: usize) -> Self {
        self.worker_threads = threads;
        self
    }

    pub fn build(self) -> Result<ServerConfig, BuilderError> {
        let host = self.host.ok_or(BuilderError::MissingField("host"))?;
        let port = self.port.ok_or(BuilderError::MissingField("port"))?;

        if port == 0 {
            return Err(BuilderError::InvalidValue("port must be > 0"));
        }
        if self.max_connections == 0 {
            return Err(BuilderError::InvalidValue("max_connections must be > 0"));
        }

        Ok(ServerConfig {
            host,
            port,
            max_connections: self.max_connections,
            tls_enabled: self.tls_enabled,
            timeout_ms: self.timeout_ms,
            worker_threads: self.worker_threads,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuilderError {
    MissingField(&'static str),
    InvalidValue(&'static str),
}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuilderError::MissingField(field) => write!(f, "Missing required field: {field}"),
            BuilderError::InvalidValue(msg) => write!(f, "Invalid value: {msg}"),
        }
    }
}

impl std::error::Error for BuilderError {}

/// Typestate builder: uses phantom types to track which fields have been set.
/// The `build()` method is only available when all required fields are set.
pub struct HttpRequest<Host = Missing, Port = Missing> {
    host: String,
    port: u16,
    path: String,
    method: HttpMethod,
    headers: Vec<(String, String)>,
    _host: std::marker::PhantomData<Host>,
    _port: std::marker::PhantomData<Port>,
}

pub struct Missing;
pub struct Set;

#[derive(Debug, Clone)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl Default for HttpRequest {
    fn default() -> Self {
        HttpRequest {
            host: String::new(),
            port: 80,
            path: "/".to_string(),
            method: HttpMethod::Get,
            headers: Vec::new(),
            _host: std::marker::PhantomData,
            _port: std::marker::PhantomData,
        }
    }
}

impl<Host, Port> HttpRequest<Host, Port> {
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }

    pub fn method(mut self, method: HttpMethod) -> Self {
        self.method = method;
        self
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }
}

// host() transitions from Missing -> Set
impl<Port> HttpRequest<Missing, Port> {
    pub fn host(self, host: impl Into<String>) -> HttpRequest<Set, Port> {
        HttpRequest {
            host: host.into(),
            port: self.port,
            path: self.path,
            method: self.method,
            headers: self.headers,
            _host: std::marker::PhantomData,
            _port: std::marker::PhantomData,
        }
    }
}

// port() transitions from Missing -> Set
impl<Host> HttpRequest<Host, Missing> {
    pub fn port(self, port: u16) -> HttpRequest<Host, Set> {
        HttpRequest {
            host: self.host,
            port,
            path: self.path,
            method: self.method,
            headers: self.headers,
            _host: std::marker::PhantomData,
            _port: std::marker::PhantomData,
        }
    }
}

// build() is only available when both Host and Port are Set
impl HttpRequest<Set, Set> {
    pub fn build(self) -> BuiltRequest {
        BuiltRequest {
            host: self.host,
            port: self.port,
            path: self.path,
            method: self.method,
            headers: self.headers,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BuiltRequest {
    pub host: String,
    pub port: u16,
    pub path: String,
    pub method: HttpMethod,
    pub headers: Vec<(String, String)>,
}

/// A fluent builder for SQL queries.
pub struct QueryBuilder {
    table: String,
    columns: Vec<String>,
    conditions: Vec<String>,
    order_by: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

impl QueryBuilder {
    pub fn select(columns: &[&str]) -> Self {
        QueryBuilder {
            table: String::new(),
            columns: columns.iter().map(|s| s.to_string()).collect(),
            conditions: Vec::new(),
            order_by: None,
            limit: None,
            offset: None,
        }
    }

    pub fn from(mut self, table: &str) -> Self {
        self.table = table.to_string();
        self
    }

    pub fn where_clause(mut self, condition: impl Into<String>) -> Self {
        self.conditions.push(condition.into());
        self
    }

    pub fn order_by(mut self, column: &str) -> Self {
        self.order_by = Some(column.to_string());
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn offset(mut self, n: usize) -> Self {
        self.offset = Some(n);
        self
    }

    pub fn build(self) -> String {
        let mut sql = format!("SELECT {} FROM {}", self.columns.join(", "), self.table);

        if !self.conditions.is_empty() {
            sql.push_str(&format!(" WHERE {}", self.conditions.join(" AND ")));
        }
        if let Some(order) = &self.order_by {
            sql.push_str(&format!(" ORDER BY {order}"));
        }
        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {limit}"));
        }
        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {offset}"));
        }

        sql
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_builder_success() {
        let config = ServerConfigBuilder::new()
            .host("localhost")
            .port(8080)
            .max_connections(200)
            .tls_enabled(true)
            .worker_threads(8)
            .build()
            .unwrap();

        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8080);
        assert_eq!(config.max_connections, 200);
        assert!(config.tls_enabled);
        assert_eq!(config.worker_threads, 8);
    }

    #[test]
    fn test_server_config_builder_defaults() {
        let config = ServerConfigBuilder::new()
            .host("localhost")
            .port(8080)
            .build()
            .unwrap();

        assert_eq!(config.max_connections, 100);
        assert!(!config.tls_enabled);
        assert_eq!(config.timeout_ms, 30000);
        assert_eq!(config.worker_threads, 4);
    }

    #[test]
    fn test_server_config_missing_host() {
        let result = ServerConfigBuilder::new().port(8080).build();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("host"));
    }

    #[test]
    fn test_server_config_missing_port() {
        let result = ServerConfigBuilder::new().host("localhost").build();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("port"));
    }

    #[test]
    fn test_server_config_invalid_port() {
        let result = ServerConfigBuilder::new()
            .host("localhost")
            .port(0)
            .build();
        assert!(result.is_err());
    }

    // Typestate builder tests
    #[test]
    fn test_typestate_builder_complete() {
        let req = HttpRequest::default()
            .host("example.com")
            .port(443)
            .path("/api/v1")
            .method(HttpMethod::Post)
            .header("Content-Type", "application/json")
            .build();

        assert_eq!(req.host, "example.com");
        assert_eq!(req.port, 443);
        assert_eq!(req.path, "/api/v1");
    }

    #[test]
    fn test_typestate_builder_order_independent() {
        // Can set port before host
        let req = HttpRequest::default()
            .port(8080)
            .host("localhost")
            .build();

        assert_eq!(req.host, "localhost");
        assert_eq!(req.port, 8080);
    }

    // These would be compile errors:
    // HttpRequest::default().build(); // Missing host AND port
    // HttpRequest::default().host("x").build(); // Missing port
    // HttpRequest::default().port(80).build(); // Missing host

    #[test]
    fn test_query_builder_select_all() {
        let sql = QueryBuilder::select(&["*"]).from("users").build();
        assert_eq!(sql, "SELECT * FROM users");
    }

    #[test]
    fn test_query_builder_with_conditions() {
        let sql = QueryBuilder::select(&["id", "name"])
            .from("users")
            .where_clause("age > 18")
            .where_clause("active = true")
            .order_by("name")
            .limit(10)
            .offset(20)
            .build();

        assert_eq!(
            sql,
            "SELECT id, name FROM users WHERE age > 18 AND active = true ORDER BY name LIMIT 10 OFFSET 20"
        );
    }

    #[test]
    fn test_query_builder_minimal() {
        let sql = QueryBuilder::select(&["count(*)"]).from("orders").build();
        assert_eq!(sql, "SELECT count(*) FROM orders");
    }

    #[test]
    fn test_builder_error_display() {
        let err = BuilderError::MissingField("host");
        assert!(err.to_string().contains("host"));
    }
}
