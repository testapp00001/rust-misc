//! # Module 02: Symmetric Encryption
//!
//! AES-256-GCM, ChaCha20-Poly1305, nonce management, key rotation, and attack demonstrations.
//!
//! ## Learning Path
//! 1. Start with `p01_aes_gcm_basics` — understand AEAD encryption
//! 2. Learn ChaCha20-Poly1305 as the software alternative
//! 3. Master nonce management and key rotation
//! 4. Study attacks to understand what goes wrong
//!
//! ## Quick Test
//! ```bash
//! cargo test -p 02-symmetric-encryption              # Test your implementation
//! cargo test -p 02-symmetric-encryption --features solution  # Test reference solution
//! ```

// Exercise stubs — implement these yourself!
#[cfg(not(feature = "solution"))]
pub mod p01_aes_gcm_basics;
#[cfg(not(feature = "solution"))]
pub mod p02_chacha20_poly1305;
#[cfg(not(feature = "solution"))]
pub mod p03_nonce_management;
#[cfg(not(feature = "solution"))]
pub mod p04_key_rotation;
#[cfg(not(feature = "solution"))]
pub mod p05_authenticated_encryption;
#[cfg(not(feature = "solution"))]
pub mod p06_stream_vs_block;
#[cfg(not(feature = "solution"))]
pub mod p07_padding_oracle_attack;
#[cfg(not(feature = "solution"))]
pub mod p08_nonce_reuse_attack;
#[cfg(not(feature = "solution"))]
pub mod p09_additional_data;
#[cfg(not(feature = "solution"))]
pub mod p10_hybrid_encryption;

// Reference solutions — study these after attempting the exercises
#[cfg(feature = "solution")]
#[path = "solution/p01_aes_gcm_basics.rs"]
pub mod p01_aes_gcm_basics;
#[cfg(feature = "solution")]
#[path = "solution/p02_chacha20_poly1305.rs"]
pub mod p02_chacha20_poly1305;
#[cfg(feature = "solution")]
#[path = "solution/p03_nonce_management.rs"]
pub mod p03_nonce_management;
#[cfg(feature = "solution")]
#[path = "solution/p04_key_rotation.rs"]
pub mod p04_key_rotation;
#[cfg(feature = "solution")]
#[path = "solution/p05_authenticated_encryption.rs"]
pub mod p05_authenticated_encryption;
#[cfg(feature = "solution")]
#[path = "solution/p06_stream_vs_block.rs"]
pub mod p06_stream_vs_block;
#[cfg(feature = "solution")]
#[path = "solution/p07_padding_oracle_attack.rs"]
pub mod p07_padding_oracle_attack;
#[cfg(feature = "solution")]
#[path = "solution/p08_nonce_reuse_attack.rs"]
pub mod p08_nonce_reuse_attack;
#[cfg(feature = "solution")]
#[path = "solution/p09_additional_data.rs"]
pub mod p09_additional_data;
#[cfg(feature = "solution")]
#[path = "solution/p10_hybrid_encryption.rs"]
pub mod p10_hybrid_encryption;
