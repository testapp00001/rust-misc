//! # Lesson 09: Data at Rest (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};

/// Encryption approach for data at rest
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EncryptionApproach {
    FullDiskEncryption,
    FileLevelEncryption,
    TransparentDataEncryption,
    None,
}

/// A security policy for data at rest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAtRestPolicy {
    pub name: String,
    pub approach: EncryptionApproach,
    pub algorithm: String,
    pub key_size: u32,
    pub encrypt_metadata: bool,
}

/// Evaluate whether a data-at-rest policy is secure.
///
/// A policy is "secure" if:
/// - It uses encryption (not `None`)
/// - Key size is at least 256 bits
/// - Algorithm is a recognized AEAD cipher
pub fn evaluate_policy(policy: &DataAtRestPolicy) -> bool {
    // Must use encryption
    if policy.approach == EncryptionApproach::None {
        return false;
    }

    // Must use strong key size
    if policy.key_size < 256 {
        return false;
    }

    // Must use a recognized secure algorithm
    let secure_algorithms = [
        "AES-256-XTS",
        "AES-256-GCM",
        "ChaCha20-Poly1305",
        "XChaCha20-Poly1305",
    ];
    secure_algorithms.contains(&policy.algorithm.as_str())
}

/// Recommend an encryption approach based on the scenario.
///
/// Uses a priority system: PHI always requires encryption, then
/// the most specific recommendation for the device type.
pub fn recommend_approach(
    is_portable_device: bool,
    is_shared_server: bool,
    is_database: bool,
    contains_phi: bool,
) -> EncryptionApproach {
    // PHI always requires encryption — choose the strongest applicable approach
    if contains_phi {
        if is_database {
            return EncryptionApproach::TransparentDataEncryption;
        }
        if is_portable_device {
            return EncryptionApproach::FullDiskEncryption;
        }
        return EncryptionApproach::FileLevelEncryption;
    }

    // Portable devices need FDE (stolen laptop risk)
    if is_portable_device {
        return EncryptionApproach::FullDiskEncryption;
    }

    // Databases use TDE
    if is_database {
        return EncryptionApproach::TransparentDataEncryption;
    }

    // Shared servers use file-level encryption
    if is_shared_server {
        return EncryptionApproach::FileLevelEncryption;
    }

    // Default: file-level encryption
    EncryptionApproach::FileLevelEncryption
}

/// Create a DataAtRestPolicy with appropriate settings.
///
/// Maps each encryption approach to its recommended algorithm and settings.
pub fn create_policy(approach: EncryptionApproach) -> DataAtRestPolicy {
    match approach {
        EncryptionApproach::FullDiskEncryption => DataAtRestPolicy {
            name: "Full Disk Encryption".to_string(),
            approach,
            algorithm: "AES-256-XTS".to_string(),
            key_size: 256,
            encrypt_metadata: false, // FDE doesn't encrypt filenames
        },
        EncryptionApproach::FileLevelEncryption => DataAtRestPolicy {
            name: "File Level Encryption".to_string(),
            approach,
            algorithm: "AES-256-GCM".to_string(),
            key_size: 256,
            encrypt_metadata: true, // FLE can encrypt filenames
        },
        EncryptionApproach::TransparentDataEncryption => DataAtRestPolicy {
            name: "Transparent Data Encryption".to_string(),
            approach,
            algorithm: "AES-256-GCM".to_string(),
            key_size: 256,
            encrypt_metadata: false,
        },
        EncryptionApproach::None => DataAtRestPolicy {
            name: "No Encryption".to_string(),
            approach,
            algorithm: "none".to_string(),
            key_size: 0,
            encrypt_metadata: false,
        },
    }
}

/// Check if a file extension indicates sensitive data.
///
/// Sensitive file types include cryptographic keys, configuration files,
/// databases, and backups.
pub fn is_sensitive_extension(filename: &str) -> bool {
    let sensitive_extensions = [
        ".key", ".pem", ".p12", ".pfx", ".jks", // Crypto keys
        ".env", ".cfg", ".ini", ".conf", ".yaml", ".yml", // Config
        ".db", ".sqlite", ".sqlite3", // Databases
        ".bak", ".dump", ".sql", ".tar", ".gz", // Backups
        ".secret", ".token", ".credentials", // Secrets
    ];

    let lower = filename.to_lowercase();
    sensitive_extensions.iter().any(|ext| lower.ends_with(ext))
}

/// Calculate the encrypted size including overhead.
///
/// AEAD ciphers add:
/// - A nonce (to ensure uniqueness)
/// - An authentication tag (to detect tampering)
pub fn calculate_encrypted_size(plaintext_size: u64, algorithm: &str) -> u64 {
    let overhead = match algorithm {
        "AES-256-GCM" | "ChaCha20-Poly1305" => 28, // 12 nonce + 16 tag
        "XChaCha20-Poly1305" => 40,                 // 24 nonce + 16 tag
        "AES-256-XTS" => 0,                          // Block cipher, no overhead
        _ => 28,                                      // Default assumption
    };
    plaintext_size + overhead
}

/// Assess the security level of a storage configuration.
///
/// Returns a score from 0 (insecure) to 10 (maximum security).
/// Each security feature adds points to the base score.
pub fn assess_security_level(
    has_fde: bool,
    has_file_encryption: bool,
    encrypts_metadata: bool,
) -> u32 {
    let mut score = 0;

    if has_fde {
        score += 7; // FDE is the foundation
    }

    if has_file_encryption {
        score += if has_fde { 2 } else { 5 }; // More valuable without FDE
    }

    if encrypts_metadata && has_file_encryption {
        score += 1; // Metadata encryption is the cherry on top
    }

    score.min(10)
}

/// Generate a security report for a storage configuration.
///
/// Provides a human-readable assessment of the security posture
/// with specific recommendations for improvement.
pub fn generate_security_report(
    has_fde: bool,
    has_file_encryption: bool,
    encrypts_metadata: bool,
) -> String {
    let mut report = String::new();
    let score = assess_security_level(has_fde, has_file_encryption, encrypts_metadata);

    report.push_str(&format!("Security Level: {}/10\n", score));
    report.push_str("─".repeat(40).as_str());
    report.push('\n');

    // Current state
    report.push_str("Current Configuration:\n");
    report.push_str(&format!("  Full-disk encryption: {}\n", if has_fde { "ENABLED" } else { "DISABLED" }));
    report.push_str(&format!("  File-level encryption: {}\n", if has_file_encryption { "ENABLED" } else { "DISABLED" }));
    report.push_str(&format!("  Metadata encryption: {}\n", if encrypts_metadata { "ENABLED" } else { "DISABLED" }));
    report.push('\n');

    // Recommendations
    report.push_str("Recommendations:\n");
    if !has_fde {
        report.push_str("  [CRITICAL] Enable full-disk encryption (LUKS/FileVault/BitLocker)\n");
    }
    if !has_file_encryption {
        report.push_str("  [HIGH] Enable file-level encryption for sensitive data\n");
    }
    if has_file_encryption && !encrypts_metadata {
        report.push_str("  [MEDIUM] Enable metadata encryption to protect filenames\n");
    }
    if has_fde && has_file_encryption && encrypts_metadata {
        report.push_str("  [OK] Storage security posture is strong\n");
    }

    report
}

/// Check if a key rotation is needed.
///
/// Keys should be rotated periodically to limit the impact of a
/// potential key compromise. The maximum age depends on the sensitivity
/// of the data and regulatory requirements.
pub fn needs_key_rotation(key_created_days_ago: u32, max_age_days: u32) -> bool {
    key_created_days_ago >= max_age_days
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_policy_secure() {
        let policy = DataAtRestPolicy {
            name: "Standard".to_string(),
            approach: EncryptionApproach::FileLevelEncryption,
            algorithm: "AES-256-GCM".to_string(),
            key_size: 256,
            encrypt_metadata: true,
        };
        assert!(evaluate_policy(&policy));
    }

    #[test]
    fn test_evaluate_policy_insecure_no_encryption() {
        let policy = DataAtRestPolicy {
            name: "None".to_string(),
            approach: EncryptionApproach::None,
            algorithm: "none".to_string(),
            key_size: 0,
            encrypt_metadata: false,
        };
        assert!(!evaluate_policy(&policy));
    }

    #[test]
    fn test_evaluate_policy_insecure_weak_key() {
        let policy = DataAtRestPolicy {
            name: "Weak".to_string(),
            approach: EncryptionApproach::FileLevelEncryption,
            algorithm: "AES-256-GCM".to_string(),
            key_size: 128, // Too weak
            encrypt_metadata: false,
        };
        assert!(!evaluate_policy(&policy));
    }

    #[test]
    fn test_recommend_portable() {
        let approach = recommend_approach(true, false, false, false);
        assert_eq!(approach, EncryptionApproach::FullDiskEncryption);
    }

    #[test]
    fn test_recommend_shared_server() {
        let approach = recommend_approach(false, true, false, false);
        assert_eq!(approach, EncryptionApproach::FileLevelEncryption);
    }

    #[test]
    fn test_recommend_database() {
        let approach = recommend_approach(false, false, true, false);
        assert_eq!(approach, EncryptionApproach::TransparentDataEncryption);
    }

    #[test]
    fn test_is_sensitive_extension() {
        assert!(is_sensitive_extension("private.key"));
        assert!(is_sensitive_extension("cert.pem"));
        assert!(is_sensitive_extension(".env"));
        assert!(is_sensitive_extension("data.db"));
        assert!(!is_sensitive_extension("readme.txt"));
        assert!(!is_sensitive_extension("photo.jpg"));
    }

    #[test]
    fn test_calculate_encrypted_size_gcm() {
        let size = calculate_encrypted_size(1000, "AES-256-GCM");
        assert_eq!(size, 1028, "GCM: +12 nonce + 16 tag = 28 bytes");
    }

    #[test]
    fn test_calculate_encrypted_size_chacha() {
        let size = calculate_encrypted_size(1000, "ChaCha20-Poly1305");
        assert_eq!(size, 1028, "ChaCha20: +12 nonce + 16 tag = 28 bytes");
    }

    #[test]
    fn test_assess_security_level() {
        assert_eq!(assess_security_level(false, false, false), 0);
        assert_eq!(assess_security_level(false, true, false), 5);
        assert_eq!(assess_security_level(true, false, false), 7);
        assert_eq!(assess_security_level(true, true, false), 9);
        assert_eq!(assess_security_level(true, true, true), 10);
    }

    #[test]
    fn test_needs_key_rotation() {
        assert!(!needs_key_rotation(30, 90));
        assert!(needs_key_rotation(90, 90));
        assert!(needs_key_rotation(365, 90));
    }

    #[test]
    fn test_create_policy_fde() {
        let policy = create_policy(EncryptionApproach::FullDiskEncryption);
        assert_eq!(policy.algorithm, "AES-256-XTS");
        assert_eq!(policy.key_size, 256);
        assert!(!policy.encrypt_metadata);
    }

    #[test]
    fn test_create_policy_fle() {
        let policy = create_policy(EncryptionApproach::FileLevelEncryption);
        assert_eq!(policy.algorithm, "AES-256-GCM");
        assert!(policy.encrypt_metadata);
    }
}
