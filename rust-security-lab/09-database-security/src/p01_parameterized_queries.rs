//! # Lesson 01: Parameterized Queries
//!
//! ## What Are Parameterized Queries?
//!
//! Parameterized queries (also called prepared statements) separate SQL code from
//! data. Instead of concatenating user input into a SQL string, you send the query
//! template and the parameters separately to the database engine, which never
//! interprets the parameters as SQL.
//!
//! ## Why They Matter
//!
//! SQL injection has been the #1 or #2 web vulnerability for over 20 years (OWASP
//! Top 10). Every time you see code like:
//!
//! ```text
//! let query = format!("SELECT * FROM users WHERE name = '{}'", user_input);
//! ```
//!
//! ...an attacker can send `'; DROP TABLE users; --` as `user_input` and destroy
//! your database. Parameterized queries make this impossible because the database
//! engine treats the parameter as a literal value, never as executable SQL.
//!
//! ## Attack vs. Defend
//!
//! ```text
//! ATTACK:  "alice' OR '1'='1"  →  bypasses authentication
//! DEFEND:  $1 parameter        →  treated as literal string "alice' OR '1'='1"
//! ```
//!
//! ## This Module
//!
//! We simulate a SQL database in memory and demonstrate both the vulnerable
//! (string-concatenation) and safe (parameterized) approaches. The functions
//! build SQL strings — the key difference is whether user input is escaped.

use std::collections::HashMap;

/// A simple in-memory table for educational purposes.
/// Each row is a map of column name to string value.
pub type Table = Vec<HashMap<String, String>>;

/// Build a SELECT query using string concatenation (VULNERABLE).
///
/// This is the anti-pattern. User input is directly interpolated into the SQL string.
/// If `username` contains SQL metacharacters, the query semantics change.
///
/// # Attack Example
/// ```
/// // Normal: SELECT * FROM users WHERE username = 'alice'
/// // Attack: SELECT * FROM users WHERE username = '' OR '1'='1'
/// ```
///
/// Exercise: Implement this by directly formatting the username into the query string.
///
/// Hints:
/// - Use `format!()` to build the query
/// - Do NOT escape or sanitize the input — that's the whole point of this exercise
pub fn build_query_unsafe(username: &str) -> String {
    todo!("Build a SELECT query by concatenating username directly into SQL")
}

/// Build a SELECT query using parameter placeholders (SAFE).
///
/// The query uses `$1` as a placeholder. The actual value is returned separately
/// so the database engine can bind it safely.
///
/// Exercise: Return a tuple of (query_template, parameters).
///
/// Hints:
/// - The query should contain `$1` where the username goes
/// - Return `(query_string, vec![username.to_string()])`
pub fn build_query_safe(username: &str) -> (String, Vec<String>) {
    todo!("Build a parameterized SELECT query with $1 placeholder")
}

/// Execute a "query" against an in-memory table (simulated).
///
/// This function simulates a database engine that interprets SQL poorly — it does
/// basic string matching on the WHERE clause. The point is to show how string
/// concatenation lets attackers change query logic.
///
/// For the unsafe path, this function parses the WHERE clause from the query string.
/// For the safe path, it uses the parameter directly.
///
/// Exercise: Implement the safe execution path.
///
/// Hints:
/// - When `params` is non-empty, use `params[0]` as the literal search value
/// - Filter rows where the `username` column exactly equals the parameter
pub fn execute_query_safe(table: &Table, _query: &str, params: &[String]) -> Table {
    todo!("Filter table rows where username equals params[0]")
}

/// Simulate a login check using the UNSAFE approach.
///
/// Returns true if any row matches — but because the query is built via string
/// concatenation, an attacker can craft input that always matches.
///
/// Exercise: Build the unsafe query and "execute" it against the table.
///
/// Hints:
/// - Call `build_query_unsafe` with the username
/// - Parse the resulting SQL to extract the WHERE value (or simulate matching)
/// - This function should be exploitable with `' OR '1'='1`
pub fn login_unsafe(table: &Table, username: &str, password: &str) -> bool {
    todo!("Simulate login with string-concatenated SQL (vulnerable)")
}

/// Simulate a login check using the SAFE approach.
///
/// Returns true only if an exact match is found for both username and password.
///
/// Exercise: Build the safe query and execute it.
///
/// Hints:
/// - Call `build_query_safe` with the username
/// - Use `execute_query_safe` with the table, query, and params
/// - Check that the password also matches the returned row
pub fn login_safe(table: &Table, username: &str, password: &str) -> bool {
    todo!("Simulate login with parameterized query (safe)")
}

/// Demonstrate the SQL injection attack on the unsafe login.
///
/// Exercise: Create a table with real users, then show that the attack string
/// `' OR '1'='1` bypasses authentication on the unsafe path but not the safe path.
///
/// Hints:
/// - Build a table with at least one user (e.g., "alice" / "password123")
/// - Try `login_unsafe` with the attack string — should return true
/// - Try `login_safe` with the attack string — should return false
pub fn demonstrate_injection_attack() -> (bool, bool) {
    todo!("Return (unsafe_login_result, safe_login_result) for injection attack")
}

/// Simulate a UNION injection attack.
///
/// In a real UNION injection, the attacker appends:
///   ' UNION SELECT username, password FROM admin_users --
///
/// This exercise builds the malicious query string and shows how it differs
/// from the safe version.
///
/// Exercise: Build both the unsafe and safe versions of a query where the
/// attacker tries to inject a UNION SELECT.
///
/// Hints:
/// - Unsafe: directly concatenate the malicious input
/// - Safe: the UNION keywords become part of the literal string parameter
pub fn demonstrate_union_injection(search_term: &str) -> (String, String) {
    todo!("Return (unsafe_query, safe_query) showing UNION injection difference")
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
        // The unsafe login should be bypassed by the injection
        assert!(unsafe_result, "Unsafe login should be bypassed by injection");
        // The safe login should block the injection
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
