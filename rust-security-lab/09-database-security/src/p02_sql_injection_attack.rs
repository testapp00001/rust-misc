//! # Lesson 02: SQL Injection Attack Demonstrations
//!
//! ## Understanding the Enemy
//!
//! You cannot defend against SQL injection if you do not understand how it works.
//! This lesson walks through the three most common injection patterns:
//!
//! ### 1. Authentication Bypass
//! ```text
//! Input:  ' OR '1'='1' --
//! Query:  SELECT * FROM users WHERE username = '' OR '1'='1' --' AND password = '...'
//! Result: Returns all rows (every user) because '1'='1' is always true
//! ```
//!
//! ### 2. UNION Injection
//! ```text
//! Input:  ' UNION SELECT username, password FROM admin_users --
//! Query:  SELECT id, name FROM products WHERE name = '' UNION SELECT username, password FROM admin_users --'
//! Result: Leaks the entire admin_users table through the products query
//! ```
//!
//! ### 3. Blind Injection (Boolean-based)
//! ```text
//! Input:  ' AND (SELECT SUBSTRING(password,1,1) FROM users WHERE username='admin') = 'a
//! Query:  SELECT * FROM posts WHERE id = '' AND (SELECT SUBSTRING(password,1,1) ...) = 'a'
//! Result: Page loads normally if first char of admin password is 'a', error otherwise
//! ```
//!
//! ## Why Rust Helps (But Doesn't Solve Everything)
//!
//! Rust's type system prevents many injection patterns — you cannot accidentally
//! pass a `String` where an `i32` is expected. But string-based queries are still
//! dangerous. The real defense is parameterized queries.

/// Simulate an authentication bypass attack.
///
/// Returns the attack string that would bypass a vulnerable login query like:
///   SELECT * FROM users WHERE username = '{input}' AND password = '{password}'
///
/// Exercise: Return the classic authentication bypass string.
///
/// Hints:
/// - The classic payload is: `' OR '1'='1' --`
/// - The `--` comments out the rest of the query (the password check)
/// - The `'1'='1'` is always true, so the WHERE clause matches every row
pub fn auth_bypass_payload() -> String {
    todo!("Return the SQL injection auth bypass string")
}

/// Simulate a UNION injection attack.
///
/// The attacker knows (or guesses) that the original query returns 2 columns.
/// They inject a UNION SELECT to pull data from a different table.
///
/// Exercise: Build the UNION injection payload for the given column count.
///
/// Hints:
/// - Start with `' UNION SELECT ` followed by `column_count` values
/// - For the secret table columns, use column names like `username`, `password`
/// - End with ` FROM secret_table --`
pub fn union_injection_payload(column_count: usize) -> String {
    todo!("Build a UNION injection payload for the given column count")
}

/// Simulate a blind boolean-based injection.
///
/// The attacker extracts data one character at a time by asking yes/no questions.
/// This function builds the payload that checks if a specific character of a
/// secret value matches an expected value.
///
/// Exercise: Build the blind injection payload.
///
/// Hints:
/// - Use: `' AND (SELECT SUBSTRING({column},1,1) FROM {table} WHERE id={row_id}) = '{expected_char}`
/// - The page returns data only if the character matches
pub fn blind_injection_payload(
    table: &str,
    column: &str,
    row_id: u32,
    position: usize,
    expected_char: char,
) -> String {
    todo!("Build a blind boolean-based injection payload")
}

/// Simulate a stacked queries injection.
///
/// Some databases (PostgreSQL, MSSQL) allow multiple statements separated by `;`.
/// The attacker appends a destructive statement.
///
/// Exercise: Build a stacked query injection payload.
///
/// Hints:
/// - Append: `; DROP TABLE {table_name}; --`
/// - This terminates the original query and runs the DROP TABLE
pub fn stacked_query_injection(original_input: &str, target_table: &str) -> String {
    todo!("Build a stacked query injection payload with DROP TABLE")
}

/// Validate that a given input does NOT contain SQL injection patterns.
///
/// This is a basic detection function for educational purposes. In production,
/// use parameterized queries instead of input validation.
///
/// Exercise: Check for common injection patterns and return true if the input
/// is suspicious.
///
/// Hints:
/// - Check for: `'`, `--`, `;`, `UNION`, `SELECT`, `DROP`, `OR '1'='1`
/// - Case-insensitive check for keywords
/// - Return `true` if any pattern is found (input is suspicious)
pub fn detect_injection_attempt(input: &str) -> bool {
    todo!("Check input for common SQL injection patterns")
}

/// Demonstrate the full attack chain: detection and exploitation.
///
/// Exercise: Create a function that takes user input, checks if it's an injection
/// attempt, and returns a report.
///
/// Hints:
/// - Check with `detect_injection_attempt`
/// - Build the attack payload if it's an attack
/// - Return (is_attack: bool, payload: String, blocked: bool)
pub fn analyze_input(input: &str) -> (bool, String, bool) {
    todo!("Analyze input for SQL injection: (is_attack, payload, blocked)")
}

/// Demonstrate how parameterized queries neutralize ALL injection patterns.
///
/// Exercise: Show that even the most malicious input becomes harmless when
/// passed as a parameter.
///
/// Hints:
/// - Take a malicious input string
/// - Return (original_query_with_concat, parameterized_query, parameter_value)
/// - The parameterized version should show $1 placeholder
pub fn show_defense_in_depth(malicious_input: &str) -> (String, String, String) {
    todo!("Show vulnerable vs parameterized query for malicious input")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_bypass_is_valid_attack() {
        let payload = auth_bypass_payload();
        assert!(payload.contains("OR"), "Bypass must contain OR");
        assert!(payload.contains("--"), "Bypass must contain comment");
        assert!(detect_injection_attempt(&payload), "Should detect bypass attempt");
    }

    #[test]
    fn test_union_injection_detected() {
        let payload = union_injection_payload(2);
        assert!(detect_injection_attempt(&payload), "Should detect UNION injection");
        assert!(payload.to_uppercase().contains("UNION"));
        assert!(payload.to_uppercase().contains("SELECT"));
    }

    #[test]
    fn test_blind_injection_detected() {
        let payload = blind_injection_payload("users", "password", 1, 1, 'a');
        assert!(detect_injection_attempt(&payload), "Should detect blind injection");
    }

    #[test]
    fn test_stacked_query_detected() {
        let payload = stacked_query_injection("normal_input", "users");
        assert!(payload.contains(';'), "Stacked queries use semicolons");
        assert!(payload.to_uppercase().contains("DROP"), "Should contain DROP TABLE");
        assert!(detect_injection_attempt(&payload));
    }

    #[test]
    fn test_clean_input_not_flagged() {
        assert!(!detect_injection_attempt("alice"));
        assert!(!detect_injection_attempt("john.doe@email.com"));
        assert!(!detect_injection_attempt("Hello World 123"));
    }

    #[test]
    fn test_analyze_clean_input() {
        let (is_attack, _payload, blocked) = analyze_input("normal_user");
        assert!(!is_attack, "Clean input should not be flagged");
    }

    #[test]
    fn test_analyze_attack_input() {
        let attack = "' OR '1'='1' --";
        let (is_attack, _payload, blocked) = analyze_input(attack);
        assert!(is_attack, "Attack input should be flagged");
    }

    #[test]
    fn test_defense_in_depth() {
        let malicious = "'; DROP TABLE users; --";
        let (vulnerable, safe, param) = show_defense_in_depth(malicious);
        assert!(vulnerable.contains("DROP"), "Vulnerable query exposes DROP");
        assert!(safe.contains("$1"), "Safe query uses placeholder");
        assert_eq!(param, malicious, "Parameter holds the raw malicious string");
    }
}
