//! # Lesson 10: Migrating from Weak Hash to Strong Hash (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHasher, SaltString};
use argon2::Argon2;

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

/// Identify the hash algorithm from a hash string.
pub fn identify_hash_version(hash: &str) -> HashVersion {
    if hash.starts_with("$argon2id$") {
        HashVersion::Argon2id
    } else if hash.starts_with("$2a$") || hash.starts_with("$2b$") || hash.starts_with("$2y$") {
        HashVersion::Bcrypt
    } else if hash.len() == 32 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
        HashVersion::Md5
    } else if hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
        HashVersion::Sha256
    } else {
        // Default to Sha256 for unknown formats
        HashVersion::Sha256
    }
}

/// Hash a password with a simulated "MD5" (legacy, for migration demo).
///
/// Since `ring` does not support MD5, we use SHA-256 truncated to 16 bytes
/// (32 hex chars) to simulate an MD5-length hash for educational purposes.
pub fn hash_md5(password: &str) -> String {
    let hash = ring::digest::digest(&ring::digest::SHA256, password.as_bytes());
    // Truncate to 16 bytes to produce 32 hex chars (same length as real MD5)
    hex::encode(&hash.as_ref()[..16])
}

/// Hash a password with SHA-256 (legacy, for migration demo).
pub fn hash_sha256(password: &str) -> String {
    let hash = ring::digest::digest(&ring::digest::SHA256, password.as_bytes());
    hex::encode(hash.as_ref())
}

/// Verify a password against a legacy hash (MD5 or SHA-256).
pub fn verify_legacy_password(password: &str, hash: &str) -> bool {
    let version = identify_hash_version(hash);
    let computed = match version {
        HashVersion::Md5 => hash_md5(password),
        HashVersion::Sha256 => hash_sha256(password),
        _ => return false,
    };
    // Constant-time comparison
    if computed.len() != hash.len() {
        return false;
    }
    let mut result = 0u8;
    for (a, b) in computed.bytes().zip(hash.bytes()) {
        result |= a ^ b;
    }
    result == 0
}

/// Migrate a single user's password to Argon2id.
pub fn migrate_user_password(
    user: &UserRecord,
    password: &str,
) -> Result<UserRecord, String> {
    if !verify_legacy_password(password, &user.password_hash) {
        return Err("Password does not match".to_string());
    }

    // Hash with Argon2id
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let new_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("Argon2id hashing failed: {}", e))?;

    Ok(UserRecord {
        user_id: user.user_id.clone(),
        password_hash: new_hash.to_string(),
        hash_version: HashVersion::Argon2id,
        migrated_at: Some(chrono_now()),
    })
}

/// Run a batch migration on a collection of user records.
pub fn batch_migrate(
    users: &[UserRecord],
    passwords: &HashMap<String, String>,
) -> Vec<(String, bool)> {
    users.iter().map(|user| {
        if let Some(password) = passwords.get(&user.user_id) {
            let success = migrate_user_password(user, password).is_ok();
            (user.user_id.clone(), success)
        } else {
            (user.user_id.clone(), false)
        }
    }).collect()
}

/// Generate a migration status report.
pub fn migration_report(users: &[UserRecord]) -> String {
    let total = users.len();
    let mut md5_count = 0;
    let mut sha256_count = 0;
    let mut bcrypt_count = 0;
    let mut argon2id_count = 0;

    for user in users {
        match user.hash_version {
            HashVersion::Md5 => md5_count += 1,
            HashVersion::Sha256 => sha256_count += 1,
            HashVersion::Bcrypt => bcrypt_count += 1,
            HashVersion::Argon2id => argon2id_count += 1,
        }
    }

    let migrated_pct = if total > 0 {
        (argon2id_count as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    format!(
        "Migration Report\n\
         ================\n\
         Total users: {}\n\
         MD5: {}\n\
         SHA-256: {}\n\
         bcrypt: {}\n\
         argon2id: {}\n\
         Migration progress: {:.1}%",
        total, md5_count, sha256_count, bcrypt_count, argon2id_count, migrated_pct
    )
}

/// Check if a user needs forced password reset.
pub fn needs_forced_reset(
    user: &UserRecord,
    current_time: u64,
    last_login: Option<u64>,
    grace_period_seconds: u64,
) -> bool {
    // Already migrated
    if user.hash_version == HashVersion::Argon2id {
        return false;
    }

    // No login at all
    if last_login.is_none() {
        return true;
    }

    let login_time = last_login.unwrap();
    // Logged in but past grace period
    current_time > login_time + grace_period_seconds
}

/// Simple timestamp helper (avoids pulling in chrono as a dependency)
fn chrono_now() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}s", duration.as_secs())
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
