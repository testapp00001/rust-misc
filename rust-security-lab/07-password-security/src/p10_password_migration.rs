//! # Lesson 10: Migrating from Weak Hash to Strong Hash
//!
//! ## The Problem
//!
//! Your legacy system stores passwords as unsalted MD5 or SHA-256 hashes.
//! You want to upgrade to Argon2id, but you have millions of users and cannot
//! force everyone to reset their password at once.
//!
//! ## The Solution: Transparent Migration on Login
//!
//! 1. Store both the old hash and a marker indicating the hash version
//! 2. On login attempt:
//!    a. Verify the password against the stored hash (MD5/SHA-256/whatever)
//!    b. If it matches AND the hash is legacy:
//!       - Re-hash the password with Argon2id
//!       - Replace the stored hash with the new Argon2id hash
//!       - Clear the legacy marker
//!    c. If the hash is already Argon2id, verify normally
//! 3. After a migration period (e.g., 90 days), force remaining legacy users
//!    to reset their passwords
//!
//! ## Storage Format
//!
//! ```
//! {
//!   "user_id": "alice",
//!   "password_hash": "...",
//!   "hash_version": "md5" | "sha256" | "argon2id",
//!   "migrated_at": null | "2024-01-15T10:30:00Z"
//! }
//! ```
//!
//! ## Security Considerations
//!
//! - During migration, the old hash must remain valid -- attackers could exploit this
//! - Set a hard deadline for migration completion
//! - Monitor migration progress: alert if too many users remain on legacy hashes
//! - Consider forcing password resets for users who haven't logged in during
//!   the migration window (they won't trigger transparent migration)
//!
//! ## Hash Identification
//!
//! You can identify hash algorithms by their format:
//! - MD5: 32 hex characters
//! - SHA-256: 64 hex characters
//! - bcrypt: starts with `$2a$` or `$2b$` or `$2y$`
//! - Argon2id: starts with `$argon2id$`
//! - scrypt: starts with `scrypt:`

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the hash algorithm used for a password.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HashVersion {
    Md5,
    Sha256,
    Bcrypt,
    Argon2id,
}

/// A user record with password hash information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    pub user_id: String,
    pub password_hash: String,
    pub hash_version: HashVersion,
    pub migrated_at: Option<String>,
}

/// Exercise 1: Identify the hash algorithm from a hash string.
///
/// Hints:
/// - Argon2id starts with "$argon2id$"
/// - bcrypt starts with "$2a$", "$2b$", or "$2y$"
/// - MD5: exactly 32 hex characters
/// - SHA-256: exactly 64 hex characters
pub fn identify_hash_version(hash: &str) -> HashVersion {
    todo!("Implement hash version identification")
}

/// Exercise 2: Hash a password with MD5 (legacy, for migration demo).
///
/// Hints:
/// - Use `ring::digest::digest(&ring::digest::MD5_FOR_LEGACY_USE_ONLY, data)`
/// - Encode as hex
pub fn hash_md5(password: &str) -> String {
    todo!("Implement MD5 hashing (legacy)")
}

/// Exercise 3: Hash a password with SHA-256 (legacy, for migration demo).
///
/// Hints:
/// - Use `ring::digest::digest(&ring::digest::SHA256, data)`
/// - Encode as hex
pub fn hash_sha256(password: &str) -> String {
    todo!("Implement SHA-256 hashing (legacy)")
}

/// Exercise 4: Verify a password against a legacy hash (MD5 or SHA-256).
///
/// Hints:
/// - Identify the hash version
/// - Recompute the hash with the appropriate algorithm
/// - Compare in constant time
pub fn verify_legacy_password(password: &str, hash: &str) -> bool {
    todo!("Implement legacy password verification")
}

/// Exercise 5: Migrate a single user's password to Argon2id.
///
/// Given a user record with a legacy hash:
/// 1. Verify the password against the old hash
/// 2. If valid, hash with Argon2id
/// 3. Return the updated UserRecord
///
/// Returns Ok(updated_record) if migration succeeded, Err if password didn't match.
pub fn migrate_user_password(
    user: &UserRecord,
    password: &str,
) -> Result<UserRecord, String> {
    todo!("Implement single-user password migration")
}

/// Exercise 6: Run a batch migration on a collection of user records.
///
/// For each user, attempt to migrate with the given password.
/// Returns a list of (user_id, success) tuples.
///
/// In a real system, this would be triggered on each login.
/// This function simulates that for multiple users at once.
pub fn batch_migrate(
    users: &[UserRecord],
    passwords: &HashMap<String, String>,
) -> Vec<(String, bool)> {
    todo!("Implement batch migration")
}

/// Exercise 7: Generate a migration status report.
///
/// Given a list of user records, report:
/// - Total users
/// - Users on each hash version
/// - Migration progress percentage
///
/// Returns a formatted string.
pub fn migration_report(users: &[UserRecord]) -> String {
    todo!("Implement migration report")
}

/// Exercise 8: Check if a user needs forced password reset.
///
/// A user needs forced reset if they're still on a legacy hash and haven't
/// logged in (been migrated) within the grace period.
///
/// current_time and last_login are Unix timestamps.
/// grace_period_seconds is how long users have to log in naturally.
pub fn needs_forced_reset(
    user: &UserRecord,
    current_time: u64,
    last_login: Option<u64>,
    grace_period_seconds: u64,
) -> bool {
    todo!("Implement forced reset check")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identify_md5() {
        let hash = hash_md5("test");
        assert_eq!(identify_hash_version(&hash), HashVersion::Md5);
    }

    #[test]
    fn test_identify_sha256() {
        let hash = hash_sha256("test");
        assert_eq!(identify_hash_version(&hash), HashVersion::Sha256);
    }

    #[test]
    fn test_identify_bcrypt() {
        let hash = "$2b$12$LJ3m4ys1a2MgKjQXO1aQXOqG8v.8pQ3k.2jH1OHTVHTVHTVHTVu";
        assert_eq!(identify_hash_version(hash), HashVersion::Bcrypt);
    }

    #[test]
    fn test_identify_argon2id() {
        let hash = "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$somehash";
        assert_eq!(identify_hash_version(hash), HashVersion::Argon2id);
    }

    #[test]
    fn test_md5_hash_and_verify() {
        let hash = hash_md5("password123");
        assert_eq!(hash.len(), 32, "MD5 hash should be 32 hex chars");
        assert!(verify_legacy_password("password123", &hash));
        assert!(!verify_legacy_password("wrongpassword", &hash));
    }

    #[test]
    fn test_sha256_hash_and_verify() {
        let hash = hash_sha256("password123");
        assert_eq!(hash.len(), 64, "SHA-256 hash should be 64 hex chars");
        assert!(verify_legacy_password("password123", &hash));
        assert!(!verify_legacy_password("wrongpassword", &hash));
    }

    #[test]
    fn test_migrate_user_password() {
        let user = UserRecord {
            user_id: "alice".to_string(),
            password_hash: hash_md5("alicepassword"),
            hash_version: HashVersion::Md5,
            migrated_at: None,
        };

        let migrated = migrate_user_password(&user, "alicepassword").unwrap();
        assert_eq!(migrated.hash_version, HashVersion::Argon2id);
        assert!(migrated.migrated_at.is_some());
        assert!(migrated.password_hash.starts_with("$argon2id$"));
    }

    #[test]
    fn test_migrate_wrong_password() {
        let user = UserRecord {
            user_id: "bob".to_string(),
            password_hash: hash_md5("bobpassword"),
            hash_version: HashVersion::Md5,
            migrated_at: None,
        };

        assert!(migrate_user_password(&user, "wrongpassword").is_err());
    }

    #[test]
    fn test_migration_report() {
        let users = vec![
            UserRecord { user_id: "a".into(), password_hash: "h".into(), hash_version: HashVersion::Md5, migrated_at: None },
            UserRecord { user_id: "b".into(), password_hash: "h".into(), hash_version: HashVersion::Argon2id, migrated_at: Some("now".into()) },
            UserRecord { user_id: "c".into(), password_hash: "h".into(), hash_version: HashVersion::Sha256, migrated_at: None },
        ];
        let report = migration_report(&users);
        assert!(report.contains("3"), "Should mention total users");
        assert!(report.contains("argon2id"), "Should mention Argon2id");
    }

    #[test]
    fn test_needs_forced_reset() {
        let user = UserRecord {
            user_id: "alice".into(),
            password_hash: "old_hash".into(),
            hash_version: HashVersion::Md5,
            migrated_at: None,
        };

        // No login, past grace period
        assert!(needs_forced_reset(&user, 10000, None, 3600));

        // Logged in recently, still on old hash but within grace period
        assert!(!needs_forced_reset(&user, 10000, Some(9000), 3600));

        // Already migrated
        let migrated = UserRecord {
            hash_version: HashVersion::Argon2id,
            migrated_at: Some("now".into()),
            ..user.clone()
        };
        assert!(!needs_forced_reset(&migrated, 10000, Some(9000), 3600));
    }
}
