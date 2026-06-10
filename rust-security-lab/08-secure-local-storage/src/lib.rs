//! # Module 08: Secure Local Storage
//!
//! Encrypting files, secure deletion, temp file safety, and protecting data at rest.
//!
//! ## Learning Path
//! 1. Start with `p01_file_encryption` — understand AES-256-GCM file encryption
//! 2. Then `p02_file_decryption` — the companion decryption
//! 3. Continue through secure deletion, temp files, and permissions
//! 4. Build up to encrypted containers and password-derived keys
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 08-secure-local-storage              # Test your implementation
//! cargo test -p 08-secure-local-storage --features solution  # Test reference solution
//! ```

// Exercise stubs — implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_file_encryption;
#[cfg(not(feature = "solution"))]
pub mod p02_file_decryption;
#[cfg(not(feature = "solution"))]
pub mod p03_secure_deletion;
#[cfg(not(feature = "solution"))]
pub mod p04_temp_file_security;
#[cfg(not(feature = "solution"))]
pub mod p05_filesystem_permissions;
#[cfg(not(feature = "solution"))]
pub mod p06_encrypted_container;
#[cfg(not(feature = "solution"))]
pub mod p07_key_derivation_from_password;
#[cfg(not(feature = "solution"))]
pub mod p08_os_keychain_integration;
#[cfg(not(feature = "solution"))]
pub mod p09_data_at_rest;
#[cfg(not(feature = "solution"))]
pub mod p10_secure_config_files;

// Reference solutions — study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_file_encryption.rs"]
pub mod p01_file_encryption;
#[cfg(feature = "solution")]
#[path = "solution/p02_file_decryption.rs"]
pub mod p02_file_decryption;
#[cfg(feature = "solution")]
#[path = "solution/p03_secure_deletion.rs"]
pub mod p03_secure_deletion;
#[cfg(feature = "solution")]
#[path = "solution/p04_temp_file_security.rs"]
pub mod p04_temp_file_security;
#[cfg(feature = "solution")]
#[path = "solution/p05_filesystem_permissions.rs"]
pub mod p05_filesystem_permissions;
#[cfg(feature = "solution")]
#[path = "solution/p06_encrypted_container.rs"]
pub mod p06_encrypted_container;
#[cfg(feature = "solution")]
#[path = "solution/p07_key_derivation_from_password.rs"]
pub mod p07_key_derivation_from_password;
#[cfg(feature = "solution")]
#[path = "solution/p08_os_keychain_integration.rs"]
pub mod p08_os_keychain_integration;
#[cfg(feature = "solution")]
#[path = "solution/p09_data_at_rest.rs"]
pub mod p09_data_at_rest;
#[cfg(feature = "solution")]
#[path = "solution/p10_secure_config_files.rs"]
pub mod p10_secure_config_files;
