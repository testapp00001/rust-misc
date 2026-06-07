//! # Builder Pattern for APIs
//!
//! Builders are essential for APIs with many configuration options. They provide
//! a fluent interface, sensible defaults, and can enforce constraints at build time.
//!
//! ## Key Concepts
//! - **Fluent APIs**: Method chaining for readable configuration
//! - **Required vs optional**: Enforce required fields, default optional ones
//! - **Request/Response builders**: Common pattern in HTTP and RPC APIs
//! - **Configuration builders**: Complex service configuration

use std::collections::HashMap;
use std::time::Duration;

/// An HTTP request builder with fluent API.
#[derive(Debug, Clone)]
pub struct RequestBuilder {
    method: Method,
    url: Option<String>,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
    timeout: Option<Duration>,
    follow_redirects: bool,
}

#[derive(Debug, Clone)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub timeout: Option<Duration>,
    pub follow_redirects: bool,
}

impl RequestBuilder {
    pub fn new() -> Self {
        RequestBuilder {
            method: Method::Get,
            url: None,
            headers: HashMap::new(),
            body: None,
            timeout: None,
            follow_redirects: true,
        }
    }

    pub fn get(url: impl Into<String>) -> Self {
        RequestBuilder {
            method: Method::Get,
            url: Some(url.into()),
            headers: HashMap::new(),
            body: None,
            timeout: None,
            follow_redirects: true,
        }
    }

    pub fn post(url: impl Into<String>) -> Self {
        RequestBuilder {
            method: Method::Post,
            url: Some(url.into()),
            headers: HashMap::new(),
            body: None,
            timeout: None,
            follow_redirects: true,
        }
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    pub fn json_body(mut self, json: &str) -> Self {
        self.headers
            .insert("Content-Type".into(), "application/json".into());
        self.body = Some(json.as_bytes().to_vec());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn follow_redirects(mut self, follow: bool) -> Self {
        self.follow_redirects = follow;
        self
    }

    pub fn build(self) -> Result<Request, RequestError> {
        let url = self.url.ok_or(RequestError::MissingUrl)?;
        if url.is_empty() {
            return Err(RequestError::EmptyUrl);
        }

        Ok(Request {
            method: self.method,
            url,
            headers: self.headers,
            body: self.body,
            timeout: self.timeout,
            follow_redirects: self.follow_redirects,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestError {
    MissingUrl,
    EmptyUrl,
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestError::MissingUrl => write!(f, "URL is required"),
            RequestError::EmptyUrl => write!(f, "URL cannot be empty"),
        }
    }
}

/// A query builder for constructing complex database queries.
#[derive(Debug)]
pub struct QueryBuilder {
    select: Vec<String>,
    from: Option<String>,
    joins: Vec<Join>,
    conditions: Vec<String>,
    group_by: Vec<String>,
    having: Option<String>,
    order_by: Vec<OrderBy>,
    limit: Option<usize>,
    offset: Option<usize>,
}

#[derive(Debug)]
struct Join {
    join_type: String,
    table: String,
    on: String,
}

#[derive(Debug)]
struct OrderBy {
    column: String,
    ascending: bool,
}

impl QueryBuilder {
    pub fn select(columns: &[&str]) -> Self {
        QueryBuilder {
            select: columns.iter().map(|s| s.to_string()).collect(),
            from: None,
            joins: Vec::new(),
            conditions: Vec::new(),
            group_by: Vec::new(),
            having: None,
            order_by: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    pub fn from(mut self, table: &str) -> Self {
        self.from = Some(table.to_string());
        self
    }

    pub fn join(mut self, table: &str, on: &str) -> Self {
        self.joins.push(Join {
            join_type: "JOIN".into(),
            table: table.to_string(),
            on: on.to_string(),
        });
        self
    }

    pub fn left_join(mut self, table: &str, on: &str) -> Self {
        self.joins.push(Join {
            join_type: "LEFT JOIN".into(),
            table: table.to_string(),
            on: on.to_string(),
        });
        self
    }

    pub fn where_clause(mut self, condition: impl Into<String>) -> Self {
        self.conditions.push(condition.into());
        self
    }

    pub fn group_by(mut self, columns: &[&str]) -> Self {
        self.group_by.extend(columns.iter().map(|s| s.to_string()));
        self
    }

    pub fn having(mut self, condition: impl Into<String>) -> Self {
        self.having = Some(condition.into());
        self
    }

    pub fn order_by(mut self, column: &str, ascending: bool) -> Self {
        self.order_by.push(OrderBy {
            column: column.to_string(),
            ascending,
        });
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
        let mut sql = format!("SELECT {}", self.select.join(", "));

        if let Some(table) = &self.from {
            sql.push_str(&format!(" FROM {table}"));
        }

        for join in &self.joins {
            sql.push_str(&format!(" {} {} ON {}", join.join_type, join.table, join.on));
        }

        if !self.conditions.is_empty() {
            sql.push_str(&format!(" WHERE {}", self.conditions.join(" AND ")));
        }

        if !self.group_by.is_empty() {
            sql.push_str(&format!(" GROUP BY {}", self.group_by.join(", ")));
        }

        if let Some(having) = &self.having {
            sql.push_str(&format!(" HAVING {having}"));
        }

        if !self.order_by.is_empty() {
            let orders: Vec<String> = self
                .order_by
                .iter()
                .map(|o| {
                    format!(
                        "{} {}",
                        o.column,
                        if o.ascending { "ASC" } else { "DESC" }
                    )
                })
                .collect();
            sql.push_str(&format!(" ORDER BY {}", orders.join(", ")));
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

/// A test case builder for constructing test scenarios.
pub struct TestCaseBuilder {
    name: String,
    actions: Vec<String>,
    assertions: Vec<String>,
}

impl TestCaseBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        TestCaseBuilder {
            name: name.into(),
            actions: Vec::new(),
            assertions: Vec::new(),
        }
    }

    pub fn action(mut self, action: impl Into<String>) -> Self {
        self.actions.push(action.into());
        self
    }

    pub fn assert(mut self, assertion: impl Into<String>) -> Self {
        self.assertions.push(assertion.into());
        self
    }

    pub fn description(&self) -> String {
        format!(
            "Test: {}\nActions: {}\nAssertions: {}",
            self.name,
            self.actions.join(" -> "),
            self.assertions.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_builder_get() {
        let req = RequestBuilder::get("https://api.example.com")
            .header("Accept", "application/json")
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        assert_eq!(req.url, "https://api.example.com");
        assert_eq!(req.headers.get("Accept"), Some(&"application/json".to_string()));
    }

    #[test]
    fn test_request_builder_post_json() {
        let req = RequestBuilder::post("https://api.example.com/users")
            .json_body(r#"{"name": "Alice"}"#)
            .build()
            .unwrap();

        assert!(req.body.is_some());
        assert_eq!(
            req.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_request_builder_missing_url() {
        let result = RequestBuilder::new().build();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("required"));
    }

    #[test]
    fn test_request_builder_empty_url() {
        let result = RequestBuilder::new().url("").build();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_request_builder_chaining() {
        let req = RequestBuilder::new()
            .url("https://api.example.com")
            .header("X-Custom", "value")
            .follow_redirects(false)
            .timeout(Duration::from_millis(500))
            .build()
            .unwrap();

        assert!(!req.follow_redirects);
        assert_eq!(req.timeout, Some(Duration::from_millis(500)));
    }

    #[test]
    fn test_query_builder_simple() {
        let sql = QueryBuilder::select(&["*"]).from("users").build();
        assert_eq!(sql, "SELECT * FROM users");
    }

    #[test]
    fn test_query_builder_complex() {
        let sql = QueryBuilder::select(&["u.name", "COUNT(o.id) as order_count"])
            .from("users u")
            .join("orders o", "u.id = o.user_id")
            .where_clause("u.active = true")
            .where_clause("o.created_at > '2024-01-01'")
            .group_by(&["u.name"])
            .having("COUNT(o.id) > 5")
            .order_by("order_count", false)
            .limit(10)
            .offset(20)
            .build();

        assert!(sql.contains("SELECT u.name, COUNT(o.id) as order_count"));
        assert!(sql.contains("FROM users u"));
        assert!(sql.contains("JOIN orders o ON u.id = o.user_id"));
        assert!(sql.contains("WHERE u.active = true AND o.created_at > '2024-01-01'"));
        assert!(sql.contains("GROUP BY u.name"));
        assert!(sql.contains("HAVING COUNT(o.id) > 5"));
        assert!(sql.contains("ORDER BY order_count DESC"));
        assert!(sql.contains("LIMIT 10"));
        assert!(sql.contains("OFFSET 20"));
    }

    #[test]
    fn test_query_builder_left_join() {
        let sql = QueryBuilder::select(&["u.name", "p.bio"])
            .from("users u")
            .left_join("profiles p", "u.id = p.user_id")
            .build();

        assert!(sql.contains("LEFT JOIN profiles p ON u.id = p.user_id"));
    }

    #[test]
    fn test_query_builder_order_asc() {
        let sql = QueryBuilder::select(&["*"])
            .from("items")
            .order_by("name", true)
            .build();

        assert!(sql.contains("ORDER BY name ASC"));
    }

    #[test]
    fn test_test_case_builder() {
        let test = TestCaseBuilder::new("user login")
            .action("POST /login with valid credentials")
            .action("Verify response status is 200")
            .assert("Token is returned")
            .assert("User object is valid");

        let desc = test.description();
        assert!(desc.contains("user login"));
        assert!(desc.contains("POST /login"));
        assert!(desc.contains("Token is returned"));
    }
}
