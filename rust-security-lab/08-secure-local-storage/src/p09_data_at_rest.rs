//! # Lesson 09: Data at Rest — Full-Disk vs File-Level Encryption
//!
//! ## What is Data at Rest?
//!
//! "Data at rest" refers to data stored on disk (as opposed to "data in transit"
//! which moves over networks). Protecting data at rest means ensuring that
//! someone with physical access to the storage medium cannot read the data.
//!
//! ## Two Approaches to Encryption
//!
//! ### Full-Disk Encryption (FDE)
//!
//! Encrypts the entire disk volume. The OS decrypts transparently when you log in.
//!
//! | OS | Tool | Algorithm |
//! |----|------|-----------|
//! | Linux | LUKS/dm-crypt | AES-256-XTS |
//! | macOS | FileVault 2 | AES-128/256-XTS |
//! | Windows | BitLocker | AES-128/256-XTS |
//!
//! **Pros**: Transparent, protects everything (including swap, temp files)
//! **Cons**: No protection while the system is running (data is decrypted)
//!
//! ### File-Level Encryption (FLE)
//!
//! Encrypts individual files or directories. User must explicitly encrypt/decrypt.
//!
//! | Tool | Use Case |
//! |------|----------|
//! | GPG/PGP | Email, individual files |
//! | age | Modern file encryption |
//! | EncFS | Encrypted filesystem overlay |
//! | eCryptfs | Linux encrypted home directories |
//!
//! **Pros**: Fine-grained control, protection even while system is running
//! **Cons**: User must manage encryption, metadata may leak
//!
//! ## When to Use Which?
//!
//! | Scenario | Recommendation |
//! |----------|---------------|
//! | Laptop/mobile device | Full-disk encryption (mandatory) |
//! | Cloud server | FDE + file-level for sensitive data |
//! | Shared server | File-level encryption |
//! | Email attachments | File-level (GPG/age) |
//! | Database files | Transparent Data Encryption (TDE) |
//!
//! ## Attack Scenario: Stolen Laptop
//!
//! Without FDE:
//! 1. Attacker removes the hard drive
//! 2. Connects it to another computer as external storage
//! 3. Reads all files directly — passwords, documents, everything
//!
//! With FDE:
//! 1. Attacker removes the hard drive
//! 2. Drive is encrypted — data is unreadable without the key
//! 3. Key is derived from the user's login password + TPM
//! 4. Without the password, the data is cryptographically destroyed

use serde::{Deserialize, Serialize};

/// Encryption approach for data at rest
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EncryptionApproach {
    /// Full-disk encryption (LUKS, FileVault, BitLocker)
    FullDiskEncryption,
    /// File-level encryption (individual files)
    FileLevelEncryption,
    /// Transparent Data Encryption (database-level)
    TransparentDataEncryption,
    /// No encryption (insecure!)
    None,
}

/// A security policy for data at rest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAtRestPolicy {
    /// Name of the policy
    pub name: String,
    /// Encryption approach
    pub approach: EncryptionApproach,
    /// Algorithm used
    pub algorithm: String,
    /// Key size in bits
    pub key_size: u32,
    /// Whether to encrypt metadata (filenames, sizes)
    pub encrypt_metadata: bool,
}

/// Exercise 1: Evaluate whether a data-at-rest policy is secure.
///
/// A policy is "secure" if:
/// - It uses encryption (not `None`)
/// - Key size is at least 256 bits
/// - Algorithm is AES-256-XTS, AES-256-GCM, ChaCha20-Poly1305, or XChaCha20-Poly1305
///
/// # Hints
/// - Match on the approach (None → insecure)
/// - Check key_size >= 256
/// - Check algorithm is in the allowed list
pub fn evaluate_policy(policy: &DataAtRestPolicy) -> bool {
    todo!("Evaluate whether a data-at-rest policy is secure")
}

/// Exercise 2: Recommend an encryption approach based on the scenario.
///
/// # Arguments
/// * `is_portable_device` - Is this a laptop/mobile device?
/// * `is_shared_server` - Is this a server shared by multiple users?
/// * `is_database` - Is this a database?
/// * `contains_phi` - Does it contain protected health information?
///
/// # Hints
/// - Portable device → FullDiskEncryption
/// - Shared server → FileLevelEncryption
/// - Database → TransparentDataEncryption
/// - PHI → Always encrypt (override other recommendations)
pub fn recommend_approach(
    is_portable_device: bool,
    is_shared_server: bool,
    is_database: bool,
    contains_phi: bool,
) -> EncryptionApproach {
    todo!("Recommend encryption approach based on scenario")
}

/// Exercise 3: Create a DataAtRestPolicy with appropriate settings.
///
/// Based on the encryption approach, create a policy with:
/// - Appropriate algorithm
/// - Appropriate key size
/// - Whether to encrypt metadata
///
/// # Hints
/// - FullDiskEncryption → AES-256-XTS, encrypt_metadata=false
/// - FileLevelEncryption → AES-256-GCM, encrypt_metadata=true
/// - TransparentDataEncryption → AES-256-GCM, encrypt_metadata=false
pub fn create_policy(approach: EncryptionApproach) -> DataAtRestPolicy {
    todo!("Create a DataAtRestPolicy with appropriate settings")
}

/// Exercise 4: Check if a file extension indicates sensitive data.
///
/// File-level encryption should be applied to sensitive file types:
/// - .key, .pem, .p12, .pfx (cryptographic keys)
/// - .env, .cfg, .ini (configuration)
/// - .db, .sqlite (databases)
/// - .bak, .dump (backups)
///
/// # Hints
/// - Extract the extension from the filename
/// - Check against a list of sensitive extensions
pub fn is_sensitive_extension(filename: &str) -> bool {
    todo!("Check if a file extension indicates sensitive data")
}

/// Exercise 5: Calculate the overhead of encryption.
///
/// Encryption adds overhead (nonce, auth tag, padding). Calculate
/// the encrypted size given the plaintext size and algorithm.
///
/// # Arguments
/// * `plaintext_size` - Original file size in bytes
/// * `algorithm` - "AES-256-GCM" or "ChaCha20-Poly1305"
///
/// # Hints
/// - AES-256-GCM: +12 (nonce) + 16 (tag) = 28 bytes overhead
/// - ChaCha20-Poly1305: +12 (nonce) + 16 (tag) = 28 bytes overhead
/// - XChaCha20-Poly1305: +24 (nonce) + 16 (tag) = 40 bytes overhead
pub fn calculate_encrypted_size(plaintext_size: u64, algorithm: &str) -> u64 {
    todo!("Calculate encrypted file size including overhead")
}

/// Exercise 6: Assess the security level of a storage configuration.
///
/// Returns a security score from 0 (insecure) to 10 (maximum security).
///
/// # Scoring
/// - No encryption: 0
/// - File-level only: 5
/// - FDE only: 7
/// - FDE + file-level: 9
/// - FDE + file-level + metadata encryption: 10
///
/// # Hints
/// - Check each security feature and add points
pub fn assess_security_level(
    has_fde: bool,
    has_file_encryption: bool,
    encrypts_metadata: bool,
) -> u32 {
    todo!("Assess storage security level (0-10)")
}

/// Exercise 7: Generate a security report for a storage configuration.
///
/// Return a human-readable string describing the security posture.
///
/// # Hints
/// - Describe what encryption is in use
/// - List any security gaps
/// - Provide recommendations
pub fn generate_security_report(
    has_fde: bool,
    has_file_encryption: bool,
    encrypts_metadata: bool,
) -> String {
    todo!("Generate a security report for the storage configuration")
}

/// Exercise 8: Check if a key rotation policy is needed.
///
/// Keys should be rotated periodically. This function checks if the
/// key age exceeds the maximum allowed age.
///
/// # Arguments
/// * `key_created_days_ago` - How many days ago the key was created
/// * `max_age_days` - Maximum allowed key age in days
///
/// # Returns
/// `true` if the key needs rotation
pub fn needs_key_rotation(key_created_days_ago: u32, max_age_days: u32) -> bool {
    todo!("Check if a key rotation is needed")
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
