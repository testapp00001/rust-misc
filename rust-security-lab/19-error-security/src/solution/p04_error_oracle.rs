//! # Lesson 04: Error-Based Oracle Attacks (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::HashMap;

/// Simulated user database: username -> (password_hash, email, is_active)
pub type UserDb = HashMap<String, (String, String, bool)>;

/// Build a sample user database for testing.
pub fn build_user_db() -> UserDb {
    let mut db = HashMap::new();
    db.insert("alice".to_string(), ("hash_alice_pw".to_string(), "alice@example.com".to_string(), true));
    db.insert("bob".to_string(), ("hash_bob_pw".to_string(), "bob@example.com".to_string(), true));
    db.insert("charlie".to_string(), ("hash_charlie_pw".to_string(), "charlie@example.com".to_string(), false));
    db
}

/// VULNERABLE authentication -- leaks information through different error messages.
pub fn authenticate_vulnerable(
    db: &UserDb,
    username: &str,
    password: &str,
) -> Result<String, String> {
    match db.get(username) {
        None => Err("User not found".to_string()),
        Some((pw_hash, email, is_active)) => {
            if pw_hash != password {
                Err("Wrong password".to_string())
            } else if !is_active {
                Err("Account is disabled".to_string())
            } else {
                Ok(format!("Welcome, {}!", email))
            }
        }
    }
}

/// Secure authentication -- identical error for all failure modes.
pub fn authenticate_secure(
    db: &UserDb,
    username: &str,
    password: &str,
) -> Result<String, String> {
    match db.get(username) {
        Some((pw_hash, email, is_active)) => {
            if pw_hash == password && *is_active {
                Ok(format!("Welcome, {}!", email))
            } else {
                Err("Authentication failed".to_string())
            }
        }
        None => Err("Authentication failed".to_string()),
    }
}

/// Check if two authentication responses are distinguishable.
pub fn are_responses_distinguishable(response1: &str, response2: &str) -> bool {
    response1 != response2
}

/// Audit authentication responses for oracle vulnerabilities.
pub fn audit_auth_responses(attempts: &[(&str, String)]) -> Result<(), String> {
    let errors: Vec<&str> = attempts
        .iter()
        .filter(|(_, resp)| !resp.starts_with("Welcome"))
        .map(|(_, resp)| resp.as_str())
        .collect();

    if errors.is_empty() {
        return Ok(());
    }

    let first = errors[0];
    for error in &errors[1..] {
        if *error != first {
            return Err(format!(
                "Oracle vulnerability detected: responses differ ('{}' vs '{}')",
                first, error
            ));
        }
    }
    Ok(())
}

/// Secure registration -- uniform response regardless of email existence.
pub fn register_secure(
    db: &mut UserDb,
    username: &str,
    password: &str,
    email: &str,
) -> Result<String, String> {
    // Only insert if not already taken
    if !db.contains_key(username) {
        db.insert(username.to_string(), (password.to_string(), email.to_string(), true));
    }
    // Always return the same message
    Ok("If this email is available, a confirmation will be sent".to_string())
}

/// Secure password reset -- uniform response regardless of email existence.
pub fn password_reset_secure(
    _db: &UserDb,
    _email: &str,
) -> Result<String, String> {
    // Always return the same message -- never reveal if email exists
    Ok("If an account exists with this email, a reset link will be sent".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vulnerable_leaks_user_existence() {
        let db = build_user_db();
        let r1 = authenticate_vulnerable(&db, "alice", "wrong").unwrap_err();
        let r2 = authenticate_vulnerable(&db, "nonexistent", "wrong").unwrap_err();
        assert_ne!(r1, r2, "Vulnerable version leaks different errors (this is the attack!)");
    }

    #[test]
    fn test_secure_same_error_user_exists() {
        let db = build_user_db();
        let r1 = authenticate_secure(&db, "alice", "wrong").unwrap_err();
        let r2 = authenticate_secure(&db, "nonexistent", "wrong").unwrap_err();
        assert_eq!(r1, r2, "Secure version must return identical errors");
    }

    #[test]
    fn test_secure_same_error_disabled_account() {
        let db = build_user_db();
        let r1 = authenticate_secure(&db, "charlie", "wrong").unwrap_err();
        let r2 = authenticate_secure(&db, "nonexistent", "wrong").unwrap_err();
        assert_eq!(r1, r2, "Disabled account must not produce different error");
    }

    #[test]
    fn test_secure_success_still_works() {
        let db = build_user_db();
        let result = authenticate_secure(&db, "alice", "hash_alice_pw");
        assert!(result.is_ok(), "Valid credentials must succeed");
        assert!(result.unwrap().contains("alice@example.com"));
    }

    #[test]
    fn test_not_distinguishable_same() {
        assert!(!are_responses_distinguishable("Auth failed", "Auth failed"));
    }

    #[test]
    fn test_distinguishable_different() {
        assert!(are_responses_distinguishable("User not found", "Wrong password"));
    }

    #[test]
    fn test_audit_secure() {
        let attempts = vec![
            ("alice", "Authentication failed".to_string()),
            ("bob", "Authentication failed".to_string()),
            ("charlie", "Authentication failed".to_string()),
        ];
        assert!(audit_auth_responses(&attempts).is_ok());
    }

    #[test]
    fn test_audit_detects_vulnerability() {
        let attempts = vec![
            ("alice", "Wrong password".to_string()),
            ("nonexistent", "User not found".to_string()),
        ];
        assert!(audit_auth_responses(&attempts).is_err());
    }

    #[test]
    fn test_register_same_response_new_and_existing() {
        let mut db = build_user_db();
        let r1 = register_secure(&mut db, "newuser", "pw123", "new@example.com").unwrap();
        let r2 = register_secure(&mut db, "newuser", "pw123", "new@example.com").unwrap();
        assert_eq!(r1, r2, "Registration must return same message for new and existing emails");
    }

    #[test]
    fn test_password_reset_same_response() {
        let db = build_user_db();
        let r1 = password_reset_secure(&db, "alice@example.com").unwrap();
        let r2 = password_reset_secure(&db, "nonexistent@example.com").unwrap();
        assert_eq!(r1, r2, "Password reset must return same message for all emails");
    }
}
