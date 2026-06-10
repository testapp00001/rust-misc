//! # Lesson 02: bcrypt Password Hashing
//!
//! ## Widely Used, Tunable Cost
//!
//! bcrypt was designed by Niels Provos and David Mazieres in 1999, based on the
//! Blowfish cipher. It has been the de facto standard for password hashing for
//! over two decades and is still found in most web frameworks.
//!
//! ## How bcrypt Works
//!
//! 1. Takes a password (up to 72 bytes) and a 128-bit salt
//! 2. Runs the Blowfish key schedule `2^cost` times (expensive by design)
//! 3. Produces a 60-character encoded string:
//!    `$2b$<cost>$<22-char-salt><31-char-hash>`
//!
//! The cost factor doubles the work with each increment:
//! | Cost | Iterations | Approximate Time |
//! |------|------------|------------------|
//! | 10   | 1,024      | ~25 ms           |
//! | 11   | 2,048      | ~50 ms           |
//! | 12   | 4,096      | ~100 ms          |
//! | 13   | 8,192      | ~200 ms          |
//! | 14   | 16,384     | ~400 ms          |
//!
//! ## Strengths and Weaknesses
//!
//! **Strengths:**
//! - Battle-tested -- 25+ years of real-world use
//! - Built-in salt generation
//! - Tunable cost factor
//! - Wide language/framework support
//!
//! **Weaknesses:**
//! - NOT memory-hard (GPU attacks are still feasible, just slower)
//! - 72-byte password limit (truncates silently!)
//! - Cache-timing side channels in some implementations
//! - Output format is not the PHC standard
//!
//! ## Attack Scenario
//!
//! With bcrypt at cost 10, an attacker with a modern GPU cluster can try
//! ~100,000 hashes per second. For a 6-character alphanumeric password
//! (62^6 = ~56 billion combinations), this takes about 6 days. With Argon2id,
//! the same attack would take months or years due to memory constraints.
//!
//! bcrypt is better than SHA-256 but weaker than Argon2id against GPU attacks.
//! Use Argon2id for new systems; bcrypt is acceptable for legacy compatibility.

/// Exercise 1: Hash a password with bcrypt using the default cost.
///
/// Hints:
/// - Use `bcrypt::hash(password, cost)?`
/// - Default cost is typically 10 or 12
/// - The result is a 60-char string like `$2b$12$...`
pub fn hash_password_bcrypt(password: &str, cost: u32) -> Result<String, String> {
    todo!("Implement bcrypt password hashing")
}

/// Exercise 2: Verify a password against a bcrypt hash.
///
/// Hints:
/// - Use `bcrypt::verify(password, &hash)?`
/// - Returns true/false directly
pub fn verify_password_bcrypt(password: &str, hash: &str) -> Result<bool, String> {
    todo!("Implement bcrypt password verification")
}

/// Exercise 3: Find the highest cost factor that stays under a time budget.
///
/// Start at cost 4 and double until hashing takes longer than `max_ms` milliseconds.
/// Return the highest cost that completes within the budget.
///
/// Hints:
/// - Use `std::time::Instant` to measure
/// - Start at cost 4, try 5, 6, ... until one exceeds the budget
/// - Return the last cost that fit
pub fn find_cost_for_time_budget(max_ms: u128) -> u32 {
    todo!("Implement cost factor discovery")
}

/// Exercise 4: Hash a password, ignoring bytes beyond the 72-byte limit.
///
/// bcrypt silently truncates passwords at 72 bytes. This function should
/// explicitly truncate the input and warn the caller.
///
/// Returns (truncated_password_used, hash).
///
/// Hints:
/// - Take only the first 72 bytes: `&password.as_bytes()[..72.min(password.len())]`
/// - Convert back to &str with `std::str::from_utf8`
/// - Hash with bcrypt
pub fn hash_with_truncation_awareness(password: &str, cost: u32) -> Result<(String, String), String> {
    todo!("Implement truncation-aware bcrypt hashing")
}

/// Exercise 5: Detect if a hash string is a valid bcrypt hash.
///
/// Valid bcrypt hashes start with `$2a$`, `$2b$`, or `$2y$` and are exactly 60 characters.
///
/// Hints:
/// - Check length == 60
/// - Check prefix is one of "$2a$", "$2b$", "$2y$"
/// - Check that char at index 4 is a digit (cost starts there)
pub fn is_valid_bcrypt_hash(hash: &str) -> bool {
    todo!("Implement bcrypt hash validation")
}

/// Exercise 6: Extract the cost factor from a bcrypt hash string.
///
/// The cost is encoded at positions 4-5 (or 4-6 for cost >= 10).
///
/// Hints:
/// - The hash format is `$2b$<cost>$...`
/// - Split by '$' and parse the third element
pub fn extract_bcrypt_cost(hash: &str) -> Result<u32, String> {
    todo!("Implement bcrypt cost extraction")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let hash = hash_password_bcrypt("mypassword", 4).unwrap();
        assert!(verify_password_bcrypt("mypassword", &hash).unwrap());
    }

    #[test]
    fn test_verify_wrong_password() {
        let hash = hash_password_bcrypt("mypassword", 4).unwrap();
        assert!(!verify_password_bcrypt("wrongpassword", &hash).unwrap());
    }

    #[test]
    fn test_hash_format() {
        let hash = hash_password_bcrypt("test", 4).unwrap();
        assert_eq!(hash.len(), 60, "bcrypt hash should be 60 characters");
        assert!(hash.starts_with("$2b$"), "Should use $2b$ prefix");
    }

    #[test]
    fn test_cost_affects_output() {
        let h1 = hash_password_bcrypt("test", 4).unwrap();
        let h2 = hash_password_bcrypt("test", 5).unwrap();
        // Different costs produce different hashes (for the same password)
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_find_cost_for_budget() {
        // With a generous budget, should find at least cost 4
        let cost = find_cost_for_time_budget(5000); // 5 seconds
        assert!(cost >= 4, "Cost should be at least 4, got {}", cost);
    }

    #[test]
    fn test_truncation_awareness() {
        let long_password = "a".repeat(100);
        let (used, hash) = hash_with_truncation_awareness(&long_password, 4).unwrap();
        assert_eq!(used.len(), 72, "Should truncate to 72 bytes");
        assert!(verify_password_bcrypt(&used, &hash).unwrap());
    }

    #[test]
    fn test_valid_bcrypt_hash() {
        let hash = hash_password_bcrypt("test", 4).unwrap();
        assert!(is_valid_bcrypt_hash(&hash));
        assert!(!is_valid_bcrypt_hash("not-a-hash"));
        assert!(!is_valid_bcrypt_hash(""));
    }

    #[test]
    fn test_extract_cost() {
        let hash = hash_password_bcrypt("test", 4).unwrap();
        assert_eq!(extract_bcrypt_cost(&hash).unwrap(), 4);

        let hash10 = hash_password_bcrypt("test", 10).unwrap();
        assert_eq!(extract_bcrypt_cost(&hash10).unwrap(), 10);
    }
}
