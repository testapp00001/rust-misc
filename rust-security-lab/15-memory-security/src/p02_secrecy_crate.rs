//! # Lesson 02: Secrecy Crate — Secret<T> Wrapper
//!
//! ## The Problem
//!
//! Secrets leak through `Debug` output, log files, error messages, and serialization.
//! Rust's `println!("{:?}", secret)` or `format!("{}", secret)` can expose keys,
//! passwords, and tokens in logs, error traces, or monitoring systems.
//!
//! ```rust,ignore
//! let api_key = String::from("sk-abc123SECRET");
//! println!("{:?}", api_key);  // Logs: "sk-abc123SECRET"  <-- LEAK!
//! log::info!("key: {}", api_key);  // Also leaks!
//! ```
//!
//! ## The Solution: `secrecy` crate
//!
//! `SecretString` (alias for `SecretBox<str>`) wraps sensitive values and intentionally
//! does NOT implement:
//! - `Debug` (prevents `{:?}` leaks)
//! - `Display` (prevents `{}` leaks)
//!
//! To access the inner value, you must call `.expose_secret()` — this makes
//! the exposure explicit and searchable in code review.
//!
//! ```rust,ignore
//! use secrecy::{SecretString, SecretBox, ExposeSecret};
//!
//! let password: SecretString = SecretBox::from("hunter2");
//! // println!("{}", password);                    // COMPILE ERROR
//! // println!("{:?}", password);                  // COMPILE ERROR
//! println!("{}", password.expose_secret());      // Explicit — easy to audit
//! ```
//!
//! ## Attack: Log Injection
//!
//! 1. Application logs user input + session tokens
//! 2. Attacker triggers error conditions to force logging
//! 3. Logs end up in SIEM, cloud storage, or shared dashboards
//! 4. Session tokens in logs → account takeover
//!
//! Defense: Wrap all secrets in `SecretString`/`SecretBox`, never log the raw value.

use secrecy::{SecretBox, ExposeSecret, SecretString};
use zeroize::Zeroize;

/// Exercise 1: Create a `SecretString` from a plaintext password.
///
/// Hints:
/// - Use `SecretBox::from(value)` or the `Into` trait
/// - `SecretString` is a type alias for `SecretBox<str>`
pub fn create_secret_password(plaintext: &str) -> SecretString {
    todo!("Wrap the plaintext in SecretBox::from")
}

/// Exercise 2: Safely compare two secrets for equality.
///
/// Requirements:
/// - Take two `SecretString` references
/// - Compare their inner values for equality
/// - Use `.expose_secret()` to access the inner value
/// - Use constant-time comparison (not `==`)
/// - Return `bool`
///
/// Hints:
/// - `.expose_secret()` returns `&str`
/// - Use your own constant-time comparison (accumulate XOR + OR)
/// - Convert strings to bytes with `.as_bytes()`
pub fn secrets_equal(a: &SecretString, b: &SecretString) -> bool {
    todo!("Compare secrets using constant-time comparison")
}

/// Exercise 3: Implement a `Credential` struct that uses `SecretString` for sensitive fields.
///
/// Requirements:
/// - `username: String` (not secret — usernames are not sensitive)
/// - `password: SecretString` (secret — wrapped in SecretBox)
/// - `api_token: Option<SecretString>` (optional secret)
/// - The struct must NOT implement `Debug` (no `#[derive(Debug)]`)
/// - Implement `new(username, password, api_token)` constructor
pub struct Credential {
    pub username: String,
    pub password: SecretString,
    pub api_token: Option<SecretString>,
}

impl Credential {
    pub fn new(username: &str, password: &str, api_token: Option<&str>) -> Self {
        todo!("Create Credential with SecretBox-wrapped sensitive fields")
    }

    /// Verify the password matches an expected value.
    /// Use constant-time comparison.
    pub fn verify_password(&self, candidate: &str) -> bool {
        todo!("Compare candidate password against stored secret")
    }
}

/// Exercise 4: Implement `SecretKey` using `secrecy::SecretSlice<u8>` for raw key bytes.
///
/// Hints:
/// - `SecretSlice<u8>` is `SecretBox<[u8]>`, created from `Vec<u8>` via `.into()`
/// - Provide `from_hex(hex_str)` that parses a hex string into bytes
/// - Provide `expose_bytes(&self) -> &[u8]` that exposes the raw bytes
/// - Use `ExposeSecret` trait to access the inner slice
pub struct SecretKey {
    key: secrecy::SecretSlice<u8>,
}

impl SecretKey {
    pub fn new(key_bytes: Vec<u8>) -> Self {
        todo!("Wrap key bytes in SecretSlice")
    }

    pub fn from_hex(hex_str: &str) -> Self {
        todo!("Parse hex string into SecretSlice")
    }

    pub fn expose_bytes(&self) -> &[u8] {
        todo!("Expose the raw key bytes")
    }
}

/// Exercise 5: Implement a function that processes a secret but never leaks it.
///
/// Requirements:
/// - Take a `SecretString` containing an API key
/// - Compute a "masked" version: show first 4 chars + "***" + last 4 chars
/// - Return the masked string (NOT a Secret — masking is safe to display)
/// - Example: "sk-abc12345678" → "sk-a***5678"
///
/// Hints:
/// - Use `.expose_secret()` to get the inner string
/// - Check length — if < 8 chars, return "***"
pub fn mask_secret(secret: &SecretString) -> String {
    todo!("Create a masked version of the secret for safe display")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_secret_password() {
        let secret = create_secret_password("hunter2");
        assert_eq!(secret.expose_secret(), "hunter2");
    }

    #[test]
    fn test_secrets_equal_same() {
        let a: SecretString = SecretBox::from("same_password");
        let b: SecretString = SecretBox::from("same_password");
        assert!(secrets_equal(&a, &b));
    }

    #[test]
    fn test_secrets_equal_different() {
        let a: SecretString = SecretBox::from("password1");
        let b: SecretString = SecretBox::from("password2");
        assert!(!secrets_equal(&a, &b));
    }

    #[test]
    fn test_credential_new() {
        let cred = Credential::new("alice", "s3cret", Some("token123"));
        assert_eq!(cred.username, "alice");
        assert_eq!(cred.password.expose_secret(), "s3cret");
        assert_eq!(
            cred.api_token.as_ref().unwrap().expose_secret(),
            "token123"
        );
    }

    #[test]
    fn test_credential_verify_password_correct() {
        let cred = Credential::new("alice", "s3cret", None);
        assert!(cred.verify_password("s3cret"));
    }

    #[test]
    fn test_credential_verify_password_wrong() {
        let cred = Credential::new("alice", "s3cret", None);
        assert!(!cred.verify_password("wrong"));
    }

    #[test]
    fn test_secret_key_from_hex() {
        let key = SecretKey::from_hex("deadbeef");
        assert_eq!(key.expose_bytes(), &[0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_mask_secret_long() {
        let secret: SecretString = SecretBox::from("sk-abc12345678");
        assert_eq!(mask_secret(&secret), "sk-a***5678");
    }

    #[test]
    fn test_mask_secret_short() {
        let secret: SecretString = SecretBox::from("abc");
        assert_eq!(mask_secret(&secret), "***");
    }
}
