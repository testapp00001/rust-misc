//! # Lesson 04: Error-Based Oracle Attacks
//!
//! ## The Problem
//!
//! An oracle attack occurs when different error responses reveal information about
//! internal state. The classic example: authentication systems that return "user not
//! found" for invalid usernames but "wrong password" for valid usernames with bad
//! passwords. This lets attackers enumerate every valid username in the system.
//!
//! ## Attack Walkthrough
//!
//! ```text
//! Attacker tries: admin / anything
//! System responds: "User not found"         → admin is not a valid username
//!
//! Attacker tries: alice / anything
//! System responds: "Wrong password"          → alice IS a valid username!
//!
//! Attacker tries: bob / anything
//! System responds: "User not found"          → bob is not valid
//!
//! Attacker tries: charlie / anything
//! System responds: "Wrong password"          → charlie IS valid!
//!
//! Result: attacker now has a list of valid usernames to target with password attacks.
//! ```
//!
//! ## Variations
//!
//! | Oracle Type | What Attacker Learns |
//! |-------------|---------------------|
//! | Different auth error messages | Valid usernames |
//! | Different registration errors | Existing emails |
//! | Different reset password errors | Valid email addresses |
//! | Different error codes in response | Internal logic flow |
//! | Different HTTP status codes | Account state |
//!
//! ## Defense
//!
//! Return IDENTICAL responses for all failure modes in security-critical paths.
//! The response, status code, headers, and timing must be indistinguishable.

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
///
/// This is intentionally insecure. Study it to understand the attack, then implement
/// the secure version below.
///
/// DO NOT FIX THIS FUNCTION -- it's meant to demonstrate the vulnerability.
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

/// Exercise 1: Implement secure authentication that returns identical errors.
///
/// The function must:
/// - Return Ok(email) ONLY when authentication fully succeeds
/// - Return Err("Authentication failed") for ALL failure modes:
///   - User not found
///   - Wrong password
///   - Account disabled
/// - Never reveal which step failed
///
/// Hints:
/// - Use a single error message for all failures
/// - Don't match on specific failure conditions with different messages
/// - Only distinguish success from failure
pub fn authenticate_secure(
    db: &UserDb,
    username: &str,
    password: &str,
) -> Result<String, String> {
    todo!("Implement secure authentication with uniform error messages")
}

/// Exercise 2: Check if two authentication responses are distinguishable.
///
/// Given two error messages from authentication, return true if they are
/// distinguishable (different text), false if they are identical.
///
/// An attacker uses this to determine if two usernames have different states.
pub fn are_responses_distinguishable(response1: &str, response2: &str) -> bool {
    todo!("Check if two responses reveal different information")
}

/// Exercise 3: Audit an authentication function for oracle vulnerabilities.
///
/// Given a list of (username, response) pairs from authentication attempts,
/// check if all error responses are identical. Return:
/// - Ok(()) if all errors are identical (secure)
/// - Err(description) if errors differ (vulnerable)
///
/// Skip entries where the response starts with "Welcome" (success cases).
pub fn audit_auth_responses(attempts: &[(&str, String)]) -> Result<(), String> {
    todo!("Audit authentication responses for oracle vulnerabilities")
}

/// Exercise 4: Implement secure registration that leaks no information.
///
/// Registration must return the same success message whether the email is
/// already registered or not. This prevents attackers from enumerating
/// registered emails.
///
/// Always return Ok("If this email is available, a confirmation will be sent")
/// regardless of whether the email exists.
///
/// The function should only actually insert the user if the email is NOT already taken.
pub fn register_secure(
    db: &mut UserDb,
    username: &str,
    password: &str,
    email: &str,
) -> Result<String, String> {
    todo!("Implement secure registration with uniform response")
}

/// Exercise 5: Implement secure password reset that leaks no information.
///
/// Password reset must return the same message whether the email exists or not.
/// This prevents attackers from discovering which emails have accounts.
///
/// Always return Ok("If an account exists with this email, a reset link will be sent")
pub fn password_reset_secure(
    db: &UserDb,
    email: &str,
) -> Result<String, String> {
    todo!("Implement secure password reset with uniform response")
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
