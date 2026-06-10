//! # Lesson 02: bcrypt Password Hashing (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::time::Instant;

/// Hash a password with bcrypt using the given cost.
pub fn hash_password_bcrypt(password: &str, cost: u32) -> Result<String, String> {
    bcrypt::hash(password, cost).map_err(|e| format!("bcrypt hash failed: {}", e))
}

/// Verify a password against a bcrypt hash.
pub fn verify_password_bcrypt(password: &str, hash: &str) -> Result<bool, String> {
    bcrypt::verify(password, hash).map_err(|e| format!("bcrypt verify failed: {}", e))
}

/// Find the highest cost factor that stays under a time budget.
pub fn find_cost_for_time_budget(max_ms: u128) -> u32 {
    let mut cost = 4u32;
    loop {
        let start = Instant::now();
        let _ = bcrypt::hash("benchmark_password", cost);
        let elapsed = start.elapsed().as_millis();

        if elapsed > max_ms {
            return cost - 1;
        }
        cost += 1;
        if cost > 31 {
            return 31; // bcrypt max cost
        }
    }
}

/// Hash a password, explicitly truncating to 72 bytes.
pub fn hash_with_truncation_awareness(password: &str, cost: u32) -> Result<(String, String), String> {
    let truncated = if password.len() > 72 {
        let bytes = &password.as_bytes()[..72];
        std::str::from_utf8(bytes)
            .map_err(|_| "Invalid UTF-8 after truncation".to_string())?
            .to_string()
    } else {
        password.to_string()
    };
    let hash = hash_password_bcrypt(&truncated, cost)?;
    Ok((truncated, hash))
}

/// Detect if a hash string is a valid bcrypt hash.
pub fn is_valid_bcrypt_hash(hash: &str) -> bool {
    if hash.len() != 60 {
        return false;
    }
    if !hash.starts_with("$2a$") && !hash.starts_with("$2b$") && !hash.starts_with("$2y$") {
        return false;
    }
    // Cost starts at position 4, should be a digit
    hash.as_bytes().get(4).map_or(false, |b| b.is_ascii_digit())
}

/// Extract the cost factor from a bcrypt hash string.
pub fn extract_bcrypt_cost(hash: &str) -> Result<u32, String> {
    if !is_valid_bcrypt_hash(hash) {
        return Err("Invalid bcrypt hash format".to_string());
    }
    // Hash format: $2b$<cost>$<salt><hash>
    // Cost is between the 2nd and 3rd '$'
    let parts: Vec<&str> = hash.split('$').collect();
    if parts.len() < 4 {
        return Err("Invalid bcrypt hash structure".to_string());
    }
    parts[2].parse::<u32>().map_err(|e| format!("Failed to parse cost: {}", e))
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
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_find_cost_for_budget() {
        let cost = find_cost_for_time_budget(5000);
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
