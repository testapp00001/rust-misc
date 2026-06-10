//! # Module 30: Capstone -- Encrypted Password Vault
//!
//! A capstone project combining all security knowledge into a working encrypted
//! password vault. Covers key derivation, authenticated encryption, secure memory,
//! breach detection, and backup integrity.
//!
//! ## Learning Path
//! 1. Start with `p01_master_key_derivation` -- derive encryption key from passphrase
//! 2. `p02_vault_encryption` / `p03_vault_decryption` -- encrypt and decrypt vault data
//! 3. `p04_entry_management` -- manage password entries securely
//! 4. `p05_secure_clipboard` / `p06_auto_lock` -- user-facing security features
//! 5. `p07_key_rotation` -- change master password safely
//! 6. `p08_breach_detection` -- check passwords against known breaches
//! 7. `p09_secure_backup` / `p10_vault_integrity` -- backup and tamper detection
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 30-capstone-encrypted-vault              # Test your implementation
//! cargo test -p 30-capstone-encrypted-vault --features solution  # Test reference solution
//! ```

// Exercise stubs -- implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_master_key_derivation;
#[cfg(not(feature = "solution"))]
pub mod p02_vault_encryption;
#[cfg(not(feature = "solution"))]
pub mod p03_vault_decryption;
#[cfg(not(feature = "solution"))]
pub mod p04_entry_management;
#[cfg(not(feature = "solution"))]
pub mod p05_secure_clipboard;
#[cfg(not(feature = "solution"))]
pub mod p06_auto_lock;
#[cfg(not(feature = "solution"))]
pub mod p07_key_rotation;
#[cfg(not(feature = "solution"))]
pub mod p08_breach_detection;
#[cfg(not(feature = "solution"))]
pub mod p09_secure_backup;
#[cfg(not(feature = "solution"))]
pub mod p10_vault_integrity;

// Reference solutions -- study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_master_key_derivation.rs"]
pub mod p01_master_key_derivation;
#[cfg(feature = "solution")]
#[path = "solution/p02_vault_encryption.rs"]
pub mod p02_vault_encryption;
#[cfg(feature = "solution")]
#[path = "solution/p03_vault_decryption.rs"]
pub mod p03_vault_decryption;
#[cfg(feature = "solution")]
#[path = "solution/p04_entry_management.rs"]
pub mod p04_entry_management;
#[cfg(feature = "solution")]
#[path = "solution/p05_secure_clipboard.rs"]
pub mod p05_secure_clipboard;
#[cfg(feature = "solution")]
#[path = "solution/p06_auto_lock.rs"]
pub mod p06_auto_lock;
#[cfg(feature = "solution")]
#[path = "solution/p07_key_rotation.rs"]
pub mod p07_key_rotation;
#[cfg(feature = "solution")]
#[path = "solution/p08_breach_detection.rs"]
pub mod p08_breach_detection;
#[cfg(feature = "solution")]
#[path = "solution/p09_secure_backup.rs"]
pub mod p09_secure_backup;
#[cfg(feature = "solution")]
#[path = "solution/p10_vault_integrity.rs"]
pub mod p10_vault_integrity;
