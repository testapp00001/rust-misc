//! # Lesson 02: Secrecy Crate (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use secrecy::{SecretBox, ExposeSecret, SecretString};

/// Constant-time byte slice comparison.
fn ct_bytes_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Wrap a plaintext password in SecretString.
pub fn create_secret_password(plaintext: &str) -> SecretString {
    SecretBox::from(plaintext)
}

/// Compare two secrets using constant-time comparison.
pub fn secrets_equal(a: &SecretString, b: &SecretString) -> bool {
    ct_bytes_eq(
        a.expose_secret().as_bytes(),
        b.expose_secret().as_bytes(),
    )
}

/// A credential with SecretBox-wrapped sensitive fields.
///
/// NOT Debug — intentionally omitted to prevent accidental logging.
pub struct Credential {
    pub username: String,
    pub password: SecretString,
    pub api_token: Option<SecretString>,
}

impl Credential {
    pub fn new(username: &str, password: &str, api_token: Option<&str>) -> Self {
        Self {
            username: username.to_string(),
            password: SecretBox::from(password),
            api_token: api_token.map(|t| SecretBox::from(t)),
        }
    }

    /// Verify the password matches an expected value using constant-time comparison.
    pub fn verify_password(&self, candidate: &str) -> bool {
        ct_bytes_eq(
            self.password.expose_secret().as_bytes(),
            candidate.as_bytes(),
        )
    }
}

/// A secret key wrapped in SecretSlice (SecretBox<[u8]>).
pub struct SecretKey {
    key: secrecy::SecretSlice<u8>,
}

impl SecretKey {
    pub fn new(key_bytes: Vec<u8>) -> Self {
        Self {
            key: key_bytes.into(),
        }
    }

    pub fn from_hex(hex_str: &str) -> Self {
        let bytes = hex::decode(hex_str).expect("Invalid hex string");
        Self::new(bytes)
    }

    pub fn expose_bytes(&self) -> &[u8] {
        self.key.expose_secret()
    }
}

/// Mask a secret for safe display: show first 4 + "***" + last 4 chars.
pub fn mask_secret(secret: &SecretString) -> String {
    let s = secret.expose_secret();
    if s.len() < 8 {
        return "***".to_string();
    }
    format!("{}***{}", &s[..4], &s[s.len() - 4..])
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
