//! # Lesson 01: Parameterized Queries (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::HashMap;

pub type Table = Vec<HashMap<String, String>>;

/// Build a SELECT query by string concatenation (VULNERABLE — anti-pattern).
pub fn build_query_unsafe(username: &str) -> String {
    format!("SELECT * FROM users WHERE username = '{}'", username)
}

/// Build a parameterized SELECT query (SAFE).
pub fn build_query_safe(username: &str) -> (String, Vec<String>) {
    (
        "SELECT * FROM users WHERE username = $1".to_string(),
        vec![username.to_string()],
    )
}

/// Execute a safe query against an in-memory table.
pub fn execute_query_safe(table: &Table, _query: &str, params: &[String]) -> Table {
    if params.is_empty() {
        return Vec::new();
    }
    let search_value = &params[0];
    table
        .iter()
        .filter(|row| row.get("username").map_or(false, |v| v == search_value))
        .cloned()
        .collect()
}

/// Simulate login with string-concatenated SQL (VULNERABLE).
///
/// Because the query is built via string concatenation, an attacker can craft
/// input like `' OR '1'='1` to bypass the WHERE clause.
pub fn login_unsafe(table: &Table, username: &str, password: &str) -> bool {
    // Simulate what a naive DB engine would do with the concatenated query.
    // The raw query becomes:
    //   SELECT * FROM users WHERE username = '' OR '1'='1'
    //
    // A real SQL parser would interpret the OR clause and match all rows.
    // We simulate this by checking if the input contains injection patterns.

    // Check if the injected username creates an always-true condition
    let upper = username.to_uppercase();
    if upper.contains("OR") && (upper.contains("1'='1") || upper.contains("1=1")) {
        // The injection bypasses the WHERE clause — match any row that has the password
        // (the attacker doesn't know the password, but the injection makes it irrelevant)
        return table.iter().any(|row| {
            row.get("password").is_some()
        });
    }

    // Normal case: exact match
    table.iter().any(|row| {
        row.get("username").map_or(false, |u| u == username)
            && row.get("password").map_or(false, |p| p == password)
    })
}

/// Simulate login with parameterized query (SAFE).
pub fn login_safe(table: &Table, username: &str, password: &str) -> bool {
    let (query, params) = build_query_safe(username);
    let results = execute_query_safe(table, &query, &params);

    results
        .iter()
        .any(|row| row.get("password").map_or(false, |p| p == password))
}

/// Demonstrate the SQL injection attack.
pub fn demonstrate_injection_attack() -> (bool, bool) {
    let mut table = Vec::new();
    let mut row = HashMap::new();
    row.insert("username".to_string(), "alice".to_string());
    row.insert("password".to_string(), "correcthorse".to_string());
    table.push(row);

    let attack_username = "' OR '1'='1";

    let unsafe_result = login_unsafe(&table, attack_username, "anything");
    let safe_result = login_safe(&table, attack_username, "anything");

    (unsafe_result, safe_result)
}

/// Demonstrate UNION injection difference between unsafe and safe queries.
pub fn demonstrate_union_injection(search_term: &str) -> (String, String) {
    let unsafe_query = format!(
        "SELECT id, name FROM products WHERE name = '{}'",
        search_term
    );
    let (safe_query, _params) = build_query_safe(search_term);
    (unsafe_query, safe_query)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_table() -> Table {
        let mut table = Vec::new();
        let mut row1 = HashMap::new();
        row1.insert("username".to_string(), "alice".to_string());
        row1.insert("password".to_string(), "correcthorse".to_string());
        row1.insert("role".to_string(), "user".to_string());
        table.push(row1);

        let mut row2 = HashMap::new();
        row2.insert("username".to_string(), "bob".to_string());
        row2.insert("password".to_string(), "batterystaple".to_string());
        row2.insert("role".to_string(), "admin".to_string());
        table.push(row2);

        table
    }

    #[test]
    fn test_safe_query_uses_placeholder() {
        let (query, params) = build_query_safe("alice");
        assert!(query.contains("$1"), "Query must use $1 placeholder");
        assert_eq!(params, vec!["alice".to_string()]);
    }

    #[test]
    fn test_safe_query_no_direct_input() {
        let (query, _params) = build_query_safe("alice");
        assert!(
            !query.contains("alice"),
            "Safe query must not contain raw user input"
        );
    }

    #[test]
    fn test_unsafe_query_contains_input() {
        let query = build_query_unsafe("alice");
        assert!(
            query.contains("alice"),
            "Unsafe query directly includes user input"
        );
    }

    #[test]
    fn test_execute_safe_finds_match() {
        let table = sample_table();
        let params = vec!["alice".to_string()];
        let results = execute_query_safe(&table, "SELECT * FROM users WHERE username = $1", &params);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["username"], "alice");
    }

    #[test]
    fn test_execute_safe_no_match() {
        let table = sample_table();
        let params = vec!["eve".to_string()];
        let results = execute_query_safe(&table, "SELECT * FROM users WHERE username = $1", &params);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_login_safe_works() {
        let table = sample_table();
        assert!(login_safe(&table, "alice", "correcthorse"));
        assert!(!login_safe(&table, "alice", "wrongpassword"));
        assert!(!login_safe(&table, "nonexistent", "password"));
    }

    #[test]
    fn test_injection_attack_demonstration() {
        let (unsafe_result, safe_result) = demonstrate_injection_attack();
        assert!(unsafe_result, "Unsafe login should be bypassed by injection");
        assert!(!safe_result, "Safe login must block injection attempt");
    }

    #[test]
    fn test_union_injection_differs() {
        let malicious = "' UNION SELECT username, password FROM admin_users --";
        let (unsafe_q, safe_q) = demonstrate_union_injection(malicious);
        assert_ne!(unsafe_q, safe_q, "Unsafe and safe queries must differ");
        assert!(
            safe_q.contains("$1"),
            "Safe query must use parameter placeholder"
        );
    }
}
