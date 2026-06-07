//! # Password Hashing
//!
//! Secure password storage requires specialized hashing algorithms that are
//! deliberately slow to resist brute-force attacks. This lesson covers password
//! hashing best practices, salt generation, and work factor tuning.
//!
//! ## Key Concepts
//! - Why fast hashes (SHA-256) are bad for passwords
//! - Password hashing algorithms (bcrypt, argon2, scrypt concepts)
//! - Salt generation and storage
//! - Work factor tuning
//! - Password policy validation
//! - Timing-safe comparison

use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// 1. Password Hasher
// ---------------------------------------------------------------------------

/// A password hasher that simulates bcrypt-like behavior.
/// In production, use the `argon2` or `bcrypt` crate.
#[derive(Debug)]
pub struct PasswordHasher {
    cost: u32,
    salt_length: usize,
}

impl PasswordHasher {
    pub fn new(cost: u32) -> Self {
        Self {
            cost,
            salt_length: 16,
        }
    }

    /// Hash a password with a generated salt.
    pub fn hash(&self, password: &str) -> PasswordHash {
        let salt = generate_salt(self.salt_length);
        let hash = self.derive(password, &salt);
        PasswordHash {
            algorithm: "pbkdf2-sha256".into(),
            cost: self.cost,
            salt: base64_encode(&salt),
            hash: base64_encode(&hash),
        }
    }

    /// Verify a password against a stored hash.
    pub fn verify(&self, password: &str, stored: &PasswordHash) -> bool {
        let salt = base64_decode(&stored.salt);
        let hash = self.derive(password, &salt);
        constant_time_eq(&hash, &base64_decode(&stored.hash))
    }

    /// Derive a key from password and salt using iterative hashing.
    fn derive(&self, password: &str, salt: &[u8]) -> Vec<u8> {
        let mut key = password.as_bytes().to_vec();
        key.extend_from_slice(salt);

        for round in 0..self.cost {
            let mut new_key = Vec::with_capacity(32);
            for (i, &byte) in key.iter().enumerate() {
                let salt_byte = salt[i % salt.len()];
                let round_byte = (round & 0xFF) as u8;
                new_key.push(
                    byte.wrapping_mul(37)
                        .wrapping_add(salt_byte)
                        .wrapping_add(round_byte),
                );
            }
            key = new_key;
        }

        key.truncate(32);
        key
    }
}

/// A stored password hash.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PasswordHash {
    pub algorithm: String,
    pub cost: u32,
    pub salt: String,
    pub hash: String,
}

impl PasswordHash {
    /// Serialize to modular crypt format.
    pub fn to_crypt_format(&self) -> String {
        format!(
            "${}${}${}${}",
            self.algorithm, self.cost, self.salt, self.hash
        )
    }

    /// Parse from modular crypt format.
    pub fn from_crypt_format(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('$').filter(|p| !p.is_empty()).collect();
        if parts.len() != 4 {
            return None;
        }
        Some(Self {
            algorithm: parts[0].into(),
            cost: parts[1].parse().ok()?,
            salt: parts[2].into(),
            hash: parts[3].into(),
        })
    }
}

// ---------------------------------------------------------------------------
// 2. Password Policy
// ---------------------------------------------------------------------------

/// Configurable password policy.
#[derive(Debug, Clone)]
pub struct PasswordPolicy {
    pub min_length: usize,
    pub max_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_digit: bool,
    pub require_special: bool,
    pub min_entropy_bits: f64,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: false,
            min_entropy_bits: 40.0,
        }
    }
}

impl PasswordPolicy {
    /// Validate a password against this policy.
    pub fn validate(&self, password: &str) -> PasswordValidation {
        let mut errors = Vec::new();

        if password.len() < self.min_length {
            errors.push(format!(
                "must be at least {} characters",
                self.min_length
            ));
        }

        if password.len() > self.max_length {
            errors.push(format!(
                "must be at most {} characters",
                self.max_length
            ));
        }

        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            errors.push("must contain at least one uppercase letter".into());
        }

        if self.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            errors.push("must contain at least one lowercase letter".into());
        }

        if self.require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
            errors.push("must contain at least one digit".into());
        }

        if self.require_special && !password.chars().any(|c| !c.is_alphanumeric()) {
            errors.push("must contain at least one special character".into());
        }

        let entropy = calculate_entropy(password);
        if entropy < self.min_entropy_bits {
            errors.push(format!(
                "insufficient entropy: {entropy:.1} bits (minimum: {:.1})",
                self.min_entropy_bits
            ));
        }

        PasswordValidation {
            valid: errors.is_empty(),
            errors,
            entropy_bits: entropy,
            strength: PasswordStrength::from_entropy(entropy),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PasswordValidation {
    pub valid: bool,
    pub errors: Vec<String>,
    pub entropy_bits: f64,
    pub strength: PasswordStrength,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub enum PasswordStrength {
    VeryWeak,
    Weak,
    Fair,
    Strong,
    VeryStrong,
}

impl PasswordStrength {
    pub fn from_entropy(bits: f64) -> Self {
        if bits < 28.0 {
            Self::VeryWeak
        } else if bits < 36.0 {
            Self::Weak
        } else if bits < 60.0 {
            Self::Fair
        } else if bits < 80.0 {
            Self::Strong
        } else {
            Self::VeryStrong
        }
    }
}

/// Calculate the entropy of a password in bits.
pub fn calculate_entropy(password: &str) -> f64 {
    let mut charset_size = 0u32;
    let mut has_lower = false;
    let mut has_upper = false;
    let mut has_digit = false;
    let mut has_special = false;

    for c in password.chars() {
        if c.is_lowercase() && !has_lower {
            charset_size += 26;
            has_lower = true;
        }
        if c.is_uppercase() && !has_upper {
            charset_size += 26;
            has_upper = true;
        }
        if c.is_ascii_digit() && !has_digit {
            charset_size += 10;
            has_digit = true;
        }
        if !c.is_alphanumeric() && !has_special {
            charset_size += 32;
            has_special = true;
        }
    }

    if charset_size == 0 {
        return 0.0;
    }

    (password.len() as f64) * (charset_size as f64).log2()
}

// ---------------------------------------------------------------------------
// 3. Common Password Detection
// ---------------------------------------------------------------------------

/// Check if a password is in a list of common passwords.
pub fn is_common_password(password: &str) -> bool {
    let common = [
        "password", "123456", "12345678", "qwerty", "abc123", "monkey",
        "master", "dragon", "111111", "baseball", "iloveyou", "trustno1",
        "sunshine", "letmein", "welcome", "shadow", "superman",
    ];
    let lower = password.to_lowercase();
    common.iter().any(|c| *c == lower)
}

// ---------------------------------------------------------------------------
// 4. Timing-Safe Comparison
// ---------------------------------------------------------------------------

/// Compare two byte slices in constant time.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

// ---------------------------------------------------------------------------
// 5. Utility Functions
// ---------------------------------------------------------------------------

fn generate_salt(length: usize) -> Vec<u8> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    (0..length)
        .map(|i| ((now >> (i * 7)) & 0xFF) as u8 ^ (i as u8).wrapping_mul(37))
        .collect()
}

fn base64_encode(data: &[u8]) -> String {
    // Simple base64-like encoding for learning
    data.iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join("")
}

fn base64_decode(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let hasher = PasswordHasher::new(100);
        let hash = hasher.hash("my-password");
        assert!(hasher.verify("my-password", &hash));
        assert!(!hasher.verify("wrong-password", &hash));
    }

    #[test]
    fn test_password_hash_different_salts() {
        let hasher = PasswordHasher::new(100);
        let h1 = hasher.hash("same");
        std::thread::sleep(Duration::from_millis(1));
        let h2 = hasher.hash("same");
        assert_ne!(h1.salt, h2.salt);
        assert_ne!(h1.hash, h2.hash);
    }

    #[test]
    fn test_password_hash_crypt_format() {
        let hasher = PasswordHasher::new(100);
        let hash = hasher.hash("test");
        let format = hash.to_crypt_format();
        let parsed = PasswordHash::from_crypt_format(&format).unwrap();
        assert_eq!(parsed.algorithm, hash.algorithm);
        assert_eq!(parsed.cost, hash.cost);
    }

    #[test]
    fn test_password_hash_crypt_format_invalid() {
        assert!(PasswordHash::from_crypt_format("").is_none());
        assert!(PasswordHash::from_crypt_format("a$b").is_none());
    }

    #[test]
    fn test_password_policy_valid() {
        let policy = PasswordPolicy::default();
        let result = policy.validate("SecureP4ss");
        assert!(result.valid, "errors: {:?}", result.errors);
    }

    #[test]
    fn test_password_policy_too_short() {
        let policy = PasswordPolicy::default();
        let result = policy.validate("Ab1");
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("at least")));
    }

    #[test]
    fn test_password_policy_no_uppercase() {
        let policy = PasswordPolicy::default();
        let result = policy.validate("alllowercase1");
        assert!(!result.valid);
    }

    #[test]
    fn test_password_policy_no_digit() {
        let policy = PasswordPolicy::default();
        let result = policy.validate("NoDigitsHere");
        assert!(!result.valid);
    }

    #[test]
    fn test_password_policy_custom() {
        let policy = PasswordPolicy {
            min_length: 4,
            require_uppercase: false,
            require_lowercase: false,
            require_digit: false,
            require_special: true,
            min_entropy_bits: 0.0,
            ..Default::default()
        };
        let result = policy.validate("test!");
        assert!(result.valid);
    }

    #[test]
    fn test_password_strength() {
        assert_eq!(
            PasswordStrength::from_entropy(10.0),
            PasswordStrength::VeryWeak
        );
        assert_eq!(
            PasswordStrength::from_entropy(30.0),
            PasswordStrength::Weak
        );
        assert_eq!(
            PasswordStrength::from_entropy(50.0),
            PasswordStrength::Fair
        );
        assert_eq!(
            PasswordStrength::from_entropy(70.0),
            PasswordStrength::Strong
        );
        assert_eq!(
            PasswordStrength::from_entropy(100.0),
            PasswordStrength::VeryStrong
        );
    }

    #[test]
    fn test_calculate_entropy() {
        // "aaaa" has very low entropy (only 1 char type)
        let e1 = calculate_entropy("aaaa");
        // "Aa1!" uses all char types
        let e2 = calculate_entropy("Aa1!");
        assert!(e2 > e1);

        // Empty string
        assert_eq!(calculate_entropy(""), 0.0);
    }

    #[test]
    fn test_common_password_detection() {
        assert!(is_common_password("password"));
        assert!(is_common_password("PASSWORD"));
        assert!(is_common_password("123456"));
        assert!(!is_common_password("X9#mK2$pL!"));
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"a", b"ab"));
    }

    #[test]
    fn test_password_validation_entropy() {
        let policy = PasswordPolicy {
            min_entropy_bits: 30.0,
            ..Default::default()
        };
        let result = policy.validate("X9#mK2$pL!");
        assert!(result.entropy_bits > 30.0);
    }

    #[test]
    fn test_password_policy_max_length() {
        let policy = PasswordPolicy {
            max_length: 10,
            min_entropy_bits: 0.0,
            ..Default::default()
        };
        let long_pw = "A".repeat(11) + "1";
        let result = policy.validate(&long_pw);
        assert!(!result.valid);
    }
}
